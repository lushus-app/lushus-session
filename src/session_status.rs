#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionStatus {
    Changed,
    Unchanged,
    Destroyed,
}

impl Default for SessionStatus {
    fn default() -> Self {
        SessionStatus::Unchanged
    }
}
