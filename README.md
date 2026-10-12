# Chess Organiser

Free, open-source chess tournament management for macOS and Windows: Swiss,
round-robin and team events, from registration to the FIDE rating report.
Offline-first, with one file per tournament.

The goal is to cover the full arbiter workflow of commercial tournament
managers: create tournament, register players, pair, enter results, print
lists, publish, send the rating report. It has no player, round or team
limits and costs nothing.

> **Status: feature-complete, pre-1.0.** All six phases of the workplan are
> implemented: FIDE Dutch and Burstein pairings that reproduce the endorsed
> engine exactly (5,009 of 5,009 random tournaments), Baku acceleration,
> Dubov, Lim, Monrad, round robins, team events with line-ups, every C.07
> tie-break, rating changes, norm checks, prize lists, FIDE list import,
> PDF/Excel/CSV/HTML lists in six languages and a live results server.
> Installers are not yet signed. See [docs/roadmap.md](docs/roadmap.md) and the
> [arbiter's manual](docs/arbiter-manual.md).

## Layout

| Path | What |
|---|---|
| `crates/co-core` | Domain model, event log, reducer, undo/redo, scoring |
| `crates/co-pairing` | Pairing engines (FIDE Dutch, Dubov, Burstein, Lim, Monrad, Berger, team) |
| `crates/co-tiebreak` | FIDE C.07 tie-breaks and standings |
| `crates/co-rating` | FIDE rating changes (B.02) and title norms (B.01) |
| `crates/co-io` | TRF-2026, rating lists, imports; lists rendered to HTML/PDF/CSV/XLSX |
| `crates/co-store` | `.cot` tournament files (SQLite event log) and the player database |
| `crates/co-i18n` | Fluent translations shared by printed lists and the UI |
| `crates/co-cli` | `co` command line, bbpPairings-compatible |
| `crates/co-publish` | Live results web server |
| `app/` | Desktop app: Tauri 2 (`src-tauri/`) + SvelteKit (`src/`) |
| `xtask/` | `cargo xtask oracle` conformance harness, `cargo xtask fixture` |
| `locales/` | `.ftl` message files: en, fr, ar, sw, pt, es |
| `docs/` | Architecture, data model, FIDE rules map, roadmap, ADRs |

## Building

Requirements: Rust (stable, ≥ 1.90), Node 22 and pnpm. On Windows, the MSVC
build tools; on macOS, the Xcode command-line tools.

```bash
cd app && pnpm install && pnpm build && cd ..
cargo test --workspace          # all Rust tests
cd app && pnpm check && pnpm test
cargo tauri dev                 # run the desktop app (from app/)
cargo tauri build               # installers: .dmg/.app, .msi/.exe
```

The UI can also run in an ordinary browser against a sample tournament:
`pnpm --dir app dev`, then open http://localhost:1420. This browser preview
uses a mock backend and is development-only.

Command line:

```bash
cargo run -p co-cli -- convert testdata/trf/sample.trf /tmp/sample.cot
cargo run -p co-cli -- report standings /tmp/sample.cot --format html -o standings.html
cargo run -p co-cli -- pair --dutch tournament.trf -p   # also --burstein, --dubov, --lim, --monrad
```

## Pairing conformance

The FIDE Dutch engine is checked against
[bbpPairings](https://github.com/BieremaBoyzProgramming/bbpPairings), the
FIDE-endorsed open-source engine. Its random tournament generator produces
tournaments, and Chess Organiser must reproduce every round:

```bash
cargo xtask oracle build             # clones and builds bbpPairings into third_party/
cargo xtask oracle run --events 5000 --require 4990
```

## Licence

GPL-3.0-or-later. See [LICENSE](LICENSE) and [NOTICE](NOTICE). Chess
Organiser is not affiliated with FIDE or with any other tournament program.
