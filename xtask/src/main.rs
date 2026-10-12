//! Project automation: `cargo xtask <command>`.
//!
//! `oracle` runs the conformance test FIDE uses for endorsement: bbpPairings'
//! random tournament generator creates tournaments paired by bbpPairings
//! itself, and our engine must reproduce every round. The Phase 1 gate is at
//! least 4,990 of 5,000 tournaments reproduced exactly.

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use co_core::pairing_view::PairingInput;
use co_core::{Error as CoreError, PairingSystem, Tournament};
use co_io::trf;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

/// bbpPairings source used as the oracle. Pin a release tag once chosen;
/// `default` clones the repository's default branch.
const BBP_REPO: &str = "https://github.com/BieremaBoyzProgramming/bbpPairings";
const BBP_REF: &str = "default";

#[derive(Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Conformance testing against bbpPairings.
    Oracle {
        #[command(subcommand)]
        command: OracleCmd,
    },
    /// Regenerate the desktop UI's browser-preview fixture from the sample tournament.
    Fixture,
    /// Time the Dutch engine re-pairing every round of a TRF file.
    Bench {
        trf: PathBuf,
        #[arg(long, default_value_t = 1)]
        repeat: u32,
    },
}

#[derive(Subcommand)]
enum OracleCmd {
    /// Clone and build bbpPairings into third_party/.
    Build {
        #[arg(long, default_value = BBP_REF)]
        git_ref: String,
    },
    /// Generate random tournaments and compare every round with our engine.
    Run {
        #[arg(long, default_value_t = 10)]
        events: u32,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// bbpPairings binary (default: the one built by `oracle build`).
        #[arg(long)]
        bbp: Option<PathBuf>,
        /// Generator configuration file passed to `-g`.
        #[arg(long)]
        config: Option<PathBuf>,
        /// Fail unless at least this many events match exactly.
        #[arg(long)]
        require: Option<u32>,
        #[arg(long, default_value = "oracle-out")]
        out: PathBuf,
        /// Parallel workers (default: all cores).
        #[arg(long)]
        jobs: Option<usize>,
        /// Pairing system to compare: dutch or burstein.
        #[arg(long, default_value = "dutch")]
        system: String,
        /// Re-pair every round with Baku acceleration; each round is written
        /// as TRF and paired by both engines.
        #[arg(long)]
        baku: bool,
    },
    /// Re-pair every round of one TRF file and print the differences.
    Check { trf: PathBuf },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Cmd::Fixture => write_fixture(),
        Cmd::Bench { trf, repeat } => bench(&trf, repeat),
        Cmd::Oracle { command } => match command {
            OracleCmd::Build { git_ref } => build_bbp(&git_ref),
            OracleCmd::Check { trf } => check_file(&trf),
            OracleCmd::Run {
                events,
                seed,
                bbp,
                config,
                require,
                out,
                jobs,
                system,
                baku,
            } => {
                let system = match system.as_str() {
                    "dutch" => PairingSystem::Dutch,
                    "burstein" => PairingSystem::Burstein,
                    other => bail!("unknown system '{other}' (dutch or burstein)"),
                };
                let mode = Mode { system, baku };
                let bbp = match bbp {
                    Some(p) => p,
                    None => find_bbp()?,
                };
                let jobs = jobs
                    .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get()));
                run_oracle(
                    &bbp,
                    mode,
                    config.as_deref(),
                    events,
                    seed,
                    require,
                    &root().join(out),
                    jobs,
                )
            }
        },
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the workspace")
        .to_path_buf()
}

fn bbp_dir() -> PathBuf {
    root().join("third_party/bbpPairings")
}

