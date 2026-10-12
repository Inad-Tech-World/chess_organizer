use super::*;
use co_core::*;

const SAMPLE: &str = include_str!("../../../../testdata/trf/sample.trf");

#[test]
fn reads_sample() {
    let import = read(SAMPLE).unwrap();
    assert!(import.warnings.is_empty(), "{:?}", import.warnings);
    let t = import.tournament;

    assert_eq!(t.info.name, "Kampala Club Championship 2026");
    assert_eq!(t.info.deputy_arbiters, vec!["John Deputy"]);
    assert_eq!(t.info.round_dates.len(), 3);
    assert_eq!(t.settings.rounds, 5);
    assert_eq!(
        t.kind,
        TournamentKind::Swiss {
            system: SwissSystem::Dutch
        }
    );
    assert_eq!(t.settings.tiebreaks.len(), 4);
    assert_eq!(t.settings.tiebreaks[0].label(), "BH/C1");

    assert_eq!(t.players.len(), 7);
    let p2 = t.player(PlayerId(2)).unwrap();
    assert_eq!(p2.title, Some(Title::WFM));
    assert_eq!(p2.sex, Some(Sex::Female));
    assert_eq!(p2.fide_id, Some(1000022));
    assert_eq!(t.player(PlayerId(6)).unwrap().ratings.fide_standard, None);

    assert_eq!(t.paired_rounds(), 3);
    let r1 = t.round(1).unwrap();
    assert_eq!(
        r1.pairings.iter().filter(|p| p.board().is_some()).count(),
        3
    );
    assert_eq!(
        r1.entry_for(PlayerId(7)),
        Some(&Pairing::Bye {
            player: PlayerId(7),
            kind: ByeKind::PairingAllocated
        })
    );
    let r3 = t.round(3).unwrap();
    let forfeit = r3.entry_for(PlayerId(7)).unwrap();
    assert!(matches!(
        forfeit,
        Pairing::Game { white: PlayerId(2), black: PlayerId(7), result: Some(r), .. }
            if *r == GameResult::WHITE_FORFEIT_WIN
    ));

    let scores = scoring::scores_after(&t, 3);
    assert_eq!(scores[&PlayerId(1)], Points(30));
    assert_eq!(scores[&PlayerId(6)], Points(15));
    assert_eq!(scores[&PlayerId(5)], Points(10));

    assert_eq!(
        t.player(PlayerId(3)).unwrap().byes.get(&4),
        Some(&ByeKind::Half)
    );
    assert_eq!(
        t.settings.forbidden_groups[0].members,
        vec![PlayerId(1), PlayerId(2)]
    );
}

#[test]
fn boards_follow_score_order() {
    let t = read(SAMPLE).unwrap().tournament;
    // Round 3: player 1 (2 points) is on board 1.
    let r3 = t.round(3).unwrap();
    assert!(r3.game_on_board(1).unwrap().involves(PlayerId(1)));
}

#[test]
fn write_round_trips_byte_for_byte() {
    let t = read(SAMPLE).unwrap().tournament;
    assert_eq!(write(&t), SAMPLE);
}

#[test]
fn reads_legacy_trf16_and_bbp_extensions() {
    let text = "\
012 Old event
XXR 7
XXC black1
BBW  3.0
BBD  1.0
001    1 m  g Alpha, A                          2500 GER     4600001 1980/01/01  3.0    1     2 b 1
001    2 w wm Beta, B                           2300 FRA     4600002 1990/01/01  0.0    2     1 w 0
";
    let import = read(text).unwrap();
    let t = import.tournament;
    assert!(import.warnings.is_empty(), "{:?}", import.warnings);
    assert_eq!(t.settings.rounds, 7);
    assert_eq!(t.settings.initial_color, Color::Black);
    assert_eq!(t.settings.points.win, Points(30));
    assert_eq!(t.player(PlayerId(1)).unwrap().title, Some(Title::GM));
    assert_eq!(t.player(PlayerId(2)).unwrap().title, Some(Title::WIM));
    assert_eq!(scoring::score_after(&t, PlayerId(1), 1), Points(30));
}

#[test]
fn point_system_record_round_trips() {
    let mut t = read(SAMPLE).unwrap().tournament;
    t.settings.points.win = Points(30);
    t.settings.points.forfeit_win = Points(30);
    t.settings.points.full_bye = Points(30);
    t.settings.points.pairing_bye = Points(30);
    t.settings.points.draw = Points(10);
    t.settings.points.half_bye = Points(10);
    let text = write(&t);
    assert!(
        text.contains("162  W 3.0    D 1.0    L 0.0    A 0.0    P 3.0"),
        "{text}"
    );
    let back = read(&text).unwrap();
    assert_eq!(back.tournament.settings.points, t.settings.points);
}

fn player_line(rank: u32, blocks: &[(&str, char, char)]) -> String {
    let mut b = columns::LineBuf::new("001");
    b.put_right(5, 8, &rank.to_string());
    b.put_left(15, 47, &format!("Player {rank}"));
    for (i, (opp, color, result)) in blocks.iter().enumerate() {
        let c = round_col(i as u16 + 1);
        b.put_right(c, c + 3, opp);
        b.put_char(c + 5, *color);
        b.put_char(c + 7, *result);
    }
    b.finish()
}

#[test]
fn reports_inconsistent_opponents() {
    let text = [
        player_line(1, &[("2", 'w', '1')]),
        player_line(2, &[("3", 'b', '0')]),
        player_line(3, &[("2", 'w', '1')]),
    ]
    .join("\n");
    let import = read(&text).unwrap();
    assert!(!import.warnings.is_empty());
    assert_eq!(import.tournament.round(1).unwrap().pairings.len(), 1);
}

#[test]
fn pending_results_and_blank_blocks() {
    let text = [
        player_line(1, &[("2", 'w', ' ')]),
        player_line(2, &[("1", 'b', ' ')]),
        player_line(3, &[("", ' ', ' ')]),
    ]
    .join("\n");
    let t = read(&text).unwrap().tournament;
    let r1 = t.round(1).unwrap();
    assert!(matches!(
        r1.game_on_board(1),
        Some(Pairing::Game { result: None, .. })
    ));
    // Player 3's line ends before round 1: not paired at all.
    assert_eq!(r1.entry_for(PlayerId(3)), None);
}
