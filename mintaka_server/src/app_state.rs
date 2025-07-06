use crate::preference::Preference;
use crate::session::Sessions;
use std::sync::atomic::AtomicUsize;

pub struct AppState {
    sessions: Sessions,
    workers: AtomicUsize,
    preference: Preference,
}

impl AppState {
    pub fn new(preference: Preference) -> Self {
        Self {
            sessions: Sessions::default(),
            workers: AtomicUsize::new(preference.cores),
            preference,
        }
    }
}
