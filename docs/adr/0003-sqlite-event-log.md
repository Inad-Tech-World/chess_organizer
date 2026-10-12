# 3. One SQLite file per tournament, holding an append-only event log

Status: accepted (2026-10-11)

## Decision

A tournament is a `.cot` file: SQLite (bundled with rusqlite, so no system
library) with an `events` table. State is rebuilt by replaying events;
undo/redo are events too. Each event commits in its own transaction with
`journal_mode=DELETE` and `synchronous=FULL`.

## Consequences

* One file to copy, e-mail or archive; survives power loss mid-round.
* Unlimited undo and a complete audit trail for appeals.
* Replay cost grows with the log; `snapshots` caches the latest state.
* Schema changes go through `co-store::schema` migrations; serde defaults
  keep old event payloads readable.
