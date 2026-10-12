//! `co`: the Chess Organiser command line.
//!
//! `co pair` and `co check` take the same arguments as bbpPairings so the
//! conformance harness can run both engines the same way.

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use co_core::pairing_view::PairingInput;
use co_core::{Event, PairingSystem, Tournament};
use co_i18n::Translator;
use co_io::render::Format;
use co_io::report::{self, ListKind};
use co_io::trf;
use co_store::TournamentFile;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "co", version, about = "Chess Organiser command line")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Pair the next round of a TRF file (bbpPairings-compatible).
    Pair {
        #[command(flatten)]
        system: SystemArg,
        input: PathBuf,
        /// Write the pairings here (stdout if omitted).
        #[arg(short = 'p', num_args = 0..=1, default_missing_value = "-")]
        output: Option<String>,
    },
    /// Check the pairings of every round of a TRF file.
    Check {
        #[command(flatten)]
        system: SystemArg,
        input: PathBuf,
        #[arg(short = 'c')]
        check: bool,
    },
    /// Convert between TRF and .cot tournament files.
    Convert { input: PathBuf, output: PathBuf },
    /// Render a list from a .cot or TRF file.
    Report {
        #[arg(value_enum)]
        list: ListArg,
        input: PathBuf,
        /// Round for pairings/standings/crosstable (default: last paired).
        #[arg(long)]
        round: Option<u16>,
        #[arg(long, value_enum, default_value = "html")]
        format: FormatArg,
        #[arg(long, default_value = "en")]
        lang: String,
        /// Output file (stdout if omitted).
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct SystemArg {
    #[arg(long)]
    dutch: bool,
    #[arg(long)]
    burstein: bool,
    #[arg(long)]
    dubov: bool,
    #[arg(long)]
    lim: bool,
    #[arg(long)]
    monrad: bool,
}

impl SystemArg {
    fn system(&self) -> PairingSystem {
        if self.burstein {
            PairingSystem::Burstein
        } else if self.dubov {
            PairingSystem::Dubov
        } else if self.lim {
            PairingSystem::Lim
        } else if self.monrad {
            PairingSystem::Monrad
        } else {
            PairingSystem::Dutch
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum ListArg {
    StartingRank,
    Alphabetical,
    Pairings,
    Standings,
    Crosstable,
    Scorecards,
    BoardCards,
    Prizes,
    RatingReport,
    TeamRosters,
    TeamPairings,
    TeamStandings,
}

#[derive(Clone, Copy, ValueEnum)]
enum FormatArg {
    Html,
    Pdf,
    Csv,
    Xlsx,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            // Distinct exit code for "planned for a later phase", so scripts can tell.
            let planned = e.chain().any(|c| {
                matches!(
                    c.downcast_ref::<co_core::Error>(),
                    Some(co_core::Error::NotYetImplemented { .. })
                )
            });
            ExitCode::from(if planned { 3 } else { 1 })
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Pair {
            system,
            input,
            output,
        } => {
            let t = read_trf(&input)?;
            let engine = co_pairing::engine_for(system.system());
            let round = engine.pair(&PairingInput::for_next_round(&t))?;
            let ids = trf::trf_ids(&t);
            let text = trf::bbp::format_pairs(&trf::bbp::round_to_pairs(&round.pairings, &ids));
            write_out(
                output.as_deref().filter(|o| *o != "-").map(Path::new),
                text.as_bytes(),
            )
        }
        Command::Check {
            system,
            input,
            check: _,
        } => {
            let full = read_trf(&input)?;
            let engine = co_pairing::engine_for(system.system());
            let mut problems = 0;
            for r in 1..=full.paired_rounds() {
                let input = PairingInput::for_existing_round(&full, r);
                let proposed = &full.round(r).expect("round exists").pairings;
                for v in engine.check(&input, proposed) {
                    println!("round {r}: {v}");
                    problems += 1;
                }
            }
            if problems > 0 {
                bail!("{problems} problem(s) found");
            }
            println!("no problems found");
            Ok(())
        }
        Command::Convert { input, output } => convert(&input, &output),
        Command::Report {
            list,
            input,
            round,
            format,
            lang,
            output,
        } => {
            let t = load(&input)?;
            let round = round.unwrap_or(t.paired_rounds()).max(1);
            let kind = match list {
                ListArg::StartingRank => ListKind::StartingRank,
                ListArg::Alphabetical => ListKind::Alphabetical,
                ListArg::Pairings => ListKind::Pairings { round },
                ListArg::Standings => ListKind::Standings { round },
                ListArg::Crosstable => ListKind::Crosstable { round },
                ListArg::Scorecards => ListKind::Scorecards,
                ListArg::BoardCards => ListKind::BoardCards { round },
                ListArg::Prizes => ListKind::Prizes,
                ListArg::RatingReport => ListKind::RatingReport,
                ListArg::TeamRosters => ListKind::TeamRosters,
                ListArg::TeamPairings => ListKind::TeamPairings { round },
                ListArg::TeamStandings => ListKind::TeamStandings { round },
            };
            let format = match format {
                FormatArg::Html => Format::Html,
                FormatArg::Pdf => Format::Pdf,
                FormatArg::Csv => Format::Csv,
                FormatArg::Xlsx => Format::Xlsx,
            };
            let report = report::build(kind, &t, &Translator::new(&lang)).map_err(flatten)?;
            let bytes = co_io::render::render(&report, format).map_err(flatten)?;
            write_out(output.as_deref(), &bytes)
        }
    }
}

/// Surfaces a wrapped `co_core::Error` so `main` can detect "planned" stubs.
fn flatten(e: co_io::Error) -> anyhow::Error {
    match e {
        co_io::Error::Core(core) => core.into(),
        other => other.into(),
    }
}

fn read_trf(path: &Path) -> Result<Tournament> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let import = trf::read(&text)?;
    for w in &import.warnings {
        eprintln!("warning: {w}");
    }
    Ok(import.tournament)
}

fn is_cot(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e == TournamentFile::EXTENSION)
}

fn load(path: &Path) -> Result<Tournament> {
    if is_cot(path) {
        Ok(TournamentFile::open(path)?.state().clone())
    } else {
        read_trf(path)
    }
}

fn convert(input: &Path, output: &Path) -> Result<()> {
    let t = load(input)?;
    if is_cot(output) {
        let source = format!(
            "{}",
            input.file_name().unwrap_or_default().to_string_lossy()
        );
        let file = TournamentFile::create(
            output,
            Event::TournamentImported {
                tournament: Box::new(t),
                source,
            },
        )?;
        file.checkpoint()?;
    } else {
        std::fs::write(output, trf::write(&t))?;
    }
    eprintln!("wrote {}", output.display());
    Ok(())
}

fn write_out(path: Option<&Path>, bytes: &[u8]) -> Result<()> {
    match path {
        Some(p) => std::fs::write(p, bytes).with_context(|| format!("writing {}", p.display())),
        None => {
            use std::io::Write;
            std::io::stdout().write_all(bytes)?;
            Ok(())
        }
    }
}
