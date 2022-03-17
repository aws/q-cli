use std::{cmp, time::Duration};

use std::future::Future;

pub struct Backoff {
    duration: Duration,
    max_duration: Duration,
}

impl Backoff {
    pub fn new(duration: Duration, max_duration: Duration) -> Self {
        Self {
            duration,
            max_duration,
        }
    }

    pub fn reset(&mut self) {
        self.duration = self.max_duration;
    }

    pub async fn sleep(&mut self) {
        let duration = self.duration;
        self.duration = cmp::min(self.duration * 2, self.max_duration);
        tokio::time::sleep(duration).await;
    }

    // This will execute the future with the backoff and will never return
    pub async fn execute<F, Fut, T, E>(&mut self, mut _f: F) -> !
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        todo!();
    }
}
