# Chess Organiser — arbiter's manual

This manual walks through a tournament from registration to the rating
report. Every screen is reached from the sidebar, the menu bar, or the
shortcuts in [keyboard.md](keyboard.md).

To try things without risk, open **File › Open Sample Tournament** (or the
button on the start screen): a fictional 25-player rapid open with four
rounds played. Every change can be undone.

## 1. Create the tournament

**File › New Tournament…** (⌘N / Ctrl+N) asks for a name, the tournament
type and where to save the file. A tournament is one `.cot` file. It is
saved after every change, survives a power cut, and can be copied or
e-mailed like any document. Several tournaments can be open at once, each in
its own window (**File › Open in New Window**).

Tournament types:

| Type | Pairing |
|---|---|
| Swiss – FIDE Dutch | C.04.3 (2025). Reproduces bbpPairings, FIDE's endorsed engine, exactly. |
| Swiss – FIDE Burstein | C.04.4.2. Reproduces bbpPairings. |
| Swiss – FIDE Dubov, Lim | C.04.4.1, C.04.4.3. No endorsed reference exists; review the pairings. |
| Swiss – simple (Monrad) | Club play: top unpaired player meets the next one not yet met. |
| Round robin / double | Berger tables (C.05 Annex 1). |
| Team Swiss / team round robin | Teams paired by match (or game) points; line-ups per round. |

## 2. Tournament setup

**Tournament setup** (⌘I / Ctrl+I) holds:

- **Information**: venue, dates, arbiters, time control. These print on every list.
- **System and rounds**: number of rounds, rating type (standard, rapid,
  blitz), the colour of the top seed on board 1 in round 1, and
  acceleration (Baku, or manual from a TRF file).
- **Points**: win, draw, loss, forfeits and every kind of bye. The default
  is 1 / ½ / 0 with a 1-point pairing-allocated bye.
- **Tie-breaks**: any of the FIDE C.07 tie-breaks, in order, with cut,
  median and forfeit modifiers. Codes follow TRF-2026 (e.g. `BH/C1`).
- **Forbidden pairings**: groups of players who must not meet (club-mates,
  family members). Enter starting ranks, e.g. `3, 17, 24`.
- **Team settings** (team events): boards per match, reserves, match
  points, ranking by match or game points.

The tournament type cannot change once round 1 is paired.

## 3. Players

Add players by hand (**Add player**, ⌘⇧N), from the **player database**, or
by importing a TRF file. **Sort starting rank** orders players by rating,
title and name as C.04.2 requires; do this before pairing round 1.

### Player database

**Player database** › **Download FIDE list** fetches the current FIDE
rating list (about 40 MB) and stores it on the computer; **Import FIDE
rating list** reads a list you saved (zip, XML or TXT). **Import national
list (CSV)** reads any federation list: choose which column holds the name,
rating, IDs and so on. After that, typing two letters of a name in the
player form finds the player and fills in title, federation, ratings and
IDs.

### During the tournament

- **Late entry**: give the player a starting round; earlier rounds count as
  zero- or half-point byes as you choose.
- **Byes and pauses**: request a half-, full- or zero-point bye for any
  future round. Requested byes are honoured when the round is paired.
- **Withdraw**: the player is not paired from the chosen round on. **Re-enter**
  undoes this.

## 4. Pairing a round

**Pairings & results** › the "next" tab › **Pair automatically**
(⌘⇧P / Ctrl+⇧P). All results of the previous round must be entered first.

To pair by hand, click two players to put them on a board (the first gets
White) and give the odd player the pairing-allocated bye. **Pair round N**
checks the rules: repeat pairings, forbidden pairs, unavailable players,
second byes. If you decide to keep a pairing that breaks a rule, choose
**Pair anyway**; the decision is recorded in the file's history.

After pairing you can still **Swap colours** on a board, **Check pairings**,
or **Delete round** (only the last round).

### Team events

Each match lists its boards under the match row. **Line-ups…** sets each
team's board order for the selected match. The default is the first
available members in team order. Results already entered are kept for boards
whose players do not change.

## 5. Entering results

Select a board and type:

| Key | Result |
|---|---|
| `1` | 1-0 |
| `0` | 0-1 |
| `5` or `=` | ½-½ |
| `+` / `-` | White / Black wins by forfeit |
| `*` | double forfeit |
| `Delete` | clear |

The cursor moves to the next board. Results of earlier rounds can be
corrected at any time; standings and tie-breaks follow immediately.
**Undo** (⌘⌥Z / Ctrl+Alt+Z) reverts any change, however old the session.

## 6. Lists and printing

**Lists** shows any list for any round: starting rank, alphabetical,
pairings, standings with tie-breaks, crosstable, scorecards, board cards,
prize list, rating report, and for team events team rosters, team pairings
and team standings.

- **Print / PDF…** opens the list in your browser to print or save as PDF.
- **Export…** saves HTML, PDF, CSV or Excel.
- **Language** prints the list in English, French, Arabic, Swahili,
  Portuguese or Spanish, whatever the program's own language is.

## 7. Prizes

**Prizes** › **Add category**: a name (U1800, Women, U16, …), who is eligible
(rating band, sex, year of birth, federation, unrated only) and the prizes
in place order. Categories are awarded in the order listed and each
player receives at most one prize. Winners appear under the categories and
in the **Prize list**.

## 8. Ratings and norms

- **FIDE rating report**: **Export TRF…** writes the TRF-2026 file for the
  federation's rating officer.
- **Rating changes**: expected score, K-factor and change for every rated
  player (FIDE B.02), plus the first-rating estimate for unrated players.
- **Title norms**: choose a player to check the GM, IM, WGM and WIM norm
  requirements (B.01): games, rated and titled opponents, federations,
  average rating and performance. Missing requirements are listed.

## 9. Publishing results online

**Publish** uploads the tournament to a results server running `co-publish`
(see [releasing.md](releasing.md)). Players open the server's address on
their phones and see pairings, standings and the crosstable in any of the
supported languages. Upload again after each round; the program works the
same with or without a connection.

## 10. Exchanging files

- **File › Import TRF…** reads TRF-2026, TRF-16 and bbpPairings files.
- **File › Import from Other Programs…** reads files exported by other
  pairing programs: TRF, or player lists saved as CSV or tab-separated text.
- **File › Export TRF…** writes TRF-2026, including byes requested for
  future rounds, acceleration, forbidden pairs and team records.

## Getting help

**Help › Keyboard Shortcuts** lists every key. Problems and suggestions go
to the project's issue tracker.
