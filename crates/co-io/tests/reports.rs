use co_i18n::Translator;
use co_io::render::{self, Format};
use co_io::report::{ListKind, build};

const SAMPLE: &str = include_str!("../../../testdata/trf/sample.trf");

fn sample() -> co_core::Tournament {
    co_io::trf::read(SAMPLE).unwrap().tournament
}

#[test]
fn standings_html() {
    let t = sample();
    let report = build(ListKind::Standings { round: 3 }, &t, &Translator::new("en")).unwrap();
    insta::assert_snapshot!(render::html::render(&report).unwrap());
}

#[test]
fn pairings_html() {
    let t = sample();
    let report = build(ListKind::Pairings { round: 3 }, &t, &Translator::new("en")).unwrap();
    insta::assert_snapshot!(render::html::render(&report).unwrap());
}

#[test]
fn starting_rank_csv() {
    let t = sample();
    let report = build(ListKind::StartingRank, &t, &Translator::new("en")).unwrap();
    insta::assert_snapshot!(
        String::from_utf8(render::render(&report, Format::Csv).unwrap()).unwrap()
    );
}

#[test]
fn rtl_languages_mark_the_document() {
    let t = sample();
    let report = build(ListKind::StartingRank, &t, &Translator::new("ar")).unwrap();
    assert!(report.rtl);
    assert!(
        render::html::render(&report)
            .unwrap()
            .contains(r#"dir="rtl""#)
    );
}

#[test]
fn crosstable_text() {
    let t = sample();
    let report = build(
        ListKind::Crosstable { round: 3 },
        &t,
        &Translator::new("en"),
    )
    .unwrap();
    insta::assert_snapshot!(
        String::from_utf8(render::render(&report, Format::Csv).unwrap()).unwrap()
    );
}

#[test]
fn every_individual_list_builds_and_renders() {
    let t = sample();
    let tr = Translator::new("en");
    for kind in [
        ListKind::StartingRank,
        ListKind::Alphabetical,
        ListKind::Pairings { round: 3 },
        ListKind::Standings { round: 3 },
        ListKind::Crosstable { round: 3 },
        ListKind::Scorecards,
        ListKind::BoardCards { round: 3 },
        ListKind::Prizes,
        ListKind::RatingReport,
        ListKind::TeamRosters,
    ] {
        let report = build(kind, &t, &tr).unwrap_or_else(|e| panic!("{kind:?}: {e}"));
        for format in [Format::Html, Format::Pdf, Format::Csv, Format::Xlsx] {
            let bytes = render::render(&report, format)
                .unwrap_or_else(|e| panic!("{kind:?} {format:?}: {e}"));
            assert!(!bytes.is_empty());
        }
    }
}

#[test]
fn unpaired_round_is_an_error() {
    let t = sample();
    assert!(
        build(
            ListKind::BoardCards { round: 9 },
            &t,
            &Translator::new("en")
        )
        .is_err()
    );
}
