use std::{borrow::Cow, collections::BinaryHeap, pin::Pin, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use futures::{future, stream::FuturesUnordered, FutureExt, StreamExt};
use std::fmt::Debug;
use tokio::time::{sleep_until, Instant, Sleep};

use crate::{cli::source::sync_all_shells, plugins::api::fetch_installed_plugins};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Tag<'a>(Cow<'a, str>);

impl<'a, S> From<S> for Tag<'a>
where
    S: Into<Cow<'a, str>>,
{
    fn from(s: S) -> Self {
        Tag(s.into())
    }
}

#[derive(Debug)]
struct ScheduledTask {
    sleep: Sleep,
    tag: Tag<'static>,
    task: Box<dyn Task>,
}

impl ScheduledTask {
    fn new(sleep: Sleep, tag: Tag<'static>, task: Box<dyn Task>) -> Self {
        ScheduledTask {
            sleep: sleep,
            tag: tag,
            task: task,
        }
    }
}

impl std::cmp::Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sleep.deadline().cmp(&other.sleep.deadline())
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
        self.sleep.deadline() == other.sleep.deadline()
    }
}

pub struct Scheduler {
    scheduled_tasks: BinaryHeap<ScheduledTask>,
}

impl Scheduler {
    fn new() -> Self {
        Self {
            scheduled_tasks: BinaryHeap::new(),
        }
    }

    fn schedule<T, B>(&mut self, task: T, when: Instant)
    where
        T: TaggedTask + Into<Box<B>>,
        B: Task + 'static,
    {
        self.scheduled_tasks.push(ScheduledTask::new(
            sleep_until(when),
            task.tag(),
            task.into(),
        ));
    }

    fn schedule_delayed<T, B>(&mut self, task: T, delay: Duration)
    where
        T: TaggedTask + Into<Box<B>>,
        B: Task + 'static,
    {
        self.schedule(task, Instant::now() + delay);
    }

    fn schedule_with_tag<T, B>(&mut self, task: T, when: Instant, tag: Tag<'static>)
    where
        T: Into<Box<B>>,
        B: Task + 'static,
    {
        self.scheduled_tasks
            .push(ScheduledTask::new(sleep_until(when), tag, task.into()));
    }

    fn schedule_delayed_with_tag<T, B>(&mut self, task: T, delay: Duration, tag: Tag<'static>)
    where
        T: Into<Box<B>>,
        B: Task + 'static,
    {
        self.schedule_with_tag(task, Instant::now() + delay, tag);
    }

    fn cancel_tag(&mut self, tag: &Tag) {
        // TODO: here
    }

    async fn next_task(&mut self) -> Option<(Tag<'static>, Box<dyn Task>)> {
        if let Some(task) = self.scheduled_tasks.peek() {
            // task.sleep.await;
            let ScheduledTask { tag, task, .. } = self.scheduled_tasks.pop().unwrap();
            Some((tag, task))
        } else {
            None
        }
    }
}

#[async_trait]
trait Task: Send + Sync + Debug {
    async fn run(&self) -> Result<()>;
}

trait TaggedTask: Task {
    fn tag(&self) -> Tag<'static> {
        format!("{:?}", self).into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SyncDotfiles;

#[async_trait]
impl Task for SyncDotfiles {
    async fn run(&self) -> Result<()> {
        sync_all_shells().await?;
        Ok(())
    }
}

impl TaggedTask for SyncDotfiles {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SyncPlugins;

#[async_trait]
impl Task for SyncPlugins {
    async fn run(&self) -> Result<()> {
        fetch_installed_plugins().await?;
        Ok(())
    }
}

impl TaggedTask for SyncPlugins {}

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
        scheduler.schedule_with_tag(TestTask, Instant::now(), "test3".into());

        scheduler.cancel_tag(&"test2".into());

        assert_eq!(scheduler.scheduled_tasks.len(), 3);
    }

    #[tokio::test]
    async fn test_next_task() {
        let mut scheduler = Scheduler::new();

        scheduler.schedule_delayed_with_tag(TestTask, Duration::from_secs(0), "test1".into());
        scheduler.schedule_delayed_with_tag(TestTask, Duration::from_secs(1), "test2".into());

        assert_eq!(scheduler.scheduled_tasks.len(), 2);
        assert_eq!(scheduler.next_task().await.unwrap().0, "test1".into());
        assert_eq!(scheduler.scheduled_tasks.len(), 1);
        assert_eq!(scheduler.next_task().await.unwrap().0, "test2".into());
        assert_eq!(scheduler.scheduled_tasks.len(), 0);
    }
}
