//! Sample data for trying the program: a fictional 25-player rapid open with
//! four rounds played (the last only half entered), and a sample player
//! list for the player database.
//!
//! Every name, rating and ID here is invented. FIDE IDs use the 99xxxxxx
//! range so nothing collides with a real player. Rounds are paired by a
//! simple score-group method that avoids repeats and forbidden pairs. That
//! is demo data only, not the FIDE Dutch system (Phase 1).

use co_core::chrono::NaiveDate;
use co_core::pairing_view::{PairingInput, PairingPlayer};
use co_core::*;
use std::cmp::Reverse;

/// One fictional person, used both in the tournament and the player list.
#[derive(Debug, Clone, Copy)]
pub struct SamplePerson {
    pub name: &'static str,
    pub federation: &'static str,
    pub sex: Sex,
    pub title: Option<Title>,
    pub rating: Option<u16>,
    pub birth_year: u16,
    pub club: Option<&'static str>,
}

const fn p(
    name: &'static str,
    federation: &'static str,
    sex: Sex,
    title: Option<Title>,
    rating: Option<u16>,
    birth_year: u16,
    club: Option<&'static str>,
) -> SamplePerson {
    SamplePerson {
        name,
        federation,
        sex,
        title,
        rating,
        birth_year,
        club,
    }
}

use Sex::{Female as F, Male as M};

const KCC: Option<&str> = Some("Kampala Chess Club");

/// The 24 players registered before round 1.
pub const ENTRANTS: [SamplePerson; 24] = [
    p(
        "Okello, David",
        "UGA",
        M,
        Some(Title::IM),
        Some(2318),
        1992,
        KCC,
    ),
    p(
        "Mwangi, Peter",
        "KEN",
        M,
        Some(Title::FM),
        Some(2246),
        1988,
        Some("Nairobi Knights"),
    ),
    p(
        "Okonkwo, Chidi",
        "NGR",
        M,
        Some(Title::FM),
        Some(2190),
        1985,
        None,
    ),
    p(
        "Nakato, Grace",
        "UGA",
        F,
        Some(Title::WFM),
        Some(2105),
        2001,
        KCC,
    ),
    p(
        "Banda, Joseph",
        "ZAM",
        M,
        Some(Title::CM),
        Some(2088),
        1995,
        None,
    ),
    p(
        "Dlamini, Thandi",
        "RSA",
        F,
        Some(Title::WIM),
        Some(2040),
        1998,
        None,
    ),
    p("Mugisha, Brian", "RWA", M, None, Some(2012), 1999, None),
    p("Ndlovu, Sipho", "ZIM", M, None, Some(1990), 1993, None),
    p("Mensah, Kofi", "GHA", M, None, Some(1985), 1979, None),
    p(
        "Achieng, Mary",
        "KEN",
        F,
        Some(Title::WCM),
        Some(1964),
        2006,
        Some("Nairobi Knights"),
    ),
    p(
        "Ssempala, Joseph",
        "UGA",
        M,
        None,
        Some(1942),
        1990,
        Some("Mengo Chess Academy"),
    ),
    p("Lubega, Ivan", "UGA", M, None, Some(1912), 1983, KCC),
    p(
        "Tumusiime, Alex",
        "UGA",
        M,
        None,
        Some(1880),
        2008,
        Some("Mengo Chess Academy"),
    ),
    p("Kimaro, Rehema", "TZA", F, None, Some(1876), 2004, None),
    p("Phiri, Chanda", "ZAM", M, None, Some(1834), 1997, None),
    p(
        "Nambi, Patience",
        "UGA",
        F,
        None,
        Some(1798),
        2010,
        Some("Mengo Chess Academy"),
    ),
    p("Otieno, Brian", "KEN", M, None, Some(1755), 2011, None),
    p("Nsubuga, Samuel", "UGA", M, None, Some(1720), 1960, KCC),
    p("Kagame, Eric", "RWA", M, None, Some(1690), 2012, None),
    p("Wanyama, Ruth", "KEN", F, None, Some(1650), 2009, None),
    p("Abebe, Selam", "ETH", F, None, Some(1602), 2013, None),
    p("Mutua, Daniel", "KEN", M, None, Some(1560), 1975, None),
    p(
        "Kasozi, Moses",
        "UGA",
        M,
        None,
        None,
        2014,
        Some("Mengo Chess Academy"),
    ),
    p("Akello, Sarah", "UGA", F, None, None, 2012, None),
];

