use std::{cmp, time::Duration};

use std::future::Future;

pub struct Backoff {
    current_duration: Duration,
    min_duration: Duration,
    max_duration: Duration,
}

impl Backoff {
    pub fn new(min_duration: Duration, max_duration: Duration) -> Self {
        Self {
            current_duration: min_duration,
            min_duration,
            max_duration,
        }
    }

    pub fn reset(&mut self) {
        self.current_duration = self.min_duration;
    }

    pub async fn sleep(&mut self) {
        let duration = self.current_duration;
        self.current_duration = cmp::min(self.current_duration * 2, self.max_duration);
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
