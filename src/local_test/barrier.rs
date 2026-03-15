#![allow(dead_code)]

use std::sync::{Condvar, Mutex};

#[derive(Debug, PartialEq, Eq)]
pub enum BarrierError {
    AlreadyUsed,
}

#[derive(Default)]
struct BarrierState {
    arrived: usize,
    completed: bool,
}

pub struct Barrier {
    parties: usize,
    state: Mutex<BarrierState>,
    cvar: Condvar,
}

impl Barrier {
    pub fn new(parties: usize) -> Self {
        assert!(parties > 0, "barrier requires at least one participant");

        Self {
            parties,
            state: Mutex::new(BarrierState::default()),
            cvar: Condvar::new(),
        }
    }

    pub fn wait(&self) -> Result<(), BarrierError> {
        let mut state = self.state.lock().unwrap();

        if state.completed {
            return Err(BarrierError::AlreadyUsed);
        }

        state.arrived += 1;

        if state.arrived == self.parties {
            state.completed = true;
            self.cvar.notify_all();
            return Ok(());
        }

        while !state.completed {
            state = self.cvar.wait(state).unwrap();
        }

        Ok(())
    }
}
