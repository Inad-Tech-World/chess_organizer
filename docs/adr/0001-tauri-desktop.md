# 1. Desktop first, with Tauri 2 and a pure-Rust core

Status: accepted (2026-10-11)

## Context

Tournament halls often have unreliable internet; arbiters expect to own their
files; a web service needs hosting money nobody has. The reference programs
are desktop applications with dense tables, many dialogs and printing.

## Decision

Build a desktop app for macOS and Windows with Tauri 2: the UI in Svelte 5 +
TypeScript, all chess logic in Rust crates that know nothing about the UI.
A small optional web server (Phase 5) reuses the same crates for live
results.

## Consequences

* HTML/CSS tables, printing and right-to-left languages come from the web
  platform; installers stay small (system WebView).
* The core can later power a CLI (already: `co`), a web app or mobile.
* Windows needs the WebView2 runtime (bootstrapped by the installer).
