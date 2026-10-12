//! Errors as the UI sees them: a stable code, a message, and for planned
//! features the phase they land in (shown as a "Planned for Phase N" banner).

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CommandError {
    /// `planned`, `rule`, `no-tournament`, `store`, `io` or `invalid`.
    pub code: &'static str,
    pub message: String,
    pub phase: Option<u8>,
}

pub type CmdResult<T> = Result<T, CommandError>;

impl CommandError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            phase: None,
        }
    }

    pub fn no_tournament() -> Self {
        Self::new("no-tournament", "No tournament is open in this window")
    }
}

impl From<co_core::Error> for CommandError {
    fn from(e: co_core::Error) -> Self {
        match e {
            co_core::Error::NotYetImplemented { phase, .. } => Self {
                code: "planned",
                message: e.to_string(),
                phase: Some(phase),
            },
            co_core::Error::Rule(r) => r.into(),
            co_core::Error::Invalid(m) => Self::new("invalid", m),
        }
    }
}

impl From<co_core::RuleError> for CommandError {
    fn from(e: co_core::RuleError) -> Self {
        Self::new("rule", e.to_string())
    }
}

impl From<co_store::StoreError> for CommandError {
    fn from(e: co_store::StoreError) -> Self {
        match e {
            co_store::StoreError::Core(c) => c.into(),
            co_store::StoreError::Rule(r) => r.into(),
            other => Self::new("store", other.to_string()),
        }
    }
}

impl From<co_io::Error> for CommandError {
    fn from(e: co_io::Error) -> Self {
        match e {
            co_io::Error::Core(c) => c.into(),
            other => Self::new("io", other.to_string()),
        }
    }
}

impl From<std::io::Error> for CommandError {
    fn from(e: std::io::Error) -> Self {
        Self::new("io", e.to_string())
    }
}

impl From<tauri::Error> for CommandError {
    fn from(e: tauri::Error) -> Self {
        Self::new("io", e.to_string())
    }
}
