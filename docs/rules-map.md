# FIDE rules map

Where each rule source is implemented. Read the current edition from the
FIDE Handbook (handbook.fide.com) and the TEC site (tec.fide.com) before
implementing a phase. Rules change, so record the edition here when a module
is filled in.

| Source | Content | Module | Phase | Status |
|---|---|---|---|---|
| C.04.1 | Basic rules for Swiss systems (no repeat pairings, one PAB per player, …) | `co-pairing::check::common` | 0 | Implemented as warnings |
| C.04.2 | General handling rules (initial ranking, late entries, byes, withdrawals) | `co-core` reducer, `players_sort` command | 0–1 | Initial ranking, late entries, byes, withdrawals done |
| C.04.3 (2025 edition, effective 1 Jul 2025) | FIDE (Dutch) System | `co-pairing::dutch`, `co-pairing::matching` | 1 | Done: port of bbpPairings; 5,009/5,009 oracle events |
| C.04.4.1 | Dubov System | `co-pairing::dubov`, `co-pairing::bracket` | 4 | Done; no reference engine to compare with |
| C.04.4.2 | Burstein System | `co-pairing::burstein` | 4 | Done: port of bbpPairings; matches wherever bbpPairings runs |
| C.04.4.3 | Lim System | `co-pairing::lim`, `co-pairing::bracket` | 4 | Done; no reference engine to compare with |
| C.04.5.1 | Baku acceleration | `co-core::pairing_view::baku_accelerations` | 2 | Done; 500/500 oracle events |
| C.04 team rules | FIDE Team Pairing System (types A/B, MP/GP) | `co-pairing::team_swiss` | 3 | Dutch criteria over teams (MP or GP); type A/B colour rules not distinguished |
| C.05 Annex 1 | Berger tables, double round robin | `co-pairing::berger`, `team_rr` | 2 / 3 | Done; last two rounds of cycle 1 swapped in double RR |
| C.07 (effective 1 Mar 2026) | Tie-breaks; TRF-2026 "Mandatory Tie-Breaks" codes | `co-core::tiebreak_spec`, `co-tiebreak` (individual), `co-tiebreak::team` | 1–3 | Done, incl. Article 16 unplayed rounds |
| B.02 | Rating regulations: expected score, K-factors, rating change | `co-rating` | 2 | Done |
| B.01 | Title regulations: norm requirements | `co-rating::norm_check` | 4 | Done (GM, IM, WGM, WIM) |
| TRF-2026 | Tournament Report File: records 0x2, 1x2, 2x2, 001, NRS, 240, 250, 260; team 310/300/320/330/80x | `co-io::trf` | 0 / 3 | Done; team records 013/310 read and written (310 summary columns left blank), matches rebuilt from games |
| TRF-16 / bbpPairings TRF(x) | Older files and oracle extensions (`XXR`, `XXC`, `BBx`) | `co-io::trf::reader` | 0 | Read |
| TRF-2026 record 192 | Tournament Type Code Table | `co-io::trf::{type_code, parse_type_code}` | 0 | Done |

## Endorsement

FIDE endorsement needs, among other things, a pairing engine that reproduces
the reference engine on random tournaments (the criterion is 4,990 matching
tournaments out of 5,000) and the mandatory tie-breaks. `cargo xtask oracle`
runs this test against bbpPairings, which implements the 2025 Dutch rules
and TRF-2026.
