use std::cmp;
use std::time::Duration;

use rand::Rng;

pub struct Backoff {
    attempt: u32,
    min_duration: Duration,
    max_duration: Duration,
}

impl Backoff {
    pub fn new(min_duration: Duration, max_duration: Duration) -> Self {
        assert!(min_duration < max_duration);
        Self {
            attempt: 0,
            min_duration,
            max_duration,
        }
    }

    pub fn reset(&mut self) {
        self.attempt = 0;
    }

    /// The sleep uses the equal jitter algorithm as described
    /// [here](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter)
    pub async fn sleep(&mut self) {
        let sleep = {
            let mut rng = rand::thread_rng();
            let temp = cmp::min(self.max_duration, self.min_duration * 2_u32.pow(self.attempt));
            temp / 2 + rng.gen_range(Duration::ZERO..=temp / 2)
        };
        self.attempt += 1;
        tokio::time::sleep(sleep).await;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_backoff() {
        let mut backoff = Backoff::new(Duration::from_millis(1), Duration::from_millis(16));
        let mut count = 0;
        loop {
            count += 1;
            if count == 10 {
                break;
            }
            backoff.sleep().await;
        }
    }
}