fn build_bbp(git_ref: &str) -> Result<()> {
    let dir = bbp_dir();
    if !dir.exists() {
        std::fs::create_dir_all(dir.parent().unwrap())?;
        let mut git = Command::new("git");
        git.args(["clone", "--depth", "1"]);
        if git_ref != "default" {
            git.args(["--branch", git_ref]);
        }
        let status = git
            .arg(BBP_REPO)
            .arg(&dir)
            .status()
            .context("running git")?;
        if !status.success() {
            bail!("git clone failed");
        }
    }
    // Compile the sources directly: the upstream Makefile enables -Werror,
    // which some toolchains trip over with harmless warnings.
    let mut sources = Vec::new();
    collect_cpp(&dir.join("src"), &mut sources)?;
    let compiler = std::env::var("CXX").unwrap_or_else(|_| "c++".into());
    let status = Command::new(&compiler)
        .args(["-std=c++20", "-O2", "-DNDEBUG", "-DVERSION_INFO=\"oracle\""])
        .arg(format!("-I{}", dir.join("src").display()))
        .args(&sources)
        .arg("-o")
        .arg(dir.join("bbpPairings.exe"))
        .status()
        .context("running the C++ compiler (C++20 required)")?;
    if !status.success() {
        bail!("building bbpPairings failed");
    }
    println!("built {}", find_bbp()?.display());
    Ok(())
}

fn collect_cpp(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_cpp(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "cpp") {
            out.push(path);
        }
    }
    Ok(())
}

fn find_bbp() -> Result<PathBuf> {
    ["bbpPairings.exe", "bbpPairings"]
        .iter()
        .map(|n| bbp_dir().join(n))
        .find(|p| p.is_file())
        .context("bbpPairings not found; run `cargo xtask oracle build` first")
}

#[derive(Default)]
struct Tally {
    events: u32,
    /// Seeds the generator could not turn into a tournament.
    skipped: u32,
    parsed: u32,
    matched: u32,
    rounds: u32,
    rounds_matched: u32,
    planned: Option<String>,
    report: Vec<(u64, String)>,
}

/// What a run compares.
#[derive(Clone, Copy)]
struct Mode {
    system: PairingSystem,
    baku: bool,
}

impl Mode {
    /// Rounds in the generated file were paired by bbpPairings' Dutch engine
    /// without acceleration, so they can be compared directly; anything else
    /// is re-paired by bbpPairings from our TRF export.
    fn direct(self) -> bool {
        self.system == PairingSystem::Dutch && !self.baku
    }

    fn bbp_flag(self) -> &'static str {
        match self.system {
            PairingSystem::Burstein => "--burstein",
            _ => "--dutch",
        }
    }
}

