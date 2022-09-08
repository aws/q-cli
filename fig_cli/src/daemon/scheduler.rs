use std::borrow::Cow;
use std::collections::BinaryHeap;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::task::{
    Context,
    Poll,
};
use std::time::Duration;

use async_trait::async_trait;
use eyre::{
    Result,
    WrapErr,
};
use fig_install::dotfiles::api::DotfileData;
use fig_install::dotfiles::download_and_notify;
use fig_install::plugins::fetch_installed_plugins;
use fig_telemetry::TrackEvent;
use fig_util::{
    directories,
    Shell,
};
use flume::Sender;
use rand::distributions::Uniform;
use rand::prelude::Distribution;
use serde_json::Map;
use tokio::task::JoinHandle;
use tokio::time::{
    sleep_until,
    Instant,
    Sleep,
};
use tracing::{
    error,
    info,
};
use yaque::Receiver;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag<'a>(Cow<'a, str>);

impl<'a, S> From<S> for Tag<'a>
where
    S: Into<Cow<'a, str>>,
{
    fn from(s: S) -> Self {
        Tag(s.into())
    }
}

impl std::fmt::Display for Tag<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct ScheduledTask {
    time: Instant,
    tag: Tag<'static>,
    task: Box<dyn Task>,
}

impl ScheduledTask {
    fn new(time: Instant, tag: Tag<'static>, task: Box<dyn Task>) -> Self {
        ScheduledTask { time, tag, task }
    }
}

impl std::cmp::Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time).reverse()
    }
}

impl std::cmp::PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Eq for ScheduledTask {}

impl std::cmp::PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

pub struct ScheduleHeap {
    heap: BinaryHeap<ScheduledTask>,
}

impl Default for ScheduleHeap {
    fn default() -> Self {
        Self::new()
    }
}

impl ScheduleHeap {
    pub fn new() -> ScheduleHeap {
        let final_task = ScheduledTask::new(
            Instant::now() + Duration::from_secs(86400 * 365 * 30),
            Tag::from("final"),
            Box::new(NoopTask),
        );

        ScheduleHeap {
            heap: [final_task].into_iter().collect(),
        }
    }

    pub fn schedule(&mut self, scheduled_task: ScheduledTask) {
        self.heap.push(scheduled_task);
    }

    pub fn cancel_tag(&mut self, tag: &Tag) {
        self.heap = self.heap.drain().filter(|task| task.tag != *tag).collect();
    }

    fn pop(&mut self) -> Option<(Tag<'static>, Box<dyn Task>)> {
        self.heap.pop().map(|ScheduledTask { tag, task, .. }| (tag, task))
    }

    pub async fn next(&mut self) -> Option<(Tag<'static>, Box<dyn Task>)> {
        if let Some(task) = self.heap.peek() {
            sleep_until(task.time).await;
            self.pop()
        } else {
            None
        }
    }
}

pub struct Scheduler {
    incoming_tasks: Sender<SchedulerMessages>,
}

pub enum SchedulerMessages {
    ScheduleTask(ScheduledTask),
    CancelTask(Tag<'static>),
}

impl Scheduler {
    pub async fn spawn() -> (Self, JoinHandle<()>) {
        // A task that is scheduled that will never be executed
        // but will not overflow the time primitive
        let (sender, receiver) = flume::unbounded::<SchedulerMessages>();

        let sender_clone = sender.clone();

        let join_handle = tokio::spawn(async move {
            let mut tasks = ScheduleHeap::new();

            let sender = sender_clone;

            loop {
                tokio::select! {
                    task = tasks.next() => {
                        if let Some((tag, task)) = task {
                            let sender_clone = sender.clone();
                            tokio::spawn(async move {
                                if let Err(err) = task.run(sender_clone).await {
                                    error!("Error running task {}: {}", tag, err);
                                }
                            });
                        }
                    }
                    recv = receiver.recv_async() => {
                        match recv {
                            Ok(SchedulerMessages::ScheduleTask(task)) => {
                                tasks.schedule(task);
                            }
                            Ok(SchedulerMessages::CancelTask(tag)) => {
                                tasks.cancel_tag(&tag);
                            }
                            // This only happens if all channels are closed
                            Err(_) => {
                                break;
                            }
                        }
                    }
                }
            }
        });

        (Self { incoming_tasks: sender }, join_handle)
    }

