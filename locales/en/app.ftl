# Desktop UI messages. List headings and column names shared with printed
# lists are in lists.ftl.

## General

untitled = Untitled tournament
new-tournament = New tournament
action-open = Open…
action-import-trf = Import TRF…
action-export-trf = Export TRF…
action-create = Create
action-cancel = Cancel
action-close = Close
action-save = Save
action-edit = Edit
action-remove = Remove
action-clear = Clear
action-undo = Undo
action-redo = Redo
action-dismiss = Dismiss
action-move-up = Move up
action-move-down = Move down
undo-what = Undo { $what }
planned-badge = Phase { $phase }
planned-body = planned for Phase { $phase } of the roadmap. The screen is ready; the calculation arrives with that phase.
no-tournament-hint = Open or create a tournament first.
export-done = Saved { $path }
import-warnings = Imported with notes:
import-new-window-hint = Close this tournament first, or use File › Open in New Window.
start-tagline = Free tournament management for Swiss, round-robin and team events — offline, on your own computer.
start-recent = Recent tournaments
status-rounds = Round { $paired } of { $total }
status-withdrawn = Withdrawn from round { $round }
status-late = Joins in round { $round }

## Navigation

nav-label = Sections
nav-overview = Overview
nav-setup = Tournament setup
nav-players = Players
nav-rounds = Pairings & results
nav-lists = Lists
nav-teams = Teams
nav-prizes = Prizes
nav-ratings = Ratings & norms
nav-publish = Publish
nav-player-db = Player database
nav-settings = Settings

## Overview

overview-type = Tournament type
overview-players = Players
overview-active = { $count } active
overview-rounds = Rounds paired
overview-pending = Results missing
overview-next = Next step
overview-add-players = Add players
overview-enter-results = Enter results of round { $round }
overview-pair = Pair round { $round }
overview-final = Final standings

## Tournament types

kind-swiss-dutch = Swiss – FIDE Dutch system
kind-swiss-dubov = Swiss – FIDE Dubov system
kind-swiss-burstein = Swiss – FIDE Burstein system
kind-swiss-lim = Swiss – FIDE Lim system
kind-swiss-monrad = Swiss – simple (Monrad)
kind-rr = Round robin
kind-rr-double = Double round robin
kind-team-swiss = Team Swiss
kind-team-rr = Team round robin
kind-team-rr-double = Team double round robin

## Setup

info-name = Tournament name
info-name-placeholder = e.g. Kampala Open 2026
setup-info = Tournament information
setup-venue = Venue
setup-start-date = Start date
setup-end-date = End date
setup-deputies = Deputy arbiters (comma-separated)
setup-organizer = Organiser
setup-system-title = System and rounds
setup-system = Tournament type
setup-rounds = Number of rounds
setup-rating-type = Rating type
setup-initial-color = Colour of top seed on board 1, round 1
setup-acceleration = Acceleration
setup-acceleration-none = None
setup-acceleration-baku = Baku acceleration
setup-acceleration-manual = Manual (from TRF)
setup-fide-rated = FIDE-rated event
setup-kind-locked = The tournament type cannot change after round 1 has been paired.
setup-auto-pairing = Automatic pairing ({ $system })
setup-points = Points
points-win = Win
points-draw = Draw
points-loss = Loss
points-forfeit-win = Forfeit win
points-forfeit-loss = Forfeit loss
points-pab = Pairing-allocated bye
points-half-bye = Half-point bye
points-full-bye = Full-point bye
points-zero-bye = Zero-point bye / absent
setup-tiebreaks = Tie-breaks (FIDE C.07)
setup-tiebreak = Tie-break
setup-tb-cut = Cut / median
setup-tb-forfeits = Forfeits as played (/P)
setup-tb-add = Add tie-break
setup-tb-help = Codes follow the TRF-2026 rank order descriptors (e.g. BH/C1 = Buchholz cut 1). Each tie-break shows the phase in which its calculation lands; until then the column shows “–”.
setup-forbidden = Forbidden pairings
setup-forbidden-help = Players in the same group are never paired against each other: club-mates in early rounds, family members, or any pair the regulations forbid.
setup-group-name = Group name
setup-group-ranks = Starting ranks, e.g. 3, 17, 24
setup-group-add = Add group
setup-group-default = Group
setup-group-too-small = A group needs at least two players (by starting rank).
setup-team = Team settings
setup-boards = Boards per match
setup-reserves = Reserves
setup-match-win = Match points for a win
setup-match-draw = Match points for a draw
setup-match-loss = Match points for a loss
setup-primary-score = Ranking by
setup-color-preference = Colour preferences
score-match-points = Match points
score-game-points = Game points
color-pref-a = Type A
color-pref-b = Type B
color-pref-none = None
color-white = White
color-black = Black
setup-saved = Settings saved.