/// Joins in round 2; round 1 counts as a half-point bye.
pub const LATE_ENTRY: SamplePerson = p(
    "Byaruhanga, Allan",
    "UGA",
    M,
    None,
    Some(1825),
    2000,
    Some("Mengo Chess Academy"),
);

const FIRST_NAMES: [(&str, Sex); 8] = [
    ("Amina", F),
    ("Joel", M),
    ("Faith", F),
    ("Isaac", M),
    ("Esther", F),
    ("Paul", M),
    ("Lydia", F),
    ("Henry", M),
];
const SURNAMES: [(&str, &str); 8] = [
    ("Kyomuhendo", "UGA"),
    ("Odhiambo", "KEN"),
    ("Mbeki", "RSA"),
    ("Ndayishimiye", "BDI"),
    ("Chanda", "ZAM"),
    ("Mushi", "TZA"),
    ("Adeyemi", "NGR"),
    ("Kariuki", "KEN"),
];

/// Fictional FIDE ID for the `i`-th person of the sample list.
pub fn sample_fide_id(i: usize) -> u32 {
    99_000_001 + i as u32
}

/// Everyone in the sample player list: the tournament's 25 players plus 64
/// more, for trying the player search.
pub fn sample_people() -> Vec<SamplePerson> {
    let mut v: Vec<SamplePerson> = ENTRANTS.to_vec();
    v.push(LATE_ENTRY);
    for (i, (surname, fed)) in SURNAMES.iter().enumerate() {
        for (j, (first, sex)) in FIRST_NAMES.iter().enumerate() {
            let name: &'static str = Box::leak(format!("{surname}, {first}").into_boxed_str());
            let rating = 1450 + ((i * 131 + j * 197) % 800) as u16;
            v.push(p(
                name,
                fed,
                *sex,
                None,
                Some(rating),
                1965 + ((i * 7 + j * 5) % 50) as u16,
                None,
            ));
        }
    }
    v
}

fn player(id: u32, person: &SamplePerson, fide_index: usize) -> Player {
    let mut pl = Player::new(PlayerId(id), person.name);
    pl.federation = Some(person.federation.to_string());
    pl.sex = Some(person.sex);
    pl.title = person.title;
    pl.ratings.fide_standard = person.rating;
    pl.ratings.fide_rapid = person.rating.map(|r| r.saturating_sub(35));
    pl.ratings.fide_blitz = person.rating.map(|r| r.saturating_sub(60));
    pl.fide_id = person.rating.map(|_| sample_fide_id(fide_index));
    pl.birth_date = Some(person.birth_year.to_string());
    pl.club = person.club.map(str::to_string);
    pl
}

/// Small deterministic generator so the sample is the same every time.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn color_balance(p: &PairingPlayer) -> i32 {
    p.played_colors()
        .map(|c| if c == Color::White { 1 } else { -1 })
        .sum()
}

/// Pairs top-down within score order, avoiding repeats and forbidden pairs.
fn demo_pairing(t: &Tournament) -> Vec<Pairing> {
    let input = PairingInput::for_next_round(t);
    let mut avail: Vec<&PairingPlayer> =
        input.players.iter().filter(|p| p.is_available()).collect();
    avail.sort_by_key(|p| (Reverse(p.score), p.starting_rank));
    let pab = if avail.len() % 2 == 1 {
        let i = avail
            .iter()
            .rposition(|p| !p.had_pairing_bye())
            .unwrap_or(avail.len() - 1);
        Some(avail.remove(i).id)
    } else {
        None
    };
    let mut games = Vec::new();
    while !avail.is_empty() {
        let a = avail.remove(0);
        let j = avail
            .iter()
            .position(|b| !a.opponents().any(|o| o == b.id) && !input.is_forbidden(a.id, b.id))
            .unwrap_or(0);
        let b = avail.remove(j);
        let pair = if color_balance(a) <= color_balance(b) {
            (a.id, b.id)
        } else {
            (b.id, a.id)
        };
        games.push(pair);
    }
    co_pairing::finalize_round(&input, &games, pab)
}

