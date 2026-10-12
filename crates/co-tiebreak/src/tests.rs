//! Values below are worked out by hand from testdata/trf/sample.trf.
//!
//! Scores: P1 3, P2 2, P3 1, P4 2, P5 1, P6 1½, P7 1.
//! Unplayed rounds: P2 R3 forfeit win (cat. 2), P3 R2 PAB (cat. 1),
//! P5 R3 half-point bye in the last round (cat. 5), P7 R1 PAB (cat. 1),
//! P7 R3 forfeit loss (cat. 4).
//! (These are unit tests, but co-io is used through a path dev-dependency to read
//! the sample; its own tie-break use is exercised by its snapshot tests.)

use super::*;
use co_core::{TiebreakModifiers, Tournament};

fn sample() -> Tournament {
    co_io::trf::read(include_str!("../../../testdata/trf/sample.trf"))
        .unwrap()
        .tournament
}

fn tb(kind: TiebreakKind, cut: u8) -> TiebreakSpec {
    TiebreakSpec {
        kind,
        modifiers: TiebreakModifiers {
            cut_lowest: cut,
            ..TiebreakModifiers::default()
        },
    }
}

fn value(t: &Tournament, spec: TiebreakSpec, p: u32) -> i64 {
    let ctx = TiebreakContext::new(t, 3);
    match compute(&spec, &ctx, PlayerId(p)).unwrap() {
        TbValue::Value(v) => v,
        TbValue::Unavailable => panic!("unavailable"),
    }
}

#[test]
fn buchholz_with_unplayed_rounds() {
    let t = sample();
    // P1: opponents 4 (2) + 7 (1) + 6 (1½) = 4½
    assert_eq!(value(&t, tb(TiebreakKind::Buchholz, 0), 1), 450);
    // P2: 5 (1) + 6 (1½) + forfeit win: dummy = min(own 2, opponent 7's 1) = 1
    assert_eq!(value(&t, tb(TiebreakKind::Buchholz, 0), 2), 350);
    // P3: 6 (1½) + PAB: dummy = min(own 1, ½ × 3 rounds) = 1 + 4 (2) = 4½
    assert_eq!(value(&t, tb(TiebreakKind::Buchholz, 0), 3), 450);
    // P5: 2 (2) + 4 (2) + half bye: dummy 1 → 5; Cut-1 removes the voluntary unplayed round first
    assert_eq!(value(&t, tb(TiebreakKind::Buchholz, 0), 5), 500);
    assert_eq!(value(&t, tb(TiebreakKind::Buchholz, 1), 5), 400);
    // P1 Cut-1 removes the lowest (1)
    assert_eq!(value(&t, tb(TiebreakKind::Buchholz, 1), 1), 350);
}

#[test]
fn sonneborn_berger_and_counts() {
    let t = sample();
    // P1: 2×1 + 1×1 + 1½×1
    assert_eq!(value(&t, tb(TiebreakKind::SonnebornBerger, 0), 1), 450);
    // P2: 1×½ + 1½×½ + dummy 1 × 1 (forfeit win)
    assert_eq!(value(&t, tb(TiebreakKind::SonnebornBerger, 0), 2), 225);
    assert_eq!(value(&t, tb(TiebreakKind::Wins, 0), 2), 100); // the forfeit win counts
    assert_eq!(value(&t, tb(TiebreakKind::WonGames, 0), 2), 0);
    assert_eq!(value(&t, tb(TiebreakKind::GamesWithBlack, 0), 4), 200);
    assert_eq!(value(&t, tb(TiebreakKind::RoundsElectedToPlay, 0), 7), 200); // forfeit loss is voluntary
    // PS P1: 1 + 2 + 3
    assert_eq!(value(&t, tb(TiebreakKind::ProgressiveScore, 0), 1), 600);
    assert_eq!(value(&t, tb(TiebreakKind::ProgressiveScore, 1), 1), 500);
}

#[test]
fn rating_based() {
    let t = sample();
    // P1 played 4 (2040), 7 (unrated), 6 (unrated): ARO over rated opponents = 2040
    assert_eq!(
        value(&t, tb(TiebreakKind::AverageRatingOfOpponents, 0), 1),
        204_000
    );
    // TPR P1: 2040 + dp(1.0) = 2840
    assert_eq!(
        value(&t, tb(TiebreakKind::TournamentPerformanceRating, 0), 1),
        284_000
    );
}

#[test]
fn standings_use_tiebreaks_and_direct_encounter() {
    let t = sample();
    let s = standings(&t, 3).unwrap();
    let order: Vec<u32> = s.rows.iter().map(|r| r.player.0).collect();
    // P2 and P4 tie on 2: BH/C1 P4 = 4 (3, 1 after cutting 1) vs P2 = 2½.
    assert_eq!(&order[..3], &[1, 4, 2]);
}

#[test]
fn direct_encounter_requires_everyone_to_have_met() {
    let t = sample();
    let ctx = TiebreakContext::new(&t, 3);
    // P2 and P4 never met: nobody is certain to lead.
    let de = direct_encounter(&ctx, &[PlayerId(2), PlayerId(4)], false);
    assert_eq!(de[&PlayerId(2)], de[&PlayerId(4)]);
    // P3 and P6 met (P6 won): DE separates them.
    let de = direct_encounter(&ctx, &[PlayerId(3), PlayerId(6)], false);
    assert!(de[&PlayerId(6)] > de[&PlayerId(3)]);
}