    pub fn schedule<T, B>(&mut self, task: T, when: Instant)
    where
        T: TaggedTask + Into<Box<B>>,
        B: Task + 'static,
    {
        info!(
            "Scheduling task {} in {:?}",
            task.tag(),
            when.duration_since(Instant::now())
        );
        self.incoming_tasks
            .send(SchedulerMessages::ScheduleTask(ScheduledTask::new(
                when,
                task.tag(),
                task.into(),
            )))
            .unwrap();
    }

    pub fn schedule_delayed<T, B>(&mut self, task: T, delay: Duration)
    where
        T: TaggedTask + Into<Box<B>>,
        B: Task + 'static,
    {
        self.schedule(task, Instant::now() + delay);
    }

    pub fn schedule_now<T, B>(&mut self, task: T)
    where
        T: TaggedTask + Into<Box<B>>,
        B: Task + 'static,
    {
        self.schedule(task, Instant::now());
    }

    pub fn schedule_with_tag<T, B>(&mut self, task: T, when: Instant, tag: Tag<'static>)
    where
        T: Into<Box<B>>,
        B: Task + 'static,
    {
        info!(
            "Scheduling task with tag {} at {:?}",
            tag,
            when.duration_since(Instant::now())
        );
        self.incoming_tasks
            .send(SchedulerMessages::ScheduleTask(ScheduledTask::new(
                when,
                tag,
                task.into(),
            )))
            .unwrap();
    }

    pub fn schedule_delayed_with_tag<T, B>(&mut self, task: T, delay: Duration, tag: Tag<'static>)
    where
        T: Into<Box<B>>,
        B: Task + 'static,
    {
        self.schedule_with_tag(task, Instant::now() + delay, tag);
    }

    pub fn schedule_random_delay<T, B>(&mut self, task: T, min: f64, max: f64)
    where
        T: TaggedTask + Into<Box<B>>,
        B: Task + 'static,
    {
        let dist = Uniform::new(min, max);
        let delay: f64 = dist.sample(&mut rand::thread_rng());
        self.schedule_delayed(task, Duration::from_secs_f64(delay));
    }
}

#[async_trait]
pub trait Task: Send + Sync + Debug {
    async fn run(&self, _sender: Sender<SchedulerMessages>) -> Result<()>;
}

pub trait TaggedTask: Task {
    fn tag(&self) -> Tag<'static> {
        format!("{:?}", self).into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncDotfiles;

#[async_trait]
impl Task for SyncDotfiles {
    async fn run(&self, sender: Sender<SchedulerMessages>) -> Result<()> {
        download_and_notify(false).await?;
        sender
            .send_async(SchedulerMessages::ScheduleTask(ScheduledTask {
                time: Instant::now() + Duration::from_secs_f64(0.5),
                tag: SyncPlugins.tag(),
                task: Box::new(SyncPlugins),
            }))
            .await?;
        Ok(())
    }
}

impl TaggedTask for SyncDotfiles {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncPlugins;

#[async_trait]
impl Task for SyncPlugins {
    async fn run(&self, _sender: Sender<SchedulerMessages>) -> Result<()> {
        fetch_installed_plugins(false).await?;
        Ok(())
    }
}

impl TaggedTask for SyncPlugins {}

trait CloneableTask: TaggedTask {
    fn as_task(&self) -> &dyn TaggedTask;
    fn box_clone(&self) -> Box<dyn CloneableTask>;
}

impl<T: TaggedTask + Clone + 'static> CloneableTask for T {
    fn as_task(&self) -> &dyn TaggedTask {
        self
    }

    fn box_clone(&self) -> Box<dyn CloneableTask> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn CloneableTask> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

#[derive(Debug, Clone)]
pub struct RecurringTask {
    task: Box<dyn CloneableTask>,
    interval: Duration,
    max_iterations: Option<u64>,
    current_iterations: u64,
}

impl RecurringTask {
    pub fn new<T, B>(task: T, interval: Duration, max_iterations: Option<u64>) -> Self
    where
        T: Into<Box<B>>,
        B: TaggedTask + Clone + 'static,
    {
        RecurringTask {
            task: task.into(),
            interval,
            max_iterations,
            current_iterations: 0,
        }
    }
}

#[async_trait]
impl Task for RecurringTask {
    async fn run(&self, sender: Sender<SchedulerMessages>) -> Result<()> {
        let time = Instant::now() + self.interval;
        if self
            .max_iterations
            .map(|n| n < self.current_iterations)
            .unwrap_or(false)
        {
            sender
                .send_async(SchedulerMessages::ScheduleTask(ScheduledTask {
                    time,
                    tag: self.task.as_task().tag(),
                    task: Box::new(RecurringTask {
                        task: self.task.clone(),
                        interval: self.interval,
                        max_iterations: self.max_iterations,
                        current_iterations: self.current_iterations + 1,
                    }),
                }))
                .await?;
        }
        self.task.run(sender).await?;
        Ok(())
    }
}

impl TaggedTask for RecurringTask {
    fn tag(&self) -> Tag<'static> {
        format!("{:?}", self.task.as_task().tag()).into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendQueuedTelemetryEvents;

struct HasSleep {
    sleep: Pin<Box<Sleep>>,
}

impl Future for HasSleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        self.sleep.as_mut().poll(cx)
    }
}

#[async_trait]
impl Task for SendQueuedTelemetryEvents {
    async fn run(&self, _sender: Sender<SchedulerMessages>) -> Result<()> {
        let data_dir = directories::fig_data_dir().context("Could not get data dir")?;
        let mut receiver = Receiver::open(data_dir.join("telemetry-track-event-queue"))?;
        loop {
            let batch = receiver
                .recv_batch_timeout(100, HasSleep {
                    sleep: Box::pin(tokio::time::sleep(Duration::from_secs(10))),
                })
                .await?;
            let tracks: Result<Vec<TrackEvent>> = batch
                .iter()
                .map(|event| -> Result<TrackEvent> { Ok(serde_json::from_slice::<TrackEvent>(event)?) })
                .collect();
            let tracks = tracks?;
            let maybe_more = !tracks.is_empty();
            fig_telemetry::emit_tracks(tracks).await?;

            if !maybe_more {
                break;
            }
        }
        Ok(())
    }
}

impl TaggedTask for SendQueuedTelemetryEvents {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendDotfilesLineCountTelemetry;

fn get_dotfile_line_count(contents: String) -> usize {
    let lines = contents
        .trim()
        .split('\n')
        .filter(|&x| !x.is_empty() && !x.starts_with('#'));
    lines.count()
}

#[async_trait]
impl Task for SendDotfilesLineCountTelemetry {
    async fn run(&self, _sender: Sender<SchedulerMessages>) -> Result<()> {
        let mut stats = Map::new();
        for shell in Shell::all() {
            let (filename, property_name) = match shell {
                Shell::Bash => (".bashrc", "bashrc_line_count"),
                Shell::Zsh => (".zshrc", "zshrc_line_count"),
                Shell::Fish => ("fish.config", "fish_config_line_count"),
            };
            let dotfile = shell.get_config_directory().ok().and_then(|dir| {
                let dotfile_path = dir.join(filename);
                std::fs::read_to_string(&dotfile_path).ok()
            });
            if let Some(contents) = dotfile {
                stats.insert(property_name.into(), get_dotfile_line_count(contents).into());
            }

            let dotfile_data: Option<DotfileData> = shell
                .get_data_path()
                .ok()
                .and_then(|path| std::fs::read_to_string(&path).ok())
                .and_then(|contents| serde_json::from_str(&contents).ok());
            if let Some(data) = dotfile_data {
                stats.insert(
                    format!("fig_{}_line_count", shell),
                    get_dotfile_line_count(data.dotfile).into(),
                );
            }
        }
        fig_telemetry::emit_track(fig_telemetry::TrackEvent::new(
            fig_telemetry::TrackEventType::DotfileLineCountsRecorded,
            fig_telemetry::TrackSource::Daemon,
            env!("CARGO_PKG_VERSION").into(),
            stats,
        ))
        .await?;
        Ok(())
    }
}

impl TaggedTask for SendDotfilesLineCountTelemetry {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoopTask;

#[async_trait]
impl Task for NoopTask {
    async fn run(&self, _sender: Sender<SchedulerMessages>) -> Result<()> {
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[derive(Debug, Clone, PartialEq, Eq)]
//     struct TestTask;

//     #[async_trait]
//     impl Task for TestTask {
//         async fn run(&self, _sender: Sender<SchedulerMessages>) -> Result<()> {
//             Ok(())
//         }
//     }

//     impl TaggedTask for TestTask {}

//     #[tokio::test]
//     async fn test_tasks() {
//         let mut scheduler = Scheduler::new();

//         scheduler.schedule(TestTask, Instant::now());

//         scheduler.schedule_delayed(TestTask, Duration::from_secs(1));

//         scheduler.schedule_with_tag(
//             TestTask,
//             Instant::now() + Duration::from_secs(2),
//             "test1".into(),
//         );

//         scheduler.schedule_delayed_with_tag(TestTask, Duration::from_secs(3), "test2".into());

//         assert_eq!(scheduler.scheduled_tasks.len(), 4);
//     }

//     #[tokio::test]
//     async fn test_scheduler_cancel() {
//         let mut scheduler = Scheduler::new();

//         scheduler.schedule_with_tag(TestTask, Instant::now(), "test1".into());
//         scheduler.schedule_with_tag(TestTask, Instant::now(), "test2".into());
//         scheduler.schedule_with_tag(TestTask, Instant::now(), "test2".into());
//         scheduler.schedule_with_tag(TestTask, Instant::now(), "test3".into());
//         assert_eq!(scheduler.scheduled_tasks.len(), 4);
//         scheduler.cancel_tag(&"test2".into());
//         assert_eq!(scheduler.scheduled_tasks.len(), 2);
//         scheduler.cancel_tag(&"test3".into());
//         assert_eq!(scheduler.scheduled_tasks.len(), 1);
//         scheduler.cancel_tag(&"test1".into());
//         assert_eq!(scheduler.scheduled_tasks.len(), 0);
//     }

//     #[tokio::test]
//     async fn test_next_task() {
//         let mut scheduler = Scheduler::new();

//         scheduler.schedule_delayed_with_tag(TestTask, Duration::from_secs(0), "test1".into());
//         scheduler.schedule_delayed_with_tag(TestTask, Duration::from_secs(1), "test2".into());
//         scheduler.schedule_delayed_with_tag(TestTask, Duration::from_secs(2), "test3".into());

//         assert_eq!(scheduler.scheduled_tasks.len(), 3);
//         assert_eq!(scheduler.pop().unwrap().0, "test1".into());
//         assert_eq!(scheduler.scheduled_tasks.len(), 2);
//         assert_eq!(scheduler.pop().unwrap().0, "test2".into());
//         assert_eq!(scheduler.scheduled_tasks.len(), 1);
//         assert_eq!(scheduler.pop().unwrap().0, "test3".into());
//         assert_eq!(scheduler.scheduled_tasks.len(), 0);
//     }
// }
