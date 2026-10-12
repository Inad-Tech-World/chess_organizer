//! Pairing engines.
//!
//! Every system implements [`PairingEngine`] over the shared
//! [`PairingInput`] view, so the desktop app, the CLI and the conformance
//! harness drive all systems the same way.

pub mod baku;
pub mod berger;
mod bracket;
pub mod burstein;
pub mod check;
pub mod dubov;
pub mod dutch;
pub mod lim;
pub mod manual;
pub mod matching;
pub mod monrad;
mod swiss_model;
pub mod team_rr;
pub mod team_swiss;

use co_core::pairing_view::{Availability, PairingInput};
use co_core::{ByeKind, Pairing, PairingSystem, PlayerId, Result, TeamMatch};

pub use check::Violation;

/// The pairings an engine proposes for one round.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RoundPairing {
    pub pairings: Vec<Pairing>,
    pub matches: Vec<TeamMatch>,
}

pub trait PairingEngine: Send + Sync {
    fn system(&self) -> PairingSystem;

    /// Pairs `input.round`.
    fn pair(&self, input: &PairingInput) -> Result<RoundPairing>;

    /// Lists rule violations in proposed pairings, e.g. after a manual edit.
    /// Systems refine this with their own absolute criteria.
    fn check(&self, input: &PairingInput, proposed: &[Pairing]) -> Vec<Violation> {
        check::common(input, proposed)
    }
}

/// The engine for a pairing system.
pub fn engine_for(system: PairingSystem) -> Box<dyn PairingEngine> {
    match system {
        PairingSystem::Dutch => Box::new(dutch::DutchEngine),
        PairingSystem::Dubov => Box::new(dubov::DubovEngine),
        PairingSystem::Burstein => Box::new(burstein::BursteinEngine),
        PairingSystem::Lim => Box::new(lim::LimEngine),
        PairingSystem::Monrad => Box::new(monrad::MonradEngine),
        PairingSystem::Berger => Box::new(berger::BergerEngine),
        PairingSystem::TeamSwiss => Box::new(team_swiss::TeamSwissEngine),
        PairingSystem::TeamRoundRobin => Box::new(team_rr::TeamRoundRobinEngine),
    }
}

/// Turns ordered `(white, black)` games into numbered boards and adds a bye
/// entry for every player who is not in a game: their requested bye, or a
/// zero-point bye if they are absent. `pab` gets the pairing-allocated bye.
/// Withdrawn players and late entries who have not joined get no entry.
pub fn finalize_round(
    input: &PairingInput,
    games: &[(PlayerId, PlayerId)],
    pab: Option<PlayerId>,
) -> Vec<Pairing> {
    let mut pairings: Vec<Pairing> = games
        .iter()
        .enumerate()
        .map(|(i, &(white, black))| Pairing::Game {
            board: i as u16 + 1,
            white,
            black,
            result: None,
        })
        .collect();
    let in_game = |id: PlayerId| games.iter().any(|&(w, b)| w == id || b == id);
    for p in &input.players {
        if in_game(p.id) {
            continue;
        }
        let kind = if Some(p.id) == pab {
            Some(ByeKind::PairingAllocated)
        } else {
            match p.availability {
                Availability::Bye(kind) => Some(kind),
                Availability::Available => Some(ByeKind::Zero),
                Availability::Unavailable => None,
            }
        };
        if let Some(kind) = kind {
            pairings.push(Pairing::Bye { player: p.id, kind });
        }
    }
    pairings
}
