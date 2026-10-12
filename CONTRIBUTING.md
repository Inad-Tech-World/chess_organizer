# Contributing

Thank you for helping arbiters everywhere run tournaments for free.

## Setup

* Rust stable (≥ 1.90), Node 22, pnpm.
* `cd app && pnpm install && pnpm build` once (the Tauri crate embeds the
  built UI), then `cargo test --workspace`.
* Run the app with `cargo tauri dev` from `app/`. To work on the UI in a
  plain browser, run `pnpm --dir app dev` (mock backend with a sample
  tournament).

## Before you open a pull request

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd app && pnpm check && pnpm test
```

If you change `co-core` types, regenerate the TypeScript bindings with
`cargo test -p co-core --features ts` and commit `app/src/lib/bindings/`.
If you change list layouts, review and accept insta snapshots
(`cargo insta review`, or `INSTA_UPDATE=always cargo test -p co-io`).

## Working in phases

Work happens phase by phase (see `docs/roadmap.md`). Each phase only fills
stubs that return `Error::NotYetImplemented`; the API, screens and menus
already exist. Nothing from a later phase starts until the current phase has
passed its gate.

Pairing changes must keep `cargo xtask oracle run` at 100% on the events
already passing.

## Rules

* Chess rules belong in `crates/`, never in the UI.
* Every state change is an `Event`; never mutate state outside
  `Tournament::apply`.
* Cite the FIDE Handbook article in code comments where a rule is
  implemented, and update `docs/rules-map.md`.
* User-facing text goes in `locales/en/*.ftl`.
* Read `docs/legal.md`: copy workflows, never other programs' code,
  screens, manuals or private file formats.

By contributing you agree that your work is licensed under GPL-3.0-or-later.
