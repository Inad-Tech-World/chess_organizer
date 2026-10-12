use co_core::Error as CoreError;
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
fn planned_lists_report_their_phase() {
    let t = sample();
    let err = build(
        ListKind::Crosstable { round: 3 },
        &t,
        &Translator::new("en"),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        co_io::Error::Core(CoreError::NotYetImplemented { phase: 1, .. })
    ));
}