## Tie-break names

tb-DE = Direct encounter
tb-BPG = Games played with Black
tb-BWG = Games won with Black
tb-REP = Rounds elected to play
tb-SB = Sonneborn-Berger
tb-ARO = Average rating of opponents
tb-TPR = Tournament performance rating
tb-PTP = Perfect tournament performance
tb-APRO = Average performance rating of opponents
tb-APPO = Average perfect performance of opponents
tb-WIN = Number of wins
tb-WON = Games won over the board
tb-PS = Progressive score
tb-BH = Buchholz
tb-AOB = Average of opponents' Buchholz
tb-FB = Fore Buchholz
tb-KS = Koya system
tb-BC = Board count
tb-TBR = Top board result
tb-BBE = Bottom board elimination
tb-MPvGP = Match points vs game points
tb-EMMSB = Team Sonneborn-Berger (MP × MP)
tb-EMGSB = Team Sonneborn-Berger (MP × GP)
tb-EGMSB = Team Sonneborn-Berger (GP × MP)
tb-EGGSB = Team Sonneborn-Berger (GP × GP)
tb-EDE = Extended direct encounter
tb-EDEBT = EDE + board count + top board
tb-EDEBB = EDE + board count + bottom board
tb-EDET = EDE + top board
tb-EDEB = EDE + bottom board
tb-SSSC = Strength-compensated score
tb-OTHER_SEC = Secondary score

## Players

col-status = Status
player-name = Name (Surname, Firstname)
player-name-placeholder = Surname, Firstname
player-national-id = National ID
player-k-factor = K-factor
sex-male = Male
sex-female = Female
rating-standard = Standard
rating-rapid = Rapid
rating-blitz = Blitz
rating-national = National
players-count = { $count ->
    [one] 1 player
   *[other] { $count } players
}
players-add = Add player
players-add-another = Add and continue
players-edit = Edit player
players-empty = No players yet. Add them one by one, from the player database, or import a TRF file.
players-hint = Enter edits · Insert adds · Delete removes a player who has not played yet.
players-remove-confirm = Remove { $name } from the tournament?
players-withdraw = Withdraw
players-withdraw-from = Not paired from round
players-reenter = Re-enter
players-byes = Byes and pauses…
players-byes-title = Byes for { $name }
players-byes-help = Requested byes are honoured when the round is paired. A zero-point bye also works as a pause.
players-sort = Sort starting rank
players-late-entry = Late entry: joins in round { $round }. Earlier rounds count as
bye-none = Plays
bye-half-points = Half-point bye
bye-zero-points = Zero-point bye (absent)
bye-full-points = Full-point bye

## Player database

player-db-help = Download the monthly FIDE rating list from ratings.fide.com (XML or TXT) and import it here. Searches then fill in names, titles, ratings and FIDE IDs.
player-db-import-fide = Import FIDE rating list…
player-db-import-national = Import national list (CSV)…
player-db-search-placeholder = Search by name or FIDE ID
player-db-no-results = No players found.
player-db-type-to-search = Type at least two letters to search.

## Rounds

