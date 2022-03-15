use std::{borrow::Cow, collections::BinaryHeap, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use rand::{distributions::Uniform, prelude::Distribution};
use std::fmt::Debug;
use tokio::time::{sleep_until, Instant};

use crate::{cli::source::sync_all_shells, plugins::api::fetch_installed_plugins};

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

pub struct Scheduler {
    scheduled_tasks: BinaryHeap<ScheduledTask>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        // A task that is scheduled that will never be executed
        // but will not overflow the time primitive
        let final_task = ScheduledTask::new(
            Instant::now() + Duration::from_secs(86400 * 365 * 30),
            Tag::from("final"),
            Box::new(NoopTask),
        );

        Self {
            scheduled_tasks: [final_task].into_iter().collect(),
        }
    }

    pub fn schedule<T, B>(&mut self, task: T, when: Instant)
    where
        T: TaggedTask + Into<Box<B>>,
        B: Task + 'static,
    {
        self.scheduled_tasks
            .push(ScheduledTask::new(when, task.tag(), task.into()));
    }

    pub fn schedule_delayed<T, B>(&mut self, task: T, delay: Duration)
    where
        T: TaggedTask + Into<Box<B>>,
        B: Task + 'static,
    {
        self.schedule(task, Instant::now() + delay);
    }

    pub fn schedule_with_tag<T, B>(&mut self, task: T, when: Instant, tag: Tag<'static>)
    where
        T: Into<Box<B>>,
        B: Task + 'static,
    {
        self.scheduled_tasks
            .push(ScheduledTask::new(when, tag, task.into()));
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

    pub fn cancel_tag(&mut self, tag: &Tag) {
        self.scheduled_tasks = self
            .scheduled_tasks
            .drain()
            .filter(|task| task.tag != *tag)
            .collect();
    }

    fn pop(&mut self) -> Option<(Tag<'static>, Box<dyn Task>)> {
        self.scheduled_tasks
            .pop()
            .map(|ScheduledTask { tag, task, .. }| (tag, task))
    }

    pub async fn next(&mut self) -> Option<(Tag<'static>, Box<dyn Task>)> {
        if let Some(task) = self.scheduled_tasks.peek() {
            sleep_until(task.time).await;
            self.pop()
        } else {
            None
        }
    }
}

#[async_trait]
pub trait Task: Send + Sync + Debug {
    async fn run(&self) -> Result<()>;
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
    async fn run(&self) -> Result<()> {
        sync_all_shells().await?;
        Ok(())
    }
}

impl TaggedTask for SyncDotfiles {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncPlugins;

#[async_trait]
impl Task for SyncPlugins {
    async fn run(&self) -> Result<()> {
        fetch_installed_plugins().await?;
        Ok(())
    }
}

impl TaggedTask for SyncPlugins {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoopTask;

#[async_trait]
impl Task for NoopTask {
    async fn run(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestTask;

    #[async_trait]
    impl Task for TestTask {
        async fn run(&self) -> Result<()> {
            Ok(())
        }
    }

    impl TaggedTask for TestTask {}

    #[tokio::test]
    async fn test_tasks() {
        let mut scheduler = Scheduler::new();

        scheduler.schedule(TestTask, Instant::now());

        scheduler.schedule_delayed(TestTask, Duration::from_secs(1));

        scheduler.schedule_with_tag(
            TestTask,
            Instant::now() + Duration::from_secs(2),
            "test1".into(),
        );

        scheduler.schedule_delayed_with_tag(TestTask, Duration::from_secs(3), "test2".into());

        assert_eq!(scheduler.scheduled_tasks.len(), 4);
    }

    #[tokio::test]
    async fn test_scheduler_cancel() {
        let mut scheduler = Scheduler::new();

        scheduler.schedule_with_tag(TestTask, Instant::now(), "test1".into());
        scheduler.schedule_with_tag(TestTask, Instant::now(), "test2".into());
        scheduler.schedule_with_tag(TestTask, Instant::now(), "test2".into());
        scheduler.schedule_with_tag(TestTask, Instant::now(), "test3".into());
        assert_eq!(scheduler.scheduled_tasks.len(), 4);
        scheduler.cancel_tag(&"test2".into());
        assert_eq!(scheduler.scheduled_tasks.len(), 2);
        scheduler.cancel_tag(&"test3".into());
        assert_eq!(scheduler.scheduled_tasks.len(), 1);
        scheduler.cancel_tag(&"test1".into());
        assert_eq!(scheduler.scheduled_tasks.len(), 0);
    }

    #[tokio::test]
    async fn test_next_task() {
        let mut scheduler = Scheduler::new();

        scheduler.schedule_delayed_with_tag(TestTask, Duration::from_secs(0), "test1".into());
        scheduler.schedule_delayed_with_tag(TestTask, Duration::from_secs(1), "test2".into());
        scheduler.schedule_delayed_with_tag(TestTask, Duration::from_secs(2), "test3".into());

        assert_eq!(scheduler.scheduled_tasks.len(), 3);
        assert_eq!(scheduler.pop().unwrap().0, "test1".into());
        assert_eq!(scheduler.scheduled_tasks.len(), 2);
        assert_eq!(scheduler.pop().unwrap().0, "test2".into());
        assert_eq!(scheduler.scheduled_tasks.len(), 1);
        assert_eq!(scheduler.pop().unwrap().0, "test3".into());
        assert_eq!(scheduler.scheduled_tasks.len(), 0);
    }
}
