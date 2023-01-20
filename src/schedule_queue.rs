use std::collections::BinaryHeap;
use std::error::Error;

use tokio::time;

use crate::schedule::Schedule;

#[async_trait::async_trait]
pub trait ScheduleQueue<T>
where
    T: Sync + Send,
{
    fn new() -> Self;

    fn is_first_urgent(&self) -> bool;

    async fn wait_for_first(&self) -> Result<(), Box<dyn Error>>;
}

#[async_trait::async_trait]
impl<T> ScheduleQueue<T> for BinaryHeap<Schedule<T>>
where
    T: Sync + Send,
{
    fn new() -> Self {
        BinaryHeap::new()
    }

    fn is_first_urgent(&self) -> bool {
        if self.is_empty() {
            return false;
        }

        let first_schedule = self.peek().unwrap();

        if first_schedule.deadline > time::Instant::now() {
            return false;
        }

        return true;
    }

    async fn wait_for_first(&self) -> Result<(), Box<dyn Error>> {
        if self.is_empty() {
            return Err("no schedule".into());
        }

        let first_schedule = self.peek().unwrap();

        time::sleep_until(first_schedule.deadline).await;

        Ok(())
    }
}