fn demo_result(t: &Tournament, white: PlayerId, black: PlayerId, rng: &mut Rng) -> GameResult {
    let rating = |id| {
        f64::from(
            t.player(id)
                .and_then(|p| p.rating(t.settings.rating_type))
                .unwrap_or(1400),
        )
    };
    let expected = 1.0 / (1.0 + 10f64.powf((rating(black) - rating(white)) / 400.0));
    let r = rng.next();
    if r < expected - 0.14 {
        GameResult::WHITE_WINS
    } else if r < expected + 0.14 {
        GameResult::DRAW
    } else {
        GameResult::BLACK_WINS
    }
}

fn play_round(
    t: &mut Tournament,
    events: &mut Vec<Event>,
    rng: &mut Rng,
    enter: impl Fn(u16) -> bool,
) {
    let round = t.paired_rounds() + 1;
    let pairings = demo_pairing(t);
    let ev = Event::RoundPaired {
        round,
        pairings: pairings.clone(),
        matches: Vec::new(),
        method: PairingMethod::Manual,
    };
    apply(t, events, ev);
    for p in pairings {
        if let Pairing::Game {
            board,
            white,
            black,
            ..
        } = p
            && enter(board)
        {
            // One forfeit in round 3 so the lists show one.
            let result = if round == 3 && board == 9 {
                GameResult::WHITE_FORFEIT_WIN
            } else {
                demo_result(t, white, black, rng)
            };
            apply(
                t,
                events,
                Event::ResultEntered {
                    round,
                    board,
                    result: Some(result),
                },
            );
        }
    }
}

fn apply(t: &mut Tournament, events: &mut Vec<Event>, ev: Event) {
    t.apply(&ev).expect("sample events are valid");
    events.push(ev);
}

