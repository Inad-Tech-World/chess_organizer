# FIDE rules map

Where each rule source is implemented. Read the current edition from the
FIDE Handbook (handbook.fide.com) and the TEC site (tec.fide.com) before
implementing a phase. Rules change, so record the edition here when a module
is filled in.

| Source | Content | Module | Phase | Status |
|---|---|---|---|---|
| C.04.1 | Basic rules for Swiss systems (no repeat pairings, one PAB per player, …) | `co-pairing::check::common` | 0 | Implemented as warnings |
| C.04.2 | General handling rules (initial ranking, late entries, byes, withdrawals) | `co-core` reducer, `players_sort` command | 0–1 | Initial ranking, late entries, byes, withdrawals done |
| C.04.3 (2025 edition, effective 1 Jul 2025) | FIDE (Dutch) System | `co-pairing::dutch`, `co-pairing::matching` | 1 | Stub |
| C.04.4.1 | Dubov System | `co-pairing::dubov` | 4 | Stub |
| C.04.4.2 | Burstein System | `co-pairing::burstein` | 4 | Stub |
| C.04.4.3 | Lim System | `co-pairing::lim` | 4 | Stub |
| C.04.5.1 | Baku acceleration | `co-pairing::baku` | 2 | Stub; manual acceleration (TRF 250) flows through |
| C.04 team rules | FIDE Team Pairing System (types A/B, MP/GP) | `co-pairing::team_swiss` | 3 | Stub; settings modelled |
| C.05 Annex 1 | Berger tables, double round robin | `co-pairing::berger`, `team_rr` | 2 / 3 | Stub |
| C.07 | Tie-breaks; TRF-2026 "Mandatory Tie-Breaks" codes | `co-core::tiebreak_spec` (catalogue), `co-tiebreak` (calculations) | 1–3 | Catalogue and descriptors done; calculations stub |
| B.02 | Rating regulations: expected score, K-factors, rating change | `co-rating` | 2 | Stub |
| B.01 | Title regulations: norm requirements | `co-rating::norm_check` | 4 | Stub |
| TRF-2026 | Tournament Report File: records 0x2, 1x2, 2x2, 001, NRS, 240, 250, 260; team 310/300/320/330/80x | `co-io::trf` | 0 / 3 | Individual records done (read + write); team records read for names and rosters only |
| TRF-16 / bbpPairings TRF(x) | Older files and oracle extensions (`XXR`, `XXC`, `BBx`) | `co-io::trf::reader` | 0 | Read |
| TRF-2026 record 192 | Tournament Type Code Table | `co-io::trf::{type_code, parse_type_code}` | 0 | Done |

## Endorsement

FIDE endorsement needs, among other things, a pairing engine that reproduces
the reference engine on random tournaments (the criterion is 4,990 matching
tournaments out of 5,000) and the mandatory tie-breaks. `cargo xtask oracle`
runs this test against bbpPairings, which implements the 2025 Dutch rules
and TRF-2026.
