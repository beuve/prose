use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};

#[derive(Clone)]
pub struct Scheduler {
    jobs: Arc<Mutex<Vec<Job>>>,
    time: Arc<AtomicU64>,
}

struct Job {
    time: u64,
    task: Box<dyn FnOnce(u64) + Send + 'static>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(Vec::new())),
            time: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn schedule<F>(&self, delay: u64, f: F)
    where
        F: FnOnce(u64) + Send + 'static,
    {
        let time = self.time.load(Ordering::Relaxed) + delay;
        let mut jobs = self.jobs.lock().unwrap();
        jobs.push(Job {
            time,
            task: Box::new(f),
        });

        jobs.sort_by_key(|j| j.time);
    }

    /// Runs jobs until queue becomes empty
    pub fn run(&self) {
        loop {
            let job = {
                let mut jobs = self.jobs.lock().unwrap();
                if jobs.is_empty() {
                    return;
                }
                jobs.remove(0)
            };

            self.time.store(job.time, Ordering::Relaxed);
            (job.task)(job.time);
        }
    }
}
