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
    /// Regenerate the desktop UI's browser-preview fixture from testdata/trf/sample.trf.
    Fixture,
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
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Cmd::Fixture => write_fixture(),
        Cmd::Oracle { command } => match command {
            OracleCmd::Build { git_ref } => build_bbp(&git_ref),
            OracleCmd::Run {
                events,
                seed,
                bbp,
                config,
                require,
                out,
            } => {
                let bbp = match bbp {
                    Some(p) => p,
                    None => find_bbp()?,
                };
                run_oracle(
                    &bbp,
                    config.as_deref(),
                    events,
                    seed,
                    require,
                    &root().join(out),
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
    let status = Command::new("make")
        .arg("-j")
        .current_dir(&dir)
        .status()
        .context("running make (a C++20 compiler is required)")?;
    if !status.success() {
        bail!("building bbpPairings failed");
    }
    println!("built {}", find_bbp()?.display());
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
    parsed: u32,
    matched: u32,
    rounds: u32,
    rounds_matched: u32,
    planned: Option<String>,
}

fn run_oracle(
    bbp: &Path,
    config: Option<&Path>,
    events: u32,
    seed: u64,
    require: Option<u32>,
    out: &Path,
) -> Result<()> {
    std::fs::create_dir_all(out)?;
    let engine = co_pairing::engine_for(PairingSystem::Dutch);
    let mut tally = Tally::default();
    let mut report = String::new();

    for i in 0..events {
        let s = seed + u64::from(i);
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
            .status()
            .context("running bbpPairings")?;
        if !status.success() {
            bail!("bbpPairings generator failed for seed {s}");
        }
        tally.events += 1;

        let text = std::fs::read_to_string(&file)?;
        let t = match trf::read(&text) {
            Ok(import) => {
                for w in import.warnings {
                    writeln!(report, "seed {s}: warning: {w}")?;
                }
                import.tournament
            }
            Err(e) => {
                writeln!(report, "seed {s}: TRF NOT READABLE: {e}")?;
                continue;
            }
        };
        tally.parsed += 1;

        let mut all_match = true;
        for r in 1..=t.paired_rounds() {
            tally.rounds += 1;
            match compare_round(&t, r, engine.as_ref()) {
                Ok(None) => tally.rounds_matched += 1,
                Ok(Some(diff)) => {
                    all_match = false;
                    writeln!(report, "seed {s} round {r}: {diff}")?;
                    break;
                }
                Err(CoreError::NotYetImplemented { feature, phase }) => {
                    all_match = false;
                    tally.planned = Some(format!("{feature} is planned for Phase {phase}"));
                    break;
                }
                Err(e) => {
                    all_match = false;
                    writeln!(report, "seed {s} round {r}: engine error: {e}")?;
                    break;
                }
            }
        }
        if all_match {
            tally.matched += 1;
        }
    }

    let summary = format!(
        "events: {}  readable TRF: {}  reproduced exactly: {}  rounds matched: {}/{}",
        tally.events, tally.parsed, tally.matched, tally.rounds_matched, tally.rounds
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

/// Writes app/src/lib/dev/fixture.json: the sample tournament as the UI's
/// `TournamentView`, plus pre-rendered lists, for `pnpm dev` in a browser.
fn write_fixture() -> Result<()> {
    use co_io::report::{self, ListKind};
    use serde_json::json;

    let text = std::fs::read_to_string(root().join("testdata/trf/sample.trf"))?;
    let t = trf::read(&text)?.tournament;
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
    });
    let out = root().join("app/src/lib/dev/fixture.json");
    std::fs::create_dir_all(out.parent().unwrap())?;
    std::fs::write(&out, serde_json::to_string(&fixture)?)?;
    println!("wrote {}", out.display());
    Ok(())
}
