use serde::{Deserialize, Serialize};
use std::fmt;

/// 1-based round number.
pub type RoundNo = u16;

macro_rules! id_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
        pub struct $name(pub u32);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

id_type!(
    /// Stable identifier of a player within one tournament. Never reused.
    PlayerId
);
id_type!(
    /// Stable identifier of a team within one tournament.
    TeamId
);
id_type!(
    /// Stable identifier of a prize category within one tournament.
    PrizeCategoryId
);
