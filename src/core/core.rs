use super::config::model::CoreLinkConfig;
use super::state::CoreState;

pub struct CoreLink {
    state: CoreState,
    config: Option<CoreLinkConfig>,
}

impl CoreLink {
    pub fn new() -> Self {
        Self {
            state: CoreState::Created,
            config: None,
        }
    }

    pub fn state(&self) -> &CoreState {
        &self.state
    }

    pub fn set_config(&mut self, config: CoreLinkConfig) {
        self.config = Some(config);
    }
}
