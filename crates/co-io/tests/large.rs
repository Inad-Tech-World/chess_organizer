//! Scale check for the 2,000-player events the program must handle.
//! Run with `cargo test --release -p co-io --test large -- --ignored --nocapture`.

use co_core::pairing_view::PairingInput;
use co_core::*;
use std::time::Instant;

fn big_tournament(players: u32, rounds: u16) -> Tournament {
    let mut t = Tournament::create(&Event::TournamentCreated {
        id: Uuid::nil(),
        info: TournamentInfo::default(),
        settings: Settings {
            rounds,
            ..Settings::default()
        },
        kind: TournamentKind::default(),
    })
    .unwrap();
    for i in 1..=players {
        let mut p = Player::new(PlayerId(i), format!("Player {i:04}, Test"));
        p.ratings.fide_standard = Some(2800 - (i % 1200) as u16);
        t.apply(&Event::PlayerAdded { player: p }).unwrap();
    }
    for r in 1..=rounds {
        // Rotating pairs: deterministic and never repeats for these sizes.
        let shift = u32::from(r);
        let pairings = (0..players / 2)
            .map(|b| {
                let a = 1 + b;
                let c = 1 + (b + shift * 7) % (players / 2) + players / 2;
                Pairing::Game {
                    board: b as u16 + 1,
                    white: PlayerId(if r % 2 == 0 { a } else { c }),
                    black: PlayerId(if r % 2 == 0 { c } else { a }),
                    result: Some(match b % 3 {
                        0 => GameResult::WHITE_WINS,
                        1 => GameResult::DRAW,
                        _ => GameResult::BLACK_WINS,
                    }),
                }
            })
            .collect();
        t.apply(&Event::RoundPaired {
            round: r,
            pairings,
            matches: vec![],
            method: PairingMethod::Manual,
        })
        .unwrap();
    }
    t
}

#[test]
#[ignore]
fn two_thousand_players_eleven_rounds() {
    let t = big_tournament(2000, 11);
    let time = |label: &str, f: &mut dyn FnMut()| {
        let start = Instant::now();
        f();
        println!(
            "{label:<28} {:>8.1} ms",
            start.elapsed().as_secs_f64() * 1000.0
        );
    };
    time("cumulative scores", &mut || {
        co_core::scoring::cumulative_scores(&t);
    });
    time("pairing input", &mut || {
        PairingInput::for_next_round(&t);
    });
    time("standings", &mut || {
        co_tiebreak::standings(&t, 11).unwrap();
    });
    time("TRF write", &mut || {
        co_io::trf::write(&t);
    });
    let text = co_io::trf::write(&t);
    time("TRF read", &mut || {
        co_io::trf::read(&text).unwrap();
    });
    time("JSON (UI view)", &mut || {
        serde_json::to_string(&t).unwrap();
    });
}
