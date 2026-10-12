# 2. A native Rust FIDE (Dutch) engine, tested against bbpPairings

Status: accepted (2026-10-11); implementation in Phase 1

## Context

Arbiters and federations trust only pairings identical to FIDE-endorsed
programs. FIDE's endorsement test asks for at least 4,990 of 5,000 random
tournaments reproduced exactly. bbpPairings (Apache-2.0) is endorsed and
implements the 2025 Dutch rules (C.04.3) and TRF-2026.

## Decision

Port bbpPairings' method to Rust in `co-pairing::dutch`:

1. Process score brackets in order.
2. For each, build a graph of compatible pairs; encode the C.04.3 quality
   criteria as prioritised bit fields of a wide unsigned edge weight.
3. Solve with maximum-weight matching (Edmonds' blossom, `co-pairing::matching`).
4. Allocate colours by the colour rules; assign the PAB by the 2025 rules.

Run bbpPairings only as an external oracle (`cargo xtask oracle`), never ship
it. If the port slips, the fallback is bundling bbpPairings as a sidecar
binary behind the same `PairingEngine` trait.

## Consequences

* No C++ toolchain for users; one engine for desktop, CLI and server.
* Correctness is measured continuously, not argued.
* Attribution to bbpPairings is kept in `NOTICE`.
