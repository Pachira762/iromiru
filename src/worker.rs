use crate::gui::compositor::Compositor;
use crate::gui::Window;
use crate::pass::Executor;
use crate::state::RefState;
use windows::core::*;

pub struct Worker {
    state: RefState,
    handler: Option<std::thread::JoinHandle<Result<()>>>,
}

impl Worker {
    pub fn new(state: RefState, window: Window, compositor: &mut Compositor) -> Result<Self> {
        let mut executor = Executor::new(RefState::clone(&state), window, compositor)?;
        let handler = Some(std::thread::spawn(move || executor.execute()));

        Ok(Self { state, handler })
    }

    pub fn stop(&mut self) {
        if let Some(handler) = self.handler.take() {
            self.state.set_active(false);
            _ = handler.join();
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop();
    }
}