/// Generates one random tournament and re-pairs each of its rounds.
fn oracle_event(
    bbp: &Path,
    mode: Mode,
    config: Option<&Path>,
    s: u64,
    out: &Path,
    tally: &mut Tally,
) -> Result<()> {
    let engine = co_pairing::engine_for(mode.system);
    let file = out.join(format!("event-{s}.trf"));
    let mut cmd = Command::new(bbp);
    cmd.arg("--dutch");
    match config {
        Some(c) => cmd.arg("-g").arg(c),
        None => cmd.arg("-g"),
    };
    let status = cmd
        .arg("-o")
        .arg(&file)
        .arg("-s")
        .arg(s.to_string())
        .stderr(std::process::Stdio::null())
        .status()
        .context("running bbpPairings")?;
    if !status.success() {
        // The generator itself sometimes cannot finish a random tournament
        // (no legal pairing for a late round). That seed is not an event.
        tally.skipped += 1;
        let _ = std::fs::remove_file(&file);
        return Ok(());
    }
    tally.events += 1;
    let text = std::fs::read_to_string(&file)?;
    let t = match trf::read(&text) {
        Ok(import) => {
            for w in import.warnings {
                tally.report.push((s, format!("warning: {w}")));
            }
            import.tournament
        }
        Err(e) => {
            tally.report.push((s, format!("TRF NOT READABLE: {e}")));
            return Ok(());
        }
    };
    tally.parsed += 1;
    let mut all_match = true;
    for r in 1..=t.paired_rounds() {
        tally.rounds += 1;
        let compared = if mode.direct() {
            compare_round(&t, r, engine.as_ref())
        } else {
            repair_round(
                bbp,
                mode,
                &t,
                r,
                engine.as_ref(),
                &out.join(format!("event-{s}-r{r}")),
            )
        };
        match compared {
            Ok(None) => tally.rounds_matched += 1,
            Ok(Some(diff)) => {
                all_match = false;
                tally.report.push((s, format!("round {r}: {diff}")));
                break;
            }
            Err(CoreError::NotYetImplemented { feature, phase }) => {
                all_match = false;
                tally.planned = Some(format!("{feature} is planned for Phase {phase}"));
                break;
            }
            Err(e) => {
                all_match = false;
                tally
                    .report
                    .push((s, format!("round {r}: engine error: {e}")));
                break;
            }
        }
    }
    if all_match {
        tally.matched += 1;
        // Keep failing events only, so the output folder stays small.
        let _ = std::fs::remove_file(&file);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_oracle(
    bbp: &Path,
    mode: Mode,
    config: Option<&Path>,
    events: u32,
    seed: u64,
    require: Option<u32>,
    out: &Path,
    jobs: usize,
) -> Result<()> {
    std::fs::create_dir_all(out)?;
    let jobs = jobs.max(1).min(events.max(1) as usize);
    use std::sync::atomic::{AtomicI64, AtomicU32, AtomicU64, Ordering};
    // Seeds are handed out until `events` tournaments were generated. A
    // worker reserves a slot first and gives it back if the generator fails.
    let generated = AtomicU32::new(0);
    let slots = AtomicI64::new(i64::from(events));
    let next_seed = AtomicU64::new(0);
    let results: Vec<Result<Tally>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..jobs)
            .map(|j| {
                let (generated, next_seed, slots) = (&generated, &next_seed, &slots);
                let _ = j;
                scope.spawn(move || -> Result<Tally> {
                    let mut tally = Tally::default();
                    while slots.fetch_sub(1, Ordering::Relaxed) > 0 {
                        let s = seed + next_seed.fetch_add(1, Ordering::Relaxed);
                        let before = tally.events;
                        oracle_event(bbp, mode, config, s, out, &mut tally)?;
                        if tally.events == before {
                            slots.fetch_add(1, Ordering::Relaxed);
                        } else {
                            let n = generated.fetch_add(1, Ordering::Relaxed) + 1;
                            if n % 100 == 0 {
                                eprintln!("  {n}/{events} events");
                            }
                        }
                    }
                    Ok(tally)
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("worker panicked"))
            .collect()
    });
    let mut tally = Tally::default();
    for r in results {
        let t = r?;
        tally.events += t.events;
        tally.skipped += t.skipped;
        tally.parsed += t.parsed;
        tally.matched += t.matched;
        tally.rounds += t.rounds;
        tally.rounds_matched += t.rounds_matched;
        tally.planned = tally.planned.or(t.planned);
        tally.report.extend(t.report);
    }
    tally.report.sort();
    let mut report = String::new();
    for (s, line) in &tally.report {
        writeln!(report, "seed {s}: {line}")?;
    }

    let summary = format!(
        "events: {}  readable TRF: {}  reproduced exactly: {}  rounds matched: {}/{}  (generator gave up on {} seeds)",
        tally.events,
        tally.parsed,
        tally.matched,
        tally.rounds_matched,
        tally.rounds,
        tally.skipped
    );
    std::fs::write(out.join("report.txt"), format!("{summary}\n{report}"))?;
    println!("{summary}");
    if let Some(p) = &tally.planned {
        println!("note: {p}; only TRF reading was checked");
    }
    println!("details: {}", out.join("report.txt").display());

    if tally.parsed < tally.events {
        bail!(
            "{} generated file(s) could not be read",
            tally.events - tally.parsed
        );
    }
    if let Some(min) = require.filter(|min| tally.matched < *min) {
        bail!("only {} events matched; {min} required", tally.matched);
    }
    Ok(())
}

fn check_file(path: &Path) -> Result<()> {
    let engine = co_pairing::engine_for(PairingSystem::Dutch);
    let t = trf::read(&std::fs::read_to_string(path)?)?.tournament;
    let mut ok = true;
    for r in 1..=t.paired_rounds() {
        match compare_round(&t, r, engine.as_ref()) {
            Ok(None) => println!("round {r}: same"),
            Ok(Some(diff)) => {
                ok = false;
                println!("round {r}: {diff}");
            }
            Err(e) => {
                ok = false;
                println!("round {r}: engine error: {e}");
            }
        }
    }
    if !ok {
        bail!("{} differs from bbpPairings", path.display());
    }
    Ok(())
}

/// The tournament as it stood before round `r` was paired, with the round's
/// requested byes and absences kept as bye requests.
fn before_round(t: &Tournament, r: u16) -> Tournament {
    let mut before = t.clone();
    before.rounds.truncate(usize::from(r) - 1);
    if let Some(round) = t.round(r) {
        for p in &round.pairings {
            if let co_core::Pairing::Bye { player, kind } = *p
                && kind != co_core::ByeKind::PairingAllocated
                && let Some(pl) = before.player_mut(player)
            {
                pl.byes.insert(r, kind);
            }
        }
    }
    before
}

/// Writes round `r`'s starting position as TRF, has bbpPairings pair it and
/// compares with our engine. `None` when both agree.
fn repair_round(
    bbp: &Path,
    mode: Mode,
    t: &Tournament,
    r: u16,
    engine: &dyn co_pairing::PairingEngine,
    stem: &Path,
) -> Result<Option<String>, CoreError> {
    let io = |e: std::io::Error| CoreError::Invalid(e.to_string());
    let mut before = before_round(t, r);
    if mode.baku {
        before.settings.acceleration = Some(co_core::AccelerationMethod::Baku);
        for p in &mut before.players {
            p.acceleration.clear();
        }
    }
    if mode.system == PairingSystem::Burstein {
        before.kind = co_core::TournamentKind::Swiss {
            system: co_core::SwissSystem::Burstein,
        };
    }
    let trf_path = stem.with_extension("trf");
    let out_path = stem.with_extension("txt");
    std::fs::write(&trf_path, trf::write(&before)).map_err(io)?;
    let status = Command::new(bbp)
        .arg(mode.bbp_flag())
        .arg(&trf_path)
        .arg("-p")
        .arg(&out_path)
        .output()
        .map_err(io)?;
    if !status.status.success() {
        return Ok(Some(format!(
            "bbpPairings failed: {}",
            String::from_utf8_lossy(&status.stderr).trim()
        )));
    }
    let expected: BTreeSet<(u32, u32)> =
        trf::bbp::parse_pairs(&std::fs::read_to_string(&out_path).map_err(io)?)
            .map_err(CoreError::Invalid)?
            .into_iter()
            .collect();
    let ids = trf::trf_ids(&before);
    let ours = engine.pair(&PairingInput::for_next_round(&before))?;
    let got: BTreeSet<(u32, u32)> = trf::bbp::round_to_pairs(&ours.pairings, &ids)
        .into_iter()
        .collect();
    if got == expected {
        let _ = std::fs::remove_file(&trf_path);
        let _ = std::fs::remove_file(&out_path);
        return Ok(None);
    }
    let missing: Vec<_> = expected.difference(&got).collect();
    let extra: Vec<_> = got.difference(&expected).collect();
    Ok(Some(format!(
        "expected but missing {missing:?}; produced instead {extra:?}"
    )))
}

/// `None` if our pairing of round `r` equals the one in the file.
fn compare_round(
    t: &Tournament,
    r: u16,
    engine: &dyn co_pairing::PairingEngine,
) -> Result<Option<String>, CoreError> {
    let ids = trf::trf_ids(t);
    let expected: BTreeSet<(u32, u32)> =
        trf::bbp::round_to_pairs(&t.round(r).expect("round exists").pairings, &ids)
            .into_iter()
            .collect();
    let ours = engine.pair(&PairingInput::for_existing_round(t, r))?;
    let got: BTreeSet<(u32, u32)> = trf::bbp::round_to_pairs(&ours.pairings, &ids)
        .into_iter()
        .collect();
    if got == expected {
        return Ok(None);
    }
    let missing: Vec<_> = expected.difference(&got).collect();
    let extra: Vec<_> = got.difference(&expected).collect();
    Ok(Some(format!(
        "expected but missing {missing:?}; produced instead {extra:?}"
    )))
}

/// Writes app/src/lib/dev/fixture.json: the sample tournament (co-io::sample) as the UI's
/// `TournamentView`, plus pre-rendered lists, for `pnpm dev` in a browser.
fn write_fixture() -> Result<()> {
    use co_io::report::{self, ListKind};
    use serde_json::json;

    let t = co_io::sample::tournament();
    let round_scores = co_core::scoring::cumulative_scores(&t);
    let view = json!({
        "tournament": t,
        "path": "preview/sample.cot",
        "can_undo": false,
        "can_redo": false,
        "undo_label": null,
        "scores": round_scores.last(),
        "round_scores": round_scores,
        "next_round": t.paired_rounds() + 1,
    });
    let mut kinds = vec![ListKind::StartingRank, ListKind::Alphabetical];
    for round in 1..=t.paired_rounds() {
        kinds.push(ListKind::Pairings { round });
        kinds.push(ListKind::Standings { round });
    }
    let tr = co_i18n::Translator::new("en");
    let mut lists = serde_json::Map::new();
    for kind in kinds {
        let html = co_io::render::html::render(&report::build(kind, &t, &tr)?)?;
        lists.insert(serde_json::to_string(&kind)?, html.into());
    }
    let fixture = json!({
        "view": view,
        "defaults": {
            "info": co_core::TournamentInfo::default(),
            "settings": co_core::Settings::default(),
        },
        "lists": lists,
        "people": co_io::sample::sample_people().iter().enumerate().map(|(i, p)| json!({
            "fide_id": p.rating.map(|_| co_io::sample::sample_fide_id(i)),
            "national_id": null,
            "name": p.name,
            "federation": p.federation,
            "sex": p.sex,
            "title": p.title,
            "rating_standard": p.rating,
            "rating_rapid": p.rating.map(|r| r.saturating_sub(35)),
            "rating_blitz": p.rating.map(|r| r.saturating_sub(60)),
            "national_rating": null,
            "k_factor": null,
            "birth_year": p.birth_year,
            "club": p.club,
        })).collect::<Vec<_>>(),
    });
    let out = root().join("app/src/lib/dev/fixture.json");
    std::fs::create_dir_all(out.parent().unwrap())?;
    std::fs::write(&out, serde_json::to_string(&fixture)?)?;
    println!("wrote {}", out.display());
    Ok(())
}

fn bench(path: &Path, repeat: u32) -> Result<()> {
    let t = trf::read(&std::fs::read_to_string(path)?)?.tournament;
    let engine = co_pairing::engine_for(PairingSystem::Dutch);
    let start = std::time::Instant::now();
    let mut mismatches = 0;
    for _ in 0..repeat {
        for r in 1..=t.paired_rounds() {
            if compare_round(&t, r, engine.as_ref())?.is_some() {
                mismatches += 1;
            }
        }
    }
    println!(
        "{} players, {} rounds x{repeat}: {:.2?} ({mismatches} mismatching rounds)",
        t.players.len(),
        t.paired_rounds(),
        start.elapsed()
    );
    Ok(())
}
