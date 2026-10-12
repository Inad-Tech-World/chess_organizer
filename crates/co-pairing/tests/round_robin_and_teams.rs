//! Whole events paired by the round-robin and team engines.

use co_core::pairing_view::PairingInput;
use co_core::{
    Event, GameResult, Pairing, PairingMethod, Player, PlayerId, Settings, SwissSystem, Team,
    TeamId, Tournament, TournamentInfo, TournamentKind,
};
use co_pairing::engine_for;
use std::collections::{HashMap, HashSet};

fn create(kind: TournamentKind, rounds: u16, boards: u8) -> Tournament {
    let mut settings = Settings {
        rounds,
        ..Settings::default()
    };
    settings.team.boards = boards;
    Tournament::create(&Event::TournamentCreated {
        id: co_core::Uuid::nil(),
        info: TournamentInfo::default(),
        settings,
        kind,
    })
    .unwrap()
}

fn add_players(t: &mut Tournament, n: u32) {
    for i in 1..=n {
        let mut p = Player::new(PlayerId(i), format!("Player {i}"));
        p.ratings.fide_standard = Some(2400 - i as u16 * 10);
        t.apply(&Event::PlayerAdded { player: p }).unwrap();
    }
}

/// A deterministic result for every game: higher id wins, every third game drawn.
fn result_for(white: PlayerId, black: PlayerId, k: usize) -> GameResult {
    if k.is_multiple_of(3) {
        GameResult::DRAW
    } else if white.0 < black.0 {
        GameResult::WHITE_WINS
    } else {
        GameResult::BLACK_WINS
    }
}

fn pair_and_play(t: &mut Tournament) {
    let system = t.kind.pairing_system();
    let input = PairingInput::for_next_round(t);
    let round = engine_for(system).pair(&input).unwrap();
    let r = input.round;
    t.apply(&Event::RoundPaired {
        round: r,
        pairings: round.pairings.clone(),
        matches: round.matches.clone(),
        method: PairingMethod::Engine { system },
    })
    .unwrap();
    let mut k = 0;
    for p in &round.pairings {
        if let Pairing::Game {
            board,
            white,
            black,
            ..
        } = *p
        {
            k += 1;
            t.apply(&Event::ResultEntered {
                round: r,
                board,
                result: Some(result_for(white, black, k)),
            })
            .unwrap();
        }
    }
    for m in &round.matches {
        for g in &m.games {
            if let Pairing::Game {
                board,
                white,
                black,
                ..
            } = *g
            {
                k += 1;
                t.apply(&Event::TeamGameResultEntered {
                    round: r,
                    match_number: m.number,
                    board,
                    result: Some(result_for(white, black, k)),
                })
                .unwrap();
            }
        }
    }
}

#[test]
fn single_round_robin_of_seven() {
    let mut t = create(TournamentKind::RoundRobin { double: false }, 7, 4);
    add_players(&mut t, 7);
    for _ in 0..7 {
        pair_and_play(&mut t);
    }
    let mut met = HashSet::new();
    let mut sat_out = HashMap::new();
    for r in &t.rounds {
        for p in &r.pairings {
            match *p {
                Pairing::Game { white, black, .. } => {
                    assert!(met.insert((white.0.min(black.0), white.0.max(black.0))))
                }
                Pairing::Bye { player, .. } => *sat_out.entry(player).or_insert(0) += 1,
            }
        }
    }
    assert_eq!(met.len(), 21, "everyone met everyone");
    assert!(
        sat_out.values().all(|&n| n == 1),
        "each player sat out once"
    );
    assert!(
        engine_for(t.kind.pairing_system())
            .pair(&PairingInput::for_next_round(&t))
            .is_err(),
        "no 8th round"
    );
}

#[test]
fn double_round_robin_of_four() {
    let mut t = create(TournamentKind::RoundRobin { double: true }, 6, 4);
    add_players(&mut t, 4);
    for _ in 0..6 {
        pair_and_play(&mut t);
    }
    let mut colours: HashMap<(u32, u32), Vec<u32>> = HashMap::new();
    for r in &t.rounds {
        for p in &r.pairings {
            if let Pairing::Game { white, black, .. } = *p {
                colours
                    .entry((white.0.min(black.0), white.0.max(black.0)))
                    .or_default()
                    .push(white.0);
            }
        }
    }
    assert_eq!(colours.len(), 6);
    for (pair, whites) in colours {
        assert_eq!(whites.len(), 2, "{pair:?} meet twice");
        assert_ne!(whites[0], whites[1], "{pair:?} swap colours");
    }
}

fn add_teams(t: &mut Tournament, teams: u32, boards: u32) {
    add_players(t, teams * boards);
    for i in 0..teams {
        let team = Team {
            id: TeamId(i + 1),
            name: format!("Team {}", i + 1),
            short_name: None,
            federation: None,
            members: (1..=boards).map(|b| PlayerId(i * boards + b)).collect(),
            captain: None,
        };
        t.apply(&Event::TeamAdded { team }).unwrap();
    }
}

#[test]
fn team_round_robin_of_five_teams() {
    let mut t = create(TournamentKind::TeamRoundRobin { double: false }, 5, 4);
    add_teams(&mut t, 5, 4);
    for _ in 0..5 {
        pair_and_play(&mut t);
    }
    let mut met = HashSet::new();
    for r in &t.rounds {
        assert_eq!(r.matches.len(), 2, "one team sits out");
        for m in &r.matches {
            let a = m.away.unwrap();
            assert!(met.insert((m.home.0.min(a.0), m.home.0.max(a.0))));
            assert_eq!(m.games.len(), 4);
        }
    }
    assert_eq!(met.len(), 10);
    let s = co_tiebreak::team_standings(&t, 5).unwrap();
    assert_eq!(s.rows.len(), 5);
}

#[test]
fn team_swiss_has_no_repeats_and_one_bye_each() {
    for primary in [
        co_core::TeamScore::MatchPoints,
        co_core::TeamScore::GamePoints,
    ] {
        let mut t = create(
            TournamentKind::TeamSwiss {
                system: SwissSystem::Dutch,
            },
            5,
            4,
        );
        t.settings.team.primary_score = primary;
        add_teams(&mut t, 9, 4);
        for _ in 0..5 {
            pair_and_play(&mut t);
        }
        let mut met = HashSet::new();
        let mut byes = HashSet::new();
        for r in &t.rounds {
            assert_eq!(r.matches.len(), 5);
            for m in &r.matches {
                match m.away {
                    Some(a) => assert!(
                        met.insert((m.home.0.min(a.0), m.home.0.max(a.0))),
                        "{primary:?} repeat"
                    ),
                    None => assert!(byes.insert(m.home), "{primary:?} second bye"),
                }
            }
        }
        assert_eq!(byes.len(), 5);
    }
}
