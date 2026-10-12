//! Team round robin: the Berger tables (C.05 Annex 1) over teams in
//! team-list order. The first-named team of each match is the home team and
//! has White on odd boards. With an odd number of teams, the team drawn
//! against the free number sits the round out.

use crate::berger::schedule;
use crate::team_swiss::build_match;
use crate::{PairingEngine, RoundPairing};
use co_core::pairing_view::PairingInput;
use co_core::{Error, PairingSystem, Result};

pub struct TeamRoundRobinEngine;

impl PairingEngine for TeamRoundRobinEngine {
    fn system(&self) -> PairingSystem {
        PairingSystem::TeamRoundRobin
    }

    fn pair(&self, input: &PairingInput) -> Result<RoundPairing> {
        if input.teams.len() < 2 {
            return Err(Error::Invalid(
                "a team round robin needs at least two teams".into(),
            ));
        }
        let mut matches = Vec::new();
        for (home, away) in schedule(input.teams.len(), input.round, input.double)? {
            if let (Some(h), Some(a)) = (input.teams.get(home - 1), input.teams.get(away - 1)) {
                matches.push(build_match(input, matches.len() as u16 + 1, h, Some(a)));
            }
        }
        Ok(RoundPairing {
            pairings: Vec::new(),
            matches,
        })
    }

    fn check(
        &self,
        _input: &PairingInput,
        _proposed: &[co_core::Pairing],
    ) -> Vec<crate::Violation> {
        Vec::new()
    }
}
