//! Points per round and running scores.

use crate::ids::{PlayerId, RoundNo};
use crate::model::{ByeKind, Pairing, Player, PointSystem, Round, SideResult, Tournament};
use crate::points::Points;
use std::collections::{BTreeMap, HashSet};

pub fn side_points(result: SideResult, ps: &PointSystem) -> Points {
    match result {
        SideResult::Win | SideResult::UnratedWin => ps.win,
        SideResult::Draw | SideResult::UnratedDraw => ps.draw,
        SideResult::Loss | SideResult::UnratedLoss => ps.loss,
        SideResult::ForfeitWin => ps.forfeit_win,
        SideResult::ForfeitLoss => ps.forfeit_loss,
    }
}

pub fn bye_points(kind: ByeKind, ps: &PointSystem) -> Points {
    match kind {
        ByeKind::PairingAllocated => ps.pairing_bye,
        ByeKind::Half => ps.half_bye,
        ByeKind::Full => ps.full_bye,
        ByeKind::Zero => ps.zero_bye,
    }
}

/// What a player got in one round, as far as scoring is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundOutcome {
    Game {
        result: Option<SideResult>,
    },
    Bye(ByeKind),
    /// Credited for rounds before a late entry joined.
    LateEntry(ByeKind),
    /// Not paired and no bye recorded (withdrawn or absent).
    Absent,
}

pub fn outcome(player: &Player, round: &Round) -> RoundOutcome {
    outcome_of(player, round.number, round.entry_for(player.id))
}

/// Like [`outcome`], for callers that already looked up the player's entry
/// (see [`Round::index`]).
pub fn outcome_of(player: &Player, round: RoundNo, entry: Option<&Pairing>) -> RoundOutcome {
    match entry {
        Some(game @ Pairing::Game { result, .. }) => {
            let (color, _) = game.side_of(player.id).expect("game involves player");
            RoundOutcome::Game {
                result: result.map(|r| r.for_color(color)),
            }
        }
        Some(Pairing::Bye { kind, .. }) => RoundOutcome::Bye(*kind),
        None if round < player.joined_round => RoundOutcome::LateEntry(player.late_entry_bye),
        None => RoundOutcome::Absent,
    }
}

pub fn outcome_points(o: RoundOutcome, ps: &PointSystem) -> Points {
    match o {
        RoundOutcome::Game { result: Some(r) } => side_points(r, ps),
        RoundOutcome::Game { result: None } | RoundOutcome::Absent => Points::ZERO,
        RoundOutcome::Bye(k) | RoundOutcome::LateEntry(k) => bye_points(k, ps),
    }
}

/// Score of `player` after `after_round` rounds (pending games count 0).
pub fn score_after(t: &Tournament, player: PlayerId, after_round: RoundNo) -> Points {
    let Some(p) = t.player(player) else {
        return Points::ZERO;
    };
    t.rounds
        .iter()
        .take(usize::from(after_round))
        .map(|r| outcome_points(outcome(p, r), &t.settings.points))
        .sum()
}

/// Adds what every player scored in `round` to `scores`, in one pass.
fn add_round(t: &Tournament, round: &Round, scores: &mut BTreeMap<PlayerId, Points>) {
    let ps = &t.settings.points;
    let mut add = |id: PlayerId, pts: Points| *scores.entry(id).or_default() += pts;
    let mut seen = HashSet::new();
    for p in round.all_pairings() {
        match *p {
            Pairing::Game {
                white,
                black,
                result,
                ..
            } => {
                if let Some(r) = result {
                    add(white, side_points(r.white, ps));
                    add(black, side_points(r.black, ps));
                }
                seen.extend([white, black]);
            }
            Pairing::Bye { player, kind } => {
                add(player, bye_points(kind, ps));
                seen.insert(player);
            }
        }
    }
    for p in &t.players {
        if round.number < p.joined_round && !seen.contains(&p.id) {
            add(p.id, bye_points(p.late_entry_bye, ps));
        }
    }
}

/// Scores of every player after `after_round` rounds.
pub fn scores_after(t: &Tournament, after_round: RoundNo) -> BTreeMap<PlayerId, Points> {
    let mut scores: BTreeMap<PlayerId, Points> =
        t.players.iter().map(|p| (p.id, Points::ZERO)).collect();
    for r in t.rounds.iter().take(usize::from(after_round)) {
        add_round(t, r, &mut scores);
    }
    scores
}

/// Scores after each paired round: element `i` is the standing after round `i + 1`.
pub fn cumulative_scores(t: &Tournament) -> Vec<BTreeMap<PlayerId, Points>> {
    let mut scores: BTreeMap<PlayerId, Points> =
        t.players.iter().map(|p| (p.id, Points::ZERO)).collect();
    t.rounds
        .iter()
        .map(|r| {
            add_round(t, r, &mut scores);
            scores.clone()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;
    use crate::model::*;
    use uuid::Uuid;

    #[test]
    fn scores_games_byes_and_late_entries() {
        let mut t = Tournament::create(&Event::TournamentCreated {
            id: Uuid::nil(),
            info: TournamentInfo::default(),
            settings: Settings::default(),
            kind: TournamentKind::default(),
        })
        .unwrap();
        for i in 1..=3 {
            t.apply(&Event::PlayerAdded {
                player: Player::new(PlayerId(i), format!("P{i}")),
            })
            .unwrap();
        }
        t.apply(&Event::RoundPaired {
            round: 1,
            pairings: vec![
                Pairing::Game {
                    board: 1,
                    white: PlayerId(1),
                    black: PlayerId(2),
                    result: Some(GameResult::WHITE_WINS),
                },
                Pairing::Bye {
                    player: PlayerId(3),
                    kind: ByeKind::PairingAllocated,
                },
            ],
            matches: vec![],
            method: PairingMethod::Manual,
        })
        .unwrap();
        let mut late = Player::new(PlayerId(4), "Late");
        late.joined_round = 2;
        late.late_entry_bye = ByeKind::Half;
        t.apply(&Event::PlayerAdded { player: late }).unwrap();

        let s = scores_after(&t, 1);
        assert_eq!(cumulative_scores(&t), vec![s.clone()]);
        for p in &t.players {
            assert_eq!(
                s[&p.id],
                score_after(&t, p.id, 1),
                "single-pass matches per-player"
            );
        }
        assert_eq!(s[&PlayerId(1)], Points::ONE);
        assert_eq!(s[&PlayerId(2)], Points::ZERO);
        assert_eq!(s[&PlayerId(3)], Points::ONE);
        assert_eq!(s[&PlayerId(4)], Points::HALF);
    }
}