round-n = Round { $round }
round-short = R{ $round }
rounds-next-tab = R{ $round } – next
rounds-results-entered = { $entered } of { $total } results entered
rounds-swap = Swap colours
rounds-check = Check pairings
rounds-check-ok = Round { $round }: no rule problems found.
rounds-check-problems = Round { $round } breaks these rules:
rounds-print = Print pairings
rounds-delete = Delete round
rounds-delete-confirm = Delete round { $round } and all its results? You can undo this.
rounds-keys-hint = Type 1, 0 or 5 (=) for 1-0, 0-1, ½-½ · + / - for forfeits · * double forfeit · Delete clears. The cursor moves to the next board.
rounds-pair-title = Pair round { $round }
rounds-auto-help = Pair automatically ({ $system }), or pair by hand below.
rounds-pair-auto = Pair automatically
rounds-manual = Pair manually
rounds-manual-help = Click two players to put them on a board (the first gets White). Players are listed by score.
rounds-give-pab = Give { $name } the pairing-allocated bye
rounds-requested-byes = Requested byes this round
rounds-boards = { $count ->
    [one] 1 board
   *[other] { $count } boards
}
rounds-commit = Pair round { $round }
rounds-unpaired-left = { $count } player(s) still unpaired.
rounds-none = No round to show.
rounds-team-feature = Team pairings and line-ups
rounds-violations-title = These pairings break the rules
rounds-violations-help = Chess Organiser found these problems. Fix them, or pair anyway if the arbiter decided so.
rounds-violations-fix = Go back and fix
rounds-violations-force = Pair anyway

## Lists

list-name-pairings = Pairings
list-name-standings = Standings
list-name-crosstable = Crosstable
lists-language = Language
lists-print = Print / PDF…
lists-format = Export format
lists-export = Export…
lists-round = Round
lists-no-rounds = No round has been paired yet.
lists-print-hint = Print / PDF opens the list in your browser; use its print dialog to print or to save as PDF.

## Teams and prizes

teams-feature = Team pairing, line-ups and team standings
teams-add = Add team
teams-edit = Edit team
teams-name = Team name
teams-captain = Captain
teams-members = Members
teams-board-order = Board order (in the order ticked)
teams-empty = No teams yet.
teams-individual-note = This is an individual tournament; teams here are only used for team prizes.
teams-remove-confirm = Remove team { $name }?
prizes-feature = Prize lists with categories and shared prizes
prizes-add = Add category
prizes-category = Category
prizes-filter = Who is eligible
prizes-prizes = Prizes
prizes-prizes-help = Prizes, one per line, in place order
prizes-empty = No prize categories yet.
prizes-max-rating = Rating at most
prizes-min-rating = Rating at least
prizes-born-from-label = Born in or after (juniors)
prizes-born-until-label = Born in or before (seniors)
prizes-born-from = born ≥ { $year }
prizes-born-until = born ≤ { $year }
prizes-unrated = Unrated only
prizes-women = Women

## Ratings and publishing

ratings-trf-title = FIDE rating report
ratings-trf-help = Export the tournament as a TRF-2026 file for the federation's rating officer.
ratings-changes-title = Rating changes
ratings-changes-feature = Rating-change calculation (FIDE B.02)
ratings-changes-help = Expected scores, K-factors and the rating change of every rated player.
ratings-calculate = Calculate rating changes
ratings-norms-title = Title norms
ratings-norms-feature = GM / IM / WGM / WIM norm checks (FIDE B.01)
ratings-check-norms = Check norms
publish-feature = Live pairings and standings website
publish-help = Upload pairings and standings to a results website so players can follow the tournament on their phones. Everything keeps working offline; uploads happen when a connection is available.
publish-server = Results server
publish-token = Upload token
publish-upload = Upload now

## Settings and shortcuts

settings-language = Language
settings-theme = Appearance
settings-translations-note = French, Arabic, Swahili, Portuguese and Spanish are being translated (Phase 5); untranslated text appears in English.
theme-system = Same as system
theme-light = Light
theme-dark = Dark
shortcuts-title = Keyboard shortcuts
shortcut-white-wins = White wins (1-0)
shortcut-black-wins = Black wins (0-1)
shortcut-draw = Draw (½-½)
shortcut-white-forfeit = White wins by forfeit (+/-)
shortcut-black-forfeit = Black wins by forfeit (-/+)
shortcut-double-forfeit = Double forfeit (-/-)
shortcut-clear-result = Clear the result
shortcut-move = Move between rows
shortcut-edit = Edit the selected row
shortcut-undo = Undo the last tournament change
shortcut-redo = Redo
shortcut-pair = Pair the next round
shortcut-sections = Jump to a section
