//! Chess Organiser core: the tournament domain model and its event log.
//!
//! All tournament state is derived by replaying [`event::Event`]s through
//! [`model::Tournament::apply`]. Nothing in this crate knows about storage,
//! user interfaces or pairing algorithms.

pub mod error;
pub mod event;
pub mod history;
pub mod ids;
pub mod model;
pub mod pairing_view;
pub mod points;
pub mod reducer;
pub mod scoring;
pub mod tiebreak_spec;

pub use error::{Error, Result, RuleError};
pub use event::{Envelope, Event};
pub use ids::{PlayerId, PrizeCategoryId, RoundNo, TeamId};
pub use model::*;
pub use points::Points;
pub use tiebreak_spec::{TiebreakKind, TiebreakModifiers, TiebreakSpec};

// Re-exported so downstream crates agree on versions.
pub use chrono;
pub use uuid::{self, Uuid};

/// The project phase in which a stubbed feature is scheduled to land.
pub type Phase = u8;

/// Shorthand for returning [`Error::NotYetImplemented`] from a stub.
#[macro_export]
macro_rules! not_yet {
    ($feature:expr, $phase:expr) => {
        Err($crate::Error::NotYetImplemented {
            feature: $feature,
            phase: $phase,
        })
    };
}
