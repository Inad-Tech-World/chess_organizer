//! Monrad (simple Swiss) for club play: players ordered by score, then
//! starting rank; the highest unpaired player meets the next player below
//! them who has not been met yet, provided everyone left can still be
//! paired. No colour or float optimisation beyond the absolute criteria.

use crate::bracket::Field;
use crate::swiss_model::Tournament;
use crate::{PairingEngine, RoundPairing};
use co_core::pairing_view::PairingInput;
use co_core::{Error, PairingSystem, Result};

pub struct MonradEngine;

impl PairingEngine for MonradEngine {
    fn system(&self) -> PairingSystem {
        PairingSystem::Monrad
    }

    fn pair(&self, input: &PairingInput) -> Result<RoundPairing> {
        let t = Tournament::from_input(input)?;
        let f = Field::new(&t, input);
        let mut order = f.available();
        order.sort_by(|&a, &b| {
            f.score(b)
                .cmp(&f.score(a))
                .then_with(|| f.player(a).rank_index.cmp(&f.player(b).rank_index))
        });
        let mut pairs = Vec::new();
        if let Some(bye) = f.choose_bye(&order)? {
            order.retain(|&p| p != bye);
            pairs.push((bye, bye));
        }
        while !order.is_empty() {
            let top = order[0];
            let found = (1..order.len()).find(|&k| {
                let cand = order[k];
                f.compatible(top, cand) && {
                    let rest: Vec<usize> = order
                        .iter()
                        .copied()
                        .filter(|&p| p != top && p != cand)
                        .collect();
                    f.perfect(&rest)
                }
            });
            let Some(k) = found else {
                return Err(Error::Invalid(
                    "no valid pairing exists: the players cannot all be paired without a rematch"
                        .into(),
                ));
            };
            let cand = order.remove(k);
            order.remove(0);
            pairs.push(f.colours(top, cand));
        }
        Ok(t.to_round(input, pairs))
    }
}
