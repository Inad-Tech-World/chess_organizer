# Chess Organiser

Free, open-source chess tournament management for macOS and Windows: Swiss,
round-robin and team events, from registration to the FIDE rating report.
Offline-first, with one file per tournament.

The goal is to cover the full arbiter workflow of commercial tournament
managers: create tournament, register players, pair, enter results, print
lists, publish, send the rating report. It has no player, round or team
limits and costs nothing.

> **Status: skeleton (all six phases scaffolded).** Every screen, menu, crate
> and API for Phases 0–5 exists and is wired end to end. The foundations
> work today: tournament files with unlimited undo, players, manual pairing,
> keyboard result entry, standings by points, printable lists, TRF-2026
> import/export, the player database and the CLI. Each phase's logic is
> filled in phase by phase. Planned features answer with "Planned for Phase N"
> instead of failing silently. See [docs/roadmap.md](docs/roadmap.md).

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
| `crates/co-publish` | Live results web server (Phase 5) |
| `app/` | Desktop app: Tauri 2 (`src-tauri/`) + SvelteKit (`src/`) |
| `xtask/` | `cargo xtask oracle` conformance harness, `cargo xtask fixture` |
| `locales/` | `.ftl` message files (en complete; fr, ar, sw, pt, es in Phase 5) |
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
cargo run -p co-cli -- pair --dutch tournament.trf -p   # Phase 1
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
