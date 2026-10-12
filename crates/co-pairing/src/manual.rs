//! Manual pairing: the arbiter decides every game and the program only
//! checks the result (C.04.1 / C.04.2 rules shared by all systems).

use crate::check::{self, Violation};
use crate::{RoundPairing, finalize_round};
use co_core::pairing_view::PairingInput;
use co_core::{Pairing, PlayerId};

/// Builds a full round from the arbiter's games and reports any violations.
pub fn manual_round(
    input: &PairingInput,
    games: &[(PlayerId, PlayerId)],
    pab: Option<PlayerId>,
) -> (RoundPairing, Vec<Violation>) {
    let pairings: Vec<Pairing> = finalize_round(input, games, pab);
    let violations = check::common(input, &pairings);
    (
        RoundPairing {
            pairings,
            matches: Vec::new(),
        },
        violations,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use co_core::pairing_view::PairingInput;
    use co_core::*;
    use uuid::Uuid;

    fn tournament(n: u32) -> Tournament {
        let mut t = Tournament::create(&Event::TournamentCreated {
            id: Uuid::nil(),
            info: TournamentInfo::default(),
            settings: Settings::default(),
            kind: TournamentKind::default(),
        })
        .unwrap();
        for i in 1..=n {
            t.apply(&Event::PlayerAdded {
                player: Player::new(PlayerId(i), format!("P{i}")),
            })
            .unwrap();
        }
        t
    }

    #[test]
    fn fills_in_byes_and_flags_violations() {
        let mut t = tournament(5);
        t.apply(&Event::ByeSet {
            player: PlayerId(4),
            round: 1,
            kind: Some(ByeKind::Half),
        })
        .unwrap();
        let input = PairingInput::for_next_round(&t);
        let (round, violations) =
            manual_round(&input, &[(PlayerId(1), PlayerId(2))], Some(PlayerId(3)));
        assert_eq!(round.pairings.len(), 4);
        assert!(round.pairings.contains(&Pairing::Bye {
            player: PlayerId(4),
            kind: ByeKind::Half
        }));
        assert!(round.pairings.contains(&Pairing::Bye {
            player: PlayerId(5),
            kind: ByeKind::Zero
        }));
        assert!(violations.is_empty(), "{violations:?}");

        t.apply(&Event::RoundPaired {
            round: 1,
            pairings: round.pairings,
            matches: vec![],
            method: PairingMethod::Manual,
        })
        .unwrap();
        let input = PairingInput::for_next_round(&t);
        let (_, violations) =
            manual_round(&input, &[(PlayerId(2), PlayerId(1))], Some(PlayerId(3)));
        assert!(violations.contains(&Violation::RepeatPairing(PlayerId(2), PlayerId(1))));
        assert!(violations.contains(&Violation::RepeatPairingBye(PlayerId(3))));
    }
}
