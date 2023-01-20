use tokio::time;

pub struct Schedule<T> {
    pub job: T,
    pub deadline: time::Instant,
}

impl<T> Schedule<T> {
    pub fn new(job: T, deadline: time::Instant) -> Self {
        Schedule { job, deadline }
    }

    pub fn instant(job: T) -> Self {
        Schedule::new(job, time::Instant::now())
    }
}

impl<T> PartialEq for Schedule<T> {
    fn eq(&self, other: &Self) -> bool {
        self.deadline == other.deadline
    }
}

impl<T> Eq for Schedule<T> {}

impl<T> PartialOrd for Schedule<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.deadline
            .partial_cmp(&other.deadline)
            .map(|ordering| ordering.reverse())
    }
}

impl<T> Ord for Schedule<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.deadline.cmp(&other.deadline).reverse()
    }
}
