#[derive(Debug)]
pub enum CoreState {
    Created,
    Configured,
    Authorized,
    Running,
    Stopped,
}
