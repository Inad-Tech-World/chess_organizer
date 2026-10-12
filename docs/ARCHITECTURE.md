# Architecture

```
┌──────────────────────────── Desktop app (Tauri 2) ────────────────────────────┐
│  app/src  (SvelteKit SPA, Svelte 5)        app/src-tauri  (Rust)               │
│  screens · DataGrid · keyboard entry  ──▶  commands/* ──▶ AppState            │
│  Fluent i18n (locales/*.ftl)         ◀──   TournamentView (JSON)              │
└────────────────────────────────────────────────┬──────────────────────────────┘
                                                 │ function calls
┌──────────────────────── Rust core (no UI code) ▼──────────────────────────────┐
│ co-core     model · events · reducer · history (undo/redo) · scoring         │
│ co-pairing  PairingEngine trait · manual · dutch · berger · team · …          │
│ co-tiebreak C.07 tie-breaks · standings                                       │
│ co-rating   B.02 rating change · B.01 norms                                   │
│ co-io       TRF · rating lists · Report model → HTML/PDF/CSV/XLSX             │
│ co-store    .cot file (SQLite event log) · player database (FTS5)             │
│ co-i18n     Fluent loader shared with the UI                                  │
└───────────────────────────────────────────────────────────────────────────────┘
  co-cli (`co`)      co-publish (axum)      xtask oracle (bbpPairings)
```

Dependency direction: `co-core` ← `co-pairing`, `co-tiebreak`, `co-rating` ←
`co-io` ← `co-store` ← `app` / `co-cli` / `co-publish`. No crate under
`crates/` depends on Tauri, so the same engine can run in a web server, the
CLI or a future mobile app.

## Flow of a change

1. The UI calls a command (`app/src/lib/api/index.ts` → `invoke`).
2. The command looks up the window's open `TournamentFile`, builds an
   `Event` and calls `TournamentFile::append`.
3. `append` applies the event to a copy of the state (`Tournament::apply`).
   Integrity errors are rejected before anything is written. Then it
   commits the event in its own SQLite transaction.
4. The command returns a `TournamentView` (state plus derived scores and
   undo info), which the UI stores in `app.view`. Every screen derives from it.

Pairing engines never write. They take a `PairingInput`, the same
information a TRF file carries, and return proposed pairings, which become a
`RoundPaired` event.

## Lists

`co-io::report` builds every list as a renderer-neutral `Report` table. The
HTML renderer (minijinja, print CSS) serves the in-app preview, printing,
save-as-PDF and the publishing site. CSV, XLSX and direct PDF render the same
`Report`, so all outputs agree.

## Files

* `.cot`: SQLite with `meta`, `events`, `snapshots`. Rollback journal
  plus `synchronous=FULL` keep it a single, crash-safe file. `application_id`
  `COT1` identifies it, and `user_version` drives migrations.
* `players.db` in the app data folder holds imported rating lists, with an
  FTS5 index for accent-insensitive name search.
* `recent.json` in the app config folder lists recent files.

## Type sharing

`co-core` types derive `ts_rs::TS` behind the `ts` feature.
`cargo test -p co-core --features ts` regenerates `app/src/lib/bindings/`.
App-level DTOs are hand-written in `app/src/lib/api/types.ts`.
