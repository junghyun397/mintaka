use crate::preference::Preference;
use crate::session::Sessions;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

pub struct WorkerPermit(OwnedSemaphorePermit);

impl WorkerPermit {

    pub fn release(self) {
        drop(self.0)
    }

}

pub struct AppState {
    pub sessions: Sessions,
    preference: Preference,
    sem: Arc<Semaphore>,
}

impl AppState {

    pub fn new(preference: Preference) -> Self {
        let sem = Arc::new(Semaphore::new(preference.cores));

        Self {
            sessions: Sessions::default(),
            preference,
            sem,
        }
    }

    pub async fn acquire_worker(&self, workers: u32) -> anyhow::Result<WorkerPermit> {
        Ok(WorkerPermit(self.sem.clone().acquire_many_owned(workers).await?))
    }

}
