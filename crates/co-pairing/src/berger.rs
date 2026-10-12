//! Round robin with FIDE Berger tables, FIDE Handbook C.05 Annex 1,
//! including double round robin.
//!
//! Pairing numbers are the starting ranks. With an odd number of players a
//! dummy number is added and whoever meets it sits the round out. In a
//! double round robin the second cycle repeats the first with colours
//! reversed, and, as C.05 recommends, the last two rounds of the first
//! cycle are swapped so nobody gets three games in a row with one colour.

use crate::{PairingEngine, RoundPairing};
use co_core::pairing_view::{Availability, PairingInput};
use co_core::{ByeKind, Error, Pairing, PairingSystem, PlayerId, Result, RoundNo};

/// Number of players in the table: the field rounded up to even.
pub fn table_size(players: usize) -> usize {
    players + players % 2
}

/// Rounds in one cycle.
pub fn cycle_rounds(players: usize) -> RoundNo {
    table_size(players).saturating_sub(1) as RoundNo
}

/// Games of `round` (1-based, within one cycle) of the Berger table for `n`
/// numbers (`n` even), as 1-based `(white, black)` in board order.
pub fn berger_round(n: usize, round: usize) -> Vec<(usize, usize)> {
    assert!(n >= 2 && n.is_multiple_of(2) && (1..n).contains(&round));
    let m = n - 1;
    // Numbers 1..=m on a circle; `wrap` maps any integer onto it.
    let wrap = |x: usize| (x - 1) % m + 1;
    // Opponent of `n`: the x with 2x ≡ round + 1 (mod m).
    let x = (1..=m)
        .find(|&x| (2 * x) % m == (round + 1) % m)
        .expect("m is odd");
    let mut games = Vec::with_capacity(n / 2);
    games.push(if round.is_multiple_of(2) {
        (n, x)
    } else {
        (x, n)
    });
    for k in 1..n / 2 {
        let white = wrap(x + k);
        let black = wrap(x + m - k);
        games.push((white, black));
    }
    games
}

/// Games of overall round `round` for `players` numbers, with the double
/// round-robin cycle handling described in the module docs.
pub fn schedule(players: usize, round: RoundNo, double: bool) -> Result<Vec<(usize, usize)>> {
    let n = table_size(players);
    let per_cycle = n - 1;
    let r = usize::from(round);
    let cycles = if double { 2 } else { 1 };
    if players < 2 {
        return Err(Error::Invalid(
            "a round robin needs at least two players".into(),
        ));
    }
    if r == 0 || r > per_cycle * cycles {
        return Err(Error::Invalid(format!(
            "a {}round robin of {players} has {} rounds",
            if double { "double " } else { "" },
            per_cycle * cycles
        )));
    }
    let (cycle, mut in_cycle) = ((r - 1) / per_cycle, (r - 1) % per_cycle + 1);
    if double && per_cycle >= 3 && cycle == 0 {
        if in_cycle == per_cycle {
            in_cycle -= 1;
        } else if in_cycle == per_cycle - 1 {
            in_cycle += 1;
        }
    }
    let games = berger_round(n, in_cycle);
    Ok(if cycle == 1 {
        games.into_iter().map(|(w, b)| (b, w)).collect()
    } else {
        games
    })
}

pub struct BergerEngine;

impl PairingEngine for BergerEngine {
    fn system(&self) -> PairingSystem {
        PairingSystem::Berger
    }

    fn pair(&self, input: &PairingInput) -> Result<RoundPairing> {
        // Every entrant keeps their number (starting rank) for the whole
        // event, including players who withdraw.
        let numbered: Vec<_> = input.players.iter().collect();
        let games = schedule(numbered.len(), input.round, input.double)?;
        let mut pairings = Vec::new();
        let mut board = 0;
        for (w, b) in games {
            let (white, black) = (numbered.get(w - 1), numbered.get(b - 1));
            match (white, black) {
                (Some(w), Some(b)) if w.is_available() && b.is_available() => {
                    board += 1;
                    pairings.push(Pairing::Game {
                        board,
                        white: w.id,
                        black: b.id,
                        result: None,
                    });
                }
                (w, b) => {
                    // One side has no opponent this round (the dummy number,
                    // a requested bye or a withdrawal).
                    for (me, other) in [(w, b), (b, w)] {
                        let Some(me) = me else { continue };
                        let kind = match me.availability {
                            Availability::Bye(kind) => kind,
                            Availability::Unavailable => continue,
                            // The opponent withdrew or took a bye: scored as a
                            // forfeit win would be. The free number scores nothing.
                            Availability::Available if other.is_some() => ByeKind::Full,
                            Availability::Available => ByeKind::Zero,
                        };
                        pairings.push(Pairing::Bye {
                            player: me.id,
                            kind,
                        });
                    }
                }
            }
        }
        Ok(RoundPairing {
            pairings,
            matches: Vec::new(),
        })
    }

