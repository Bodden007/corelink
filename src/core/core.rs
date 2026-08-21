use super::state::CoreState;

pub struct CoreLink {
    state: CoreState,
}

impl CoreLink {
    pub fn new() -> Self {
        Self {
            state: CoreState::Created,
        }
    }

    pub fn state(&self) -> &CoreState {
        &self.state
    }
}
