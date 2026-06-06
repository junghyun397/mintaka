use crate::protocol::response::{Response, ResponseSender};
use crate::time_manager::TimeManager;
use crate::utils::monotonic_clock::MonotonicClock;

pub trait ThreadType {
    const IS_MAIN: bool;

    type Clock: MonotonicClock;

    fn make_response(&self, response: Response);

    fn time_manager(&self) -> &TimeManager<Self::Clock>;

    fn time_manager_mut(&mut self) -> &mut TimeManager<Self::Clock>;
}

pub struct MainThread<CLK: MonotonicClock, T: ResponseSender> {
    response_sender: T,
    time_manager: TimeManager<CLK>
}

impl<CLK: MonotonicClock, T: ResponseSender> MainThread<CLK, T> {
    pub fn new(
        response_sender: T,
        time_manager: TimeManager<CLK>,
    ) -> Self {
        Self {
            response_sender,
            time_manager,
        }
    }
}

impl<CLK: MonotonicClock, T: ResponseSender> ThreadType for MainThread<CLK, T> {
    const IS_MAIN: bool = true;

    type Clock = CLK;

    fn make_response(&self, response: Response) {
        self.response_sender.response(response);
    }

    fn time_manager(&self) -> &TimeManager<CLK> {
        &self.time_manager
    }

    fn time_manager_mut(&mut self) -> &mut TimeManager<CLK> {
        &mut self.time_manager
    }
}

#[derive(Clone)]
pub struct WorkerThread<CLK: MonotonicClock> {
    _phantom: std::marker::PhantomData<CLK>,
}

impl<CLK: MonotonicClock> WorkerThread<CLK> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<CLK: MonotonicClock> ThreadType for WorkerThread<CLK> {
    const IS_MAIN: bool = false;

    type Clock = CLK;

    fn make_response(&self, _response: Response) {
        unreachable!();
    }

    fn time_manager(&self) -> &TimeManager<CLK> {
        unreachable!()
    }

    fn time_manager_mut(&mut self) -> &mut TimeManager<CLK> {
        unreachable!()
    }
}
