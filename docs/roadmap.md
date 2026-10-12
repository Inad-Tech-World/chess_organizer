# Roadmap and status

Six phases, each closed by a real-world gate (see the project workplan).
All the software work in the six phases is done. What remains are the gates
that need people and accounts: a pilot tournament, a FIDE-rated event run
beside the reference program, a national team event, the FIDE endorsement
dossier, and signing keys for the installers.

## Status by phase

| Phase | Delivered | Evidence | Gate still open |
|---|---|---|---|
| 0 · Foundations | Workspace, `.cot` event-log file, undo/redo, TRF-2026 I/O, report model, desktop shell, CI | `cargo test --workspace` | — |
| 1 · Individual Swiss MVP | FIDE Dutch engine (port of bbpPairings), blossom matching, core tie-breaks, FIDE XML/TXT/zip list import + one-click download, national CSV import with column mapping, PDF renderer, crosstable | Oracle: **5,009 / 5,009 events, 50,094 / 50,094 rounds** identical to bbpPairings (seed 1) | One real club tournament |
| 2 · Complete individual | Berger round robin (single and double, C.05), Baku acceleration, all individual C.07 tie-breaks, Excel export, scorecards, board cards, FIDE B.02 rating changes and rating report | Berger tables match C.05; Baku oracle **500 / 500 events, 4,981 / 4,981 rounds** | A FIDE-rated event run beside the reference program |
| 3 · Teams + prizes | Team Swiss and team round robin engines, line-ups, team standings and team tie-breaks, TRF-2026 team records (310) with matches rebuilt on import, prize lists | Engine and round-trip tests | A national team championship or league |
| 4 · Advanced systems | Burstein (port of bbpPairings), Dubov, Lim, Monrad; GM/IM/WGM/WIM norm checks (B.01); import of exported files (TRF, player lists) | Burstein oracle **989 / 1,000 events**; the 11 others are bbpPairings crashes, see below. All Swiss engines pass the absolute-criteria simulation tests | Endorsement dossier submitted to FIDE |
| 5 · Publishing + v1.0 | Publishing server (persistence, index pages, all lists, languages, withdrawals) and in-app upload; French, Arabic, Swahili, Portuguese and Spanish; updater wiring; arbiter manual | Translation completeness test; server tests | **Signing**: Apple Developer ID + notarisation, a Windows code-signing certificate, and the update-signing key (see [releasing.md](releasing.md)) |

## Conformance runs

Run with `cargo xtask oracle run`; a mismatching event's TRF is kept in the
output folder and `cargo xtask oracle check <file>` re-pairs it.

```bash
cargo xtask oracle run --events 5000 --seed 1 --require 4990          # Dutch
cargo xtask oracle run --events 500 --seed 9001 --baku                # Dutch + Baku
cargo xtask oracle run --events 1000 --seed 7001 --system burstein \
  --config testdata/oracle/small.cfg                                  # Burstein
```

bbpPairings' Burstein engine crashes (segmentation fault) on some inputs,
more often with large fields, so Burstein runs use 30-player events
(`testdata/oracle/small.cfg`). Every event where bbpPairings produced a
pairing matched ours. Dubov, Lim and Monrad have no endorsed reference
program; their engines are covered by simulation tests of the absolute
criteria and the round-1 rules, and the setup screen tells arbiters to
review their pairings.

## Known follow-ups

- `TournamentView` sends the whole tournament to the UI after every change.
  This is fine for club events. For 2,000-player opens, switch to diffs or
  per-screen queries.
- The native menu bar is in English; the rest of the program is translated.
- TRF-2026 team records are written with team number, name and members; the
  optional summary columns of record 310 are left blank.
- Team Swiss uses the Dutch criteria over teams. The FIDE team Swiss rules
  (C.04.6) differ in details, e.g. colour-preference types A and B; compare
  with the regulations before using it for a FIDE team event.
- Windows builds are verified in CI only, so CI has to run before relying
  on them.
