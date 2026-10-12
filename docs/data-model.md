# Data model

Defined in `crates/co-core/src/model.rs` and `event.rs`.

## State

`Tournament` = `info` + `settings` + `kind` + `players` + `teams` + `rounds`
\+ `prize_categories`. It covers every tournament type from all phases, so
files written today stay readable as features land. Settings structs use
`#[serde(default)]`, so new fields never break old files.

* **Players** are identified by a `PlayerId` that is never reused. Starting
  rank is a separate, editable field. Requested future byes and pauses live
  in `Player::byes`. Late entries have `joined_round > 1` and a
  `late_entry_bye` (zero or half point) for the rounds they missed.
* **Rounds** are the source of truth for the past. Each holds `Pairing`s:
  `Game { board, white, black, result }` or `Bye { player, kind }`. When a
  round is paired, every available player gets an entry, so a round fully
  describes who got what. Team events store `TeamMatch`es with line-ups,
  and their games are rebuilt from the line-ups.
* **Results** are recorded per side (`SideResult`), which covers 1-0, ½-½,
  forfeits, double forfeits, unrated games and arbiter decisions such as 0-0.
* **Points** are fixed-point tenths (`Points(15)` = 1½), so any point system
  compares exactly.

## Events

Every change is an `Event`, stored as `Envelope { seq, at, event }`:

| Area | Events |
|---|---|
| Tournament | `TournamentCreated`, `TournamentImported`, `InfoUpdated`, `SettingsUpdated`, `KindChanged` |
| Players | `PlayerAdded`, `PlayerUpdated`, `PlayerRemoved`, `PlayerWithdrawn`, `PlayerReentered`, `ByeSet`, `StartingRanksSet` |
| Rounds | `RoundPaired`, `PairingsEdited`, `RoundDeleted`, `ResultEntered` |
| Teams | `TeamAdded`, `TeamUpdated`, `TeamRemoved`, `LineupSet`, `TeamGameResultEntered` |
| Prizes | `PrizeCategoryAdded`, `PrizeCategoryUpdated`, `PrizeCategoryRemoved` |
| History | `Undo { target }`, `Redo { target }` |

`Tournament::apply` is the only place state changes. It checks integrity:
unknown players, duplicate boards, rounds out of sequence, results on
unpaired boards, removing players who have played. Chess-rule checks
(repeat pairings, forbidden pairs, PAB rules) are pairing-engine
*violations*, so the arbiter can override them.

## Undo and redo

Undo and redo work like a text editor's. `Undo` reverts the latest effective
event, `Redo` re-applies the latest undone one, and a new event clears the
redo stack. They are stored as events, so the audit trail keeps everything,
which helps with appeals. `history::replay` rebuilds the state from the
effective events. Creation cannot be undone.
