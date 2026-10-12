//! Whole tournaments paired by every Swiss engine, checked against the
//! absolute criteria of C.04.1.

use co_core::pairing_view::PairingInput;
use co_core::{
    ByeKind, Color, Event, GameResult, Pairing, PairingMethod, Player, PlayerId, Settings,
    SwissSystem, Tournament, TournamentInfo, TournamentKind,
};
use co_pairing::engine_for;
use std::collections::{HashMap, HashSet};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn run(system: SwissSystem, players: u32, rounds: u16, seed: u64) -> Tournament {
    let mut t = Tournament::create(&Event::TournamentCreated {
        id: co_core::Uuid::nil(),
        info: TournamentInfo::default(),
        settings: Settings {
            rounds,
            ..Settings::default()
        },
        kind: TournamentKind::Swiss { system },
    })
    .unwrap();
    for i in 1..=players {
        let mut p = Player::new(PlayerId(i), format!("P{i}"));
        p.ratings.fide_standard = Some(2500 - (i as u16) * 7);
        t.apply(&Event::PlayerAdded { player: p }).unwrap();
    }
    let mut rng = Rng(seed);
    for _ in 0..rounds {
        // Occasionally someone asks for a half-point bye.
        let r = t.paired_rounds() + 1;
        if rng.next().is_multiple_of(4) {
            let id = PlayerId((rng.next() % u64::from(players)) as u32 + 1);
            t.apply(&Event::ByeSet {
                player: id,
                round: r,
                kind: Some(ByeKind::Half),
            })
            .unwrap();
        }
        let sys = t.kind.pairing_system();
        let round = engine_for(sys)
            .pair(&PairingInput::for_next_round(&t))
            .unwrap_or_else(|e| panic!("{system:?} seed {seed} round {r}: {e}"));
        t.apply(&Event::RoundPaired {
            round: r,
            pairings: round.pairings.clone(),
            matches: vec![],
            method: PairingMethod::Engine { system: sys },
        })
        .unwrap();
        for p in &round.pairings {
            if let Pairing::Game {
                board,
                white,
                black,
                ..
            } = *p
            {
                let diff = i64::from(black.0) - i64::from(white.0);
                let x = rng.next() % 100;
                let result = if x < 3 {
                    GameResult::WHITE_FORFEIT_WIN
                } else if (x as i64) < 45 + diff {
                    GameResult::WHITE_WINS
                } else if x < 75 {
                    GameResult::DRAW
                } else {
                    GameResult::BLACK_WINS
                };
                t.apply(&Event::ResultEntered {
                    round: r,
                    board,
                    result: Some(result),
                })
                .unwrap();
            }
        }
    }
    t
}

fn check(t: &Tournament, label: &str) {
    let mut met: HashSet<(u32, u32)> = HashSet::new();
    let mut pab: HashSet<PlayerId> = HashSet::new();
    let mut colours: HashMap<PlayerId, Vec<Color>> = HashMap::new();
    for r in &t.rounds {
        let mut seen = HashSet::new();
        for p in &r.pairings {
            for id in p.players() {
                assert!(seen.insert(id), "{label}: {id} twice in round {}", r.number);
            }
            match *p {
                Pairing::Game {
                    white,
                    black,
                    result,
                    ..
                } => {
                    let key = (white.0.min(black.0), white.0.max(black.0));
                    assert!(
                        !met.contains(&key),
                        "{label}: rematch {key:?} in round {}",
                        r.number
                    );
                    if result.is_some_and(|r| r.is_played()) {
                        met.insert(key);
                        colours.entry(white).or_default().push(Color::White);
                        colours.entry(black).or_default().push(Color::Black);
                    }
                }
                Pairing::Bye {
                    player,
                    kind: ByeKind::PairingAllocated,
                } => {
                    assert!(pab.insert(player), "{label}: second PAB for {player}");
                }
                Pairing::Bye { .. } => {}
            }
        }
        assert_eq!(
            seen.len(),
            t.players.len(),
            "{label}: everyone accounted for in round {}",
            r.number
        );
    }
    // Colour absolute criteria (C.04.1 f/g), except in the last round.
    for (id, cs) in colours {
        let upto = cs.len().saturating_sub(1);
        for k in 2..upto {
            assert!(
                !(cs[k] == cs[k - 1] && cs[k] == cs[k - 2]),
                "{label}: {id} three in a row {cs:?}"
            );
        }
    }
}

#[test]
fn every_swiss_system_respects_the_absolute_criteria() {
    for system in [
        SwissSystem::Dutch,
        SwissSystem::Burstein,
        SwissSystem::Dubov,
        SwissSystem::Lim,
        SwissSystem::Monrad,
    ] {
        for (players, rounds, seed) in [
            (10, 5, 1),
            (17, 7, 2),
            (24, 9, 3),
            (31, 9, 4),
            (8, 5, 5),
            (40, 11, 6),
        ] {
            let t = run(system, players, rounds, seed);
            check(&t, &format!("{system:?} {players}p seed {seed}"));
        }
    }
}

#[test]
fn round_one_pairs_top_half_against_bottom_half() {
    for system in [SwissSystem::Dutch, SwissSystem::Dubov, SwissSystem::Lim] {
        let t = run(system, 8, 1, 9);
        let mut games: Vec<(u32, u32)> = t.rounds[0]
            .pairings
            .iter()
            .filter_map(|p| match *p {
                Pairing::Game { white, black, .. } => {
                    Some((white.0.min(black.0), white.0.max(black.0)))
                }
                _ => None,
            })
            .collect();
        games.sort();
        assert_eq!(games, [(1, 5), (2, 6), (3, 7), (4, 8)], "{system:?}");
    }
}
