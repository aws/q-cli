use std::borrow::Cow;
use std::collections::BinaryHeap;
use std::fmt::Debug;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use fig_install::dotfiles::download_and_notify;
use fig_install::plugins::fetch_installed_plugins;
use flume::Sender;
use rand::distributions::Uniform;
use rand::prelude::Distribution;
use tokio::task::JoinHandle;
use tokio::time::{
    sleep_until,
    Instant,
};
use tracing::{
    error,
    info,
};

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
    incomming_tasks: Sender<SchedulerMessages>,
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

        (
            Self {
                incomming_tasks: sender,
            },
            join_handle,
        )
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
        self.incomming_tasks
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
        self.incomming_tasks
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
        download_and_notify().await?;
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
