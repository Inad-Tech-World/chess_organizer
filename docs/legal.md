# Legal guardrails

From the project workplan:

* Replicate the *workflow and functions* of existing tournament software,
  never its code, icons, screenshots, manuals, or file layouts obtained by
  reverse-engineering its binaries.
* Use our own name and visual identity (Chess Organiser). Do not market the
  program as "compatible with" another product.
* Reading files that other programs *export* (for migration) is acceptable.
  Get a short legal opinion before shipping that importer
  (`co-io::swiss_manager_import`, Phase 4).
* bbpPairings is Apache-2.0. Porting its approach into this GPL-3.0 project
  is licence-compatible; keep the attribution in `NOTICE`. bbpPairings
  itself is only used as an external test oracle and is never shipped.
* "FIDE" is a trademark. The program is not endorsed by FIDE until the
  endorsement process completes. Until then, describe it as "for non-rated
  and national events".