/// The sample tournament as the event log an arbiter would have produced.
pub fn tournament_events() -> Vec<Event> {
    let date = |d| NaiveDate::from_ymd_opt(2026, 10, d);
    let info = TournamentInfo {
        name: "Kampala Rapid Open 2026 (sample)".into(),
        city: "Kampala".into(),
        federation: "UGA".into(),
        venue: "Sample Hall, Kampala".into(),
        start_date: date(3),
        end_date: date(5),
        chief_arbiter: "Sample Arbiter".into(),
        deputy_arbiters: vec!["Sample Deputy".into()],
        organizer: "Sample Chess Association".into(),
        time_control: "25 min + 10 s/move".into(),
        round_dates: [3, 3, 3, 4, 4, 5, 5].iter().map(|d| date(*d)).collect(),
    };
    let settings = Settings {
        rounds: 7,
        rating_type: RatingType::Rapid,
        ..Settings::default()
    };
    let first = Event::TournamentCreated {
        id: Uuid::new_v4(),
        info,
        settings,
        kind: TournamentKind::Swiss {
            system: SwissSystem::Dutch,
        },
    };
    let mut t = Tournament::create(&first).expect("valid creation");
    let mut events = vec![first];

    for (i, person) in ENTRANTS.iter().enumerate() {
        apply(
            &mut t,
            &mut events,
            Event::PlayerAdded {
                player: player(i as u32 + 1, person, i),
            },
        );
    }
    // C.04.2 initial order: rating, then title, then name.
    let mut order: Vec<&Player> = t.players.iter().collect();
    let rt = t.settings.rating_type;
    order.sort_by_key(|p| {
        (
            Reverse(p.rating(rt).unwrap_or(0)),
            p.title.is_none(),
            p.name.clone(),
        )
    });
    let order = order.into_iter().map(|p| p.id).collect();
    apply(&mut t, &mut events, Event::StartingRanksSet { order });

    // Club-mates Okello (1) and Lubega (12) must not meet.
    let mut s = t.settings.clone();
    s.forbidden_groups.push(ForbiddenGroup {
        name: "Kampala Chess Club (same family)".into(),
        members: vec![PlayerId(1), PlayerId(12)],
    });
    apply(&mut t, &mut events, Event::SettingsUpdated { settings: s });

    let prizes = [
        (
            "Under 1800",
            PrizeFilter {
                max_rating: Some(1799),
                ..PrizeFilter::default()
            },
            vec!["150,000 UGX", "100,000 UGX"],
        ),
        (
            "Best woman",
            PrizeFilter {
                sex: Some(Sex::Female),
                ..PrizeFilter::default()
            },
            vec!["150,000 UGX"],
        ),
        (
            "Under 16",
            PrizeFilter {
                born_from: Some(2010),
                ..PrizeFilter::default()
            },
            vec!["Trophy", "Medal"],
        ),
    ];
    for (i, (name, filter, list)) in prizes.into_iter().enumerate() {
        let category = PrizeCategory {
            id: PrizeCategoryId(i as u32 + 1),
            name: name.into(),
            filter,
            prizes: list.into_iter().map(str::to_string).collect(),
        };
        apply(&mut t, &mut events, Event::PrizeCategoryAdded { category });
    }

    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    play_round(&mut t, &mut events, &mut rng, |_| true);

    let mut late = player(25, &LATE_ENTRY, 24);
    late.starting_rank = 25;
    late.joined_round = 2;
    late.late_entry_bye = ByeKind::Half;
    apply(&mut t, &mut events, Event::PlayerAdded { player: late });

    play_round(&mut t, &mut events, &mut rng, |_| true);
    play_round(&mut t, &mut events, &mut rng, |_| true);

    // Before round 4: one withdrawal, one half-point bye, one future pause.
    apply(
        &mut t,
        &mut events,
        Event::PlayerWithdrawn {
            player: PlayerId(22),
            from_round: 4,
        },
    );
    apply(
        &mut t,
        &mut events,
        Event::ByeSet {
            player: PlayerId(9),
            round: 4,
            kind: Some(ByeKind::Half),
        },
    );
    apply(
        &mut t,
        &mut events,
        Event::ByeSet {
            player: PlayerId(14),
            round: 5,
            kind: Some(ByeKind::Zero),
        },
    );
    apply(
        &mut t,
        &mut events,
        Event::ByeSet {
            player: PlayerId(6),
            round: 6,
            kind: Some(ByeKind::Half),
        },
    );

    // Round 4 is in progress: results entered on the top half of the boards.
    play_round(&mut t, &mut events, &mut rng, |board| board <= 6);
    events
}

/// The sample tournament's final state.
pub fn tournament() -> Tournament {
    let events = tournament_events();
    let mut t = Tournament::create(&events[0]).expect("valid creation");
    for ev in &events[1..] {
        t.apply(ev).expect("valid sample");
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_is_consistent() {
        let t = tournament();
        assert_eq!(t.players.len(), 25);
        assert_eq!(t.paired_rounds(), 4);
        assert_eq!(t.completed_rounds(), 3);
        // Every completed round passes the shared rule checks.
        for r in 1..=4 {
            let input = PairingInput::for_existing_round(&t, r);
            let problems = co_pairing::check::common(&input, &t.round(r).unwrap().pairings);
            assert!(problems.is_empty(), "round {r}: {problems:?}");
        }
        // Round 4 has results on six boards only.
        let r4 = t.round(4).unwrap();
        let entered = r4
            .pairings
            .iter()
            .filter(|p| {
                matches!(
                    p,
                    Pairing::Game {
                        result: Some(_),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(entered, 6);
        assert!(
            r4.entry_for(PlayerId(22)).is_none(),
            "withdrawn player is not paired"
        );
        assert_eq!(
            r4.entry_for(PlayerId(9)),
            Some(&Pairing::Bye {
                player: PlayerId(9),
                kind: ByeKind::Half
            })
        );
    }

    #[test]
    fn sample_people_have_unique_names() {
        let people = sample_people();
        let mut names: Vec<_> = people.iter().map(|p| p.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), people.len());
        assert_eq!(people.len(), 25 + 64);
    }
}