    /// Players meet twice in a double round robin.
    fn check(&self, input: &PairingInput, proposed: &[Pairing]) -> Vec<crate::Violation> {
        let mut v = crate::check::common(input, proposed);
        if input.double {
            v.retain(|x| !matches!(x, crate::Violation::RepeatPairing(..)));
        }
        v
    }
}

/// Player ids of a Berger round, for tests and the team engine.
pub fn ids_for(
    players: &[PlayerId],
    round: RoundNo,
    double: bool,
) -> Result<Vec<(Option<PlayerId>, Option<PlayerId>)>> {
    Ok(schedule(players.len(), round, double)?
        .into_iter()
        .map(|(w, b)| (players.get(w - 1).copied(), players.get(b - 1).copied()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn six_player_table_matches_c05() {
        let expected = [
            vec![(1, 6), (2, 5), (3, 4)],
            vec![(6, 4), (5, 3), (1, 2)],
            vec![(2, 6), (3, 1), (4, 5)],
            vec![(6, 5), (1, 4), (2, 3)],
            vec![(3, 6), (4, 2), (5, 1)],
        ];
        for (r, games) in expected.iter().enumerate() {
            assert_eq!(&berger_round(6, r + 1), games, "round {}", r + 1);
        }
    }

    #[test]
    fn four_player_table_matches_c05() {
        assert_eq!(berger_round(4, 1), [(1, 4), (2, 3)]);
        assert_eq!(berger_round(4, 2), [(4, 3), (1, 2)]);
        assert_eq!(berger_round(4, 3), [(2, 4), (3, 1)]);
    }

    #[test]
    fn everyone_meets_everyone_once() {
        for n in (4..=20).step_by(2) {
            let mut met = HashSet::new();
            let mut whites = vec![0; n + 1];
            for r in 1..n {
                let games = berger_round(n, r);
                assert_eq!(games.len(), n / 2);
                for (w, b) in games {
                    assert!(met.insert((w.min(b), w.max(b))), "n={n} repeat {w}-{b}");
                    whites[w] += 1;
                }
            }
            assert_eq!(met.len(), n * (n - 1) / 2);
            // Colours are balanced to within one game.
            for &c in &whites[1..] {
                assert!(c == (n - 1) / 2 || c == n / 2, "n={n} whites {whites:?}");
            }
        }
    }

    #[test]
    fn double_round_robin_reverses_colours() {
        let first: Vec<_> = (1..=5)
            .flat_map(|r| schedule(6, r, true).unwrap())
            .collect();
        let second: Vec<_> = (6..=10)
            .flat_map(|r| schedule(6, r, true).unwrap())
            .collect();
        let mut a: Vec<_> = first.iter().map(|&(w, b)| (b, w)).collect();
        let mut b = second.clone();
        a.sort();
        b.sort();
        assert_eq!(a, b);
        // Last two rounds of the first cycle swapped.
        assert_eq!(schedule(6, 4, true).unwrap(), berger_round(6, 5));
        assert_eq!(schedule(6, 5, true).unwrap(), berger_round(6, 4));
        assert!(schedule(6, 11, true).is_err());
    }

    #[test]
    fn odd_fields_use_a_dummy_number() {
        let ids: Vec<_> = (1..=5).map(PlayerId).collect();
        let r1 = ids_for(&ids, 1, false).unwrap();
        assert_eq!(r1[0], (Some(PlayerId(1)), None));
        assert_eq!(cycle_rounds(5), 5);
    }
}
