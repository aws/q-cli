use std::cmp;
use std::future::Future;
use std::time::Duration;

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

    // This will execute the future with the backoff and should never return
    pub async fn execute<F, Fut>(&mut self, mut f: F) -> !
    where
        F: FnMut(&mut Backoff) -> Fut,
        Fut: Future<Output = ()>,
    {
        loop {
            f(self).await;
            self.sleep().await;
        }
    }
}
