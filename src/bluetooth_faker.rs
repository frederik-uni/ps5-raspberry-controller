use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::time::sleep;

use crate::interfaces::bluetooth::ControllerState;

pub struct DualSenseController {
    state: Arc<Mutex<ControllerState>>,
}

impl DualSenseController {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ControllerState::default())),
        }
    }

    pub fn get_state(&self) -> ControllerState {
        let state = self.state.lock().unwrap();
        state.clone()
    }

    pub fn update_state<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut ControllerState),
    {
        let mut state = self.state.lock().unwrap();
        update_fn(&mut state);
    }

    pub async fn run_report_loop(&self) {
        loop {
            sleep(Duration::from_millis(200)).await
        }
    }

    pub async fn initialize_bluetooth(&self) -> Result<(), ()> {
        Ok(())
    }
}
