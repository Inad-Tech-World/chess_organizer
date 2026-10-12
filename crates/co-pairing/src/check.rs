//! Rule checks that apply to every individual pairing system (C.04.1).

use co_core::pairing_view::{Availability, PairingInput};
use co_core::{ByeKind, Pairing, PlayerId};
use std::collections::HashSet;
use std::fmt;

/// A rule a proposed pairing breaks. The arbiter may still override it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Violation {
    /// The two players have already met.
    RepeatPairing(PlayerId, PlayerId),
    /// The players are in the same forbidden group (club, family, ...).
    ForbiddenPairing(PlayerId, PlayerId),
    /// A withdrawn, absent or not-yet-joined player has a game.
    UnavailablePlayerPaired(PlayerId),
    /// An available player has neither a game nor a bye.
    PlayerNotPaired(PlayerId),
    /// The player already received a pairing-allocated bye.
    RepeatPairingBye(PlayerId),
    /// More than one pairing-allocated bye in the round.
    MultiplePairingByes,
    /// Some rule specific to the pairing system.
    System(String),
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Violation::RepeatPairing(a, b) => write!(f, "players {a} and {b} have already met"),
            Violation::ForbiddenPairing(a, b) => {
                write!(
                    f,
                    "players {a} and {b} must not be paired (forbidden group)"
                )
            }
            Violation::UnavailablePlayerPaired(p) => {
                write!(f, "player {p} is not available this round")
            }
            Violation::PlayerNotPaired(p) => write!(f, "player {p} has no game and no bye"),
            Violation::RepeatPairingBye(p) => {
                write!(f, "player {p} has already received a pairing-allocated bye")
            }
            Violation::MultiplePairingByes => {
                write!(f, "more than one pairing-allocated bye in this round")
            }
            Violation::System(msg) => f.write_str(msg),
        }
    }
}

/// Checks shared by all systems: no repeat or forbidden pairings, only
/// available players paired, everyone accounted for, at most one PAB per
/// round and per player.
pub fn common(input: &PairingInput, proposed: &[Pairing]) -> Vec<Violation> {
    let mut out = Vec::new();
    let mut covered = HashSet::new();
    let mut pabs = 0;
    for p in proposed {
        covered.extend(p.players());
        match *p {
            Pairing::Game { white, black, .. } => {
                for id in [white, black] {
                    let available = input.player(id).is_some_and(|pp| pp.is_available());
                    if !available {
                        out.push(Violation::UnavailablePlayerPaired(id));
                    }
                }
                let met = input
                    .player(white)
                    .is_some_and(|pp| pp.opponents().any(|o| o == black));
                if met {
                    out.push(Violation::RepeatPairing(white, black));
                }
                if input.is_forbidden(white, black) {
                    out.push(Violation::ForbiddenPairing(white, black));
                }
            }
            Pairing::Bye {
                player,
                kind: ByeKind::PairingAllocated,
            } => {
                pabs += 1;
                if input.player(player).is_some_and(|pp| pp.had_pairing_bye()) {
                    out.push(Violation::RepeatPairingBye(player));
                }
            }
            Pairing::Bye { .. } => {}
        }
    }
    if pabs > 1 {
        out.push(Violation::MultiplePairingByes);
    }
    for p in &input.players {
        if p.availability != Availability::Unavailable && !covered.contains(&p.id) {
            out.push(Violation::PlayerNotPaired(p.id));
        }
    }
    out
}
