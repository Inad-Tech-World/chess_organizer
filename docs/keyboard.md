# Keyboard

## Result entry (Pairings & results)

Select a board (click or arrow keys), then type:

| Key | Result |
|---|---|
| `1` | 1-0 |
| `0` | 0-1 |
| `5` or `=` | ½-½ |
| `+` | +/- (White wins by forfeit) |
| `-` | -/+ (Black wins by forfeit) |
| `*` | -/- (double forfeit) |
| `Delete` / `Backspace` | clear the result |

After a result is entered, the cursor moves to the next board. The keymap is
in `app/src/lib/keyboard/results.ts`.

## Lists and grids

`↑` `↓` `PgUp` `PgDn` `Home` `End` move; `Enter` edits; `Insert` adds a
player; `Delete` removes one who has not played.

## Menus

| Shortcut (macOS / Windows) | Action |
|---|---|
| ⌘N / Ctrl+N | New tournament |
| ⌘O / Ctrl+O | Open |
| ⌘⇧O / Ctrl+Shift+O | Open in new window |
| ⌘I / Ctrl+I | Tournament setup |
| ⌘E / Ctrl+E | Export TRF |
| ⌘⇧S / Ctrl+Shift+S | Save a copy |
| ⌘⌥Z / Ctrl+Alt+Z | Undo tournament change (⌘Z/Ctrl+Z also works outside text fields) |
| ⌘⌥⇧Z / Ctrl+Alt+Shift+Z | Redo |
| ⌘⇧N / Ctrl+Shift+N | Add player |
| ⌘⇧D / Ctrl+Shift+D | Player database |
| ⌘⇧P / Ctrl+Shift+P | Pair next round |
| ⌘1 … ⌘7 / Ctrl+1 … Ctrl+7 | Players, Rounds, Lists, Teams, Prizes, Ratings, Publish |
| F1 | Keyboard shortcuts |
