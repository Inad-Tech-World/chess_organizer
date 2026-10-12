# Roadmap and status

Six phases, each closed by a real-world gate (see the project workplan).
The skeleton gives every phase its final API, screen and menu entry. Each
phase only replaces the stubs listed in its row. A stub returns
`Error::NotYetImplemented { feature, phase }`; the UI shows it as "Planned for
Phase N" and the CLI exits with code 3.

## Works now (skeleton)

- Tournament files (`.cot`): create, open, save a copy, several open at once
  (one per window), crash-safe, unlimited undo/redo with a full audit trail.
- Tournament setup: information, all tournament types, rounds, rating type,
  point system, C.07 tie-break configuration (TRF-2026 codes), forbidden
  pairings, team settings.
- Players: add, edit, remove, late entries (zero- or half-point byes for
  missed rounds), withdraw/re-enter, requested byes and pauses, FIDE C.04.2
  starting-rank sort.
- Player database: search by name (accent-insensitive) or ID. List import
  is a Phase 1 stub.
- Rounds: manual pairing with rule checks (repeat pairings, forbidden pairs,
  availability, PAB rules) and a "pair anyway" override; keyboard result
  entry; colour swaps; delete the last round; check any round.
- Scoring for any point system, including byes and late entries.
- Lists: starting rank, alphabetical, pairings, standings (points), rendered
  as HTML for preview, printing and save-as-PDF via the browser, plus HTML/CSV
  export. Lists can be printed in a different language from the UI, with RTL
  support.
- TRF-2026 export and TRF-2026/TRF-16/bbpPairings import (round-trip tested).
- `co` CLI, publishing-server skeleton, bbpPairings conformance harness.

## Stubs by phase

| Phase | Stubs to fill in | Gate |
|---|---|---|
| 1 · Individual Swiss MVP | `co-pairing::dutch`, `co-pairing::matching`; core tie-breaks (BH, SB, PS, DE, WIN + cut/median); `co-io::fide_list`, `national_list`; `render::pdf`; crosstable list | Oracle ≥ 4,990/5,000 events; one real club tournament |
| 2 · Complete individual | `berger` (single/double RR), `baku`; all individual tie-breaks; `render::xlsx`; scorecards, board cards; `co-rating` rating changes and the rating report list | A FIDE-rated event run beside the reference program; TRF accepted |
| 3 · Teams + prizes | `team_swiss`, `team_rr`, team standings and tie-breaks, TRF-2026 team records (310/3xx/80x), prize lists, team rosters | A national team championship or league |
| 4 · Advanced systems | `dubov`, `burstein`, `lim`, `monrad`; `co-rating::norm_check`; `swiss_manager_import`; conformance dossier | Endorsement dossier submitted to FIDE |
| 5 · Publishing + v1.0 | `co-publish` hardening + app upload; fr/ar/sw/pt/es translations; signed installers + updater; arbiter manual | v1.0 release |

## Known follow-ups

- `TournamentView` sends the whole tournament to the UI after every change.
  This is fine for club events. For 2,000-player opens, switch to diffs or
  per-screen queries.
- Windows builds are verified in CI only, so CI has to run before relying
  on them.
