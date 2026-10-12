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
setup-tb-help = Codes follow the TRF-2026 rank order descriptors (e.g. BH/C1 = Buchholz cut 1). A “–” means the value cannot be calculated for that player, e.g. a rating tie-break without rated opponents.
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

player-db-help = Download the monthly FIDE rating list (or import one you saved, as zip, XML or TXT) and add national lists from CSV. Searches then fill in names, titles, ratings and IDs.
player-db-download-fide = Download FIDE list
player-db-connecting = Connecting to ratings.fide.com…
player-db-downloading = Downloading the FIDE list… { $mb } MB
player-db-importing = Importing… { $count } players
player-db-imported = { $count } players imported.
player-db-mapping-title = Import national list
player-db-mapping-help = Choose which column holds each field. Either a full-name column or a surname column is required.
player-db-source-name = List name
player-db-has-header = First row holds column names
player-db-field-name = Full name
player-db-field-surname = Surname
player-db-field-first-name = First name
player-db-field-national-id = National ID
player-db-field-rating = National rating
player-db-import = Import
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
rounds-matches = { $count } matches
rounds-lineups = Line-ups…
rounds-lineups-title = Line-ups
rounds-lineups-help = Boards follow the order chosen here. Results already entered are kept for boards whose players did not change.
rounds-team-keys-hint = Select a board and type 1, 0 or 5 (=) for 1-0, 0-1, ½-½ · + / - forfeits · Delete clears. Match rows show the running score.
rounds-violations-title = These pairings break the rules
rounds-violations-help = Chess Organiser found these problems. Fix them, or pair anyway if the arbiter decided so.
rounds-violations-fix = Go back and fix
rounds-violations-force = Pair anyway

## Lists

list-name-pairings = Pairings
list-name-standings = Standings
list-name-crosstable = Crosstable
list-name-team-pairings = Team pairings
list-name-team-standings = Team standings
lists-language = Language
lists-print = Print / PDF…
lists-format = Export format
lists-export = Export…
lists-round = Round
lists-no-rounds = No round has been paired yet.
lists-print-hint = Print / PDF opens the list in your browser; use its print dialog to print or to save as PDF.

## Teams and prizes

teams-add = Add team
teams-edit = Edit team
teams-name = Team name
teams-captain = Captain
teams-members = Members
teams-board-order = Board order (in the order ticked)
teams-empty = No teams yet.
teams-individual-note = This is an individual tournament; teams here are only used for team prizes.
teams-remove-confirm = Remove team { $name }?
teams-standings = Team standings after round { $round }
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
prizes-winners = Prize winners after round { $round }
prizes-one-each = Each player receives at most one prize, in the order the categories are listed.

## Ratings and publishing

ratings-trf-title = FIDE rating report
ratings-trf-help = Export the tournament as a TRF-2026 file for the federation's rating officer.
ratings-changes-title = Rating changes
ratings-changes-help = Expected scores, K-factors and the rating change of every rated player.
ratings-calculate = Calculate rating changes
ratings-norms-title = Title norms
ratings-check-norms = Check norms
ratings-norms-help = Checks the FIDE B.01 requirements for a GM, IM, WGM or WIM norm over the rounds played so far: games, rated and titled opponents, federations and performance.
ratings-norms-none = No norm applies to this player.
ratings-norm-achieved = norm achieved
ratings-norm-missed = not achieved
ratings-norm-stats = { $games } games · average opponent rating { $aro } · performance { $perf }
ratings-first-rating = first rating { $rating }
publish-help = Upload pairings and standings to a results website so players can follow the tournament on their phones. Everything keeps working offline; uploads happen when a connection is available.
publish-server = Results server
publish-token = Upload token
publish-upload = Upload now
publish-done = Uploaded. Players can follow the tournament online.
publish-live-at = Live at
publish-server-title = Running a results server
publish-server-help = The co-publish program serves the uploaded tournaments. Run it on any server behind HTTPS with an upload token; the address and the token go in the fields above.

## Settings and shortcuts

settings-language = Language
settings-theme = Appearance
settings-translations-note = Printed lists can use a different language from the program: choose it on the Lists screen. Any text not yet translated appears in English.
settings-updates = Updates
settings-check-updates = Check for updates
settings-install-update = Install version { $version } and restart
settings-updates-disabled = This build ({ $version }) does not update itself; download new versions from the project website.
settings-up-to-date = You have the latest version ({ $version }).
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

## Welcome screen

welcome-sample-title = New here? Explore with sample data
welcome-sample-body = Opens a 25-player rapid open with four rounds played (the last one half entered), a late entry, a withdrawal, byes, prize categories and a sample player database. Everything is fictional; try anything, undo anything.
welcome-sample-load = Load sample tournament
welcome-tour = Tour of the program
welcome-features = What Chess Organiser does
welcome-free = free for every federation and club
welcome-shot-rounds = Pairings & results
welcome-shot-rounds-body = Enter results straight from the keyboard: 1, 0 or 5 on each board, + and − for forfeits. The cursor moves to the next board, and every change can be undone.
welcome-shot-players = Players
welcome-shot-players-body = Register players by hand or from your rating list, with titles, federations and ratings. Late entries, withdrawals, requested byes and pauses are handled per round.
welcome-shot-lists = Lists
welcome-shot-lists-body = Starting rank, pairings and standings, ready to print, save as PDF or export, in the language of your choice.
welcome-shot-overview = Overview
welcome-shot-overview-body = See where the tournament stands at a glance and what to do next.
feat-files = One file per tournament
feat-files-body = Saved after every change and safe if the power fails. Works offline; copy or e-mail it like any document. Unlimited undo with a full audit trail.
feat-players = Registration
feat-players-body = Players from the FIDE or national list, late entries with zero- or half-point byes, withdrawals, pauses and FIDE starting-rank order.
feat-results = Fast result entry
feat-results-body = Keyboard-first result entry, corrections in earlier rounds and manual pairing with rule checks for repeats, clubmates and byes.
feat-lists = Lists and printing
feat-lists-body = Pairings, standings, crosstables, scorecards, board cards and prize lists to print, save as PDF or export as HTML, CSV and Excel.
feat-trf = FIDE rating reports
feat-trf-body = Export and import TRF-2026 files for the rating officer and for moving events between programs.
feat-pairing = Automatic FIDE pairings
feat-pairing-body = Dutch and Burstein systems checked against FIDE's endorsed engine, Baku acceleration, Dubov, Lim, Monrad and round robins with Berger tables.
feat-teams = Teams and prizes
feat-teams-body = Team Swiss and team round robin with line-ups and team tie-breaks; prize categories for women, juniors and rating bands.
feat-ratings = Ratings and norms
feat-ratings-body = Every FIDE tie-break, rating changes by FIDE B.02 and GM, IM, WGM and WIM norm checks.
feat-languages = Languages and live results
feat-languages-body = French, Arabic, Swahili, Portuguese and Spanish, and a live results website for players' phones.
sample-replace-confirm = Close the current tournament and open a fresh copy of the sample tournament? Your tournament is saved and stays in Recent tournaments.

setup-note-unverified = Pairings follow this system's rules with the same absolute criteria as the Dutch engine. Unlike the Dutch and Burstein engines, it has no endorsed reference program to be checked against, so review the pairings before publishing them.
setup-note-monrad = Monrad pairs the top unpaired player with the next player below who has not been met yet. It is meant for club events, not FIDE-rated play.
setup-note-team-swiss = Teams are paired with the Dutch criteria on match points (or game points), White on board 1 being the team's colour.
