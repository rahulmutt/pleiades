# Appendix 2 — Phase Workable-State Matrix

Each remaining phase must leave the repository in a useful, buildable state even if later phases are not complete.

| Phase | Workable state after completion | Primary users unblocked |
| --- | --- | --- |
| Phase 1 — Production ephemeris accuracy | Maintainers can compute and validate source-backed major-body and lunar positions with documented tolerances and structured unsupported-mode errors. | Backend implementers, validation maintainers, chart API users evaluating accuracy. |
| Phase 2 — Reproducible compressed artifacts | Maintainers can regenerate, decode, validate, benchmark, and query a deterministic 1500-2500 CE artifact. | Application developers needing offline packaged data; release maintainers validating artifacts. |
| Phase 3 — Compatibility catalog completion | Maintainers can publish compatibility profiles whose house/ayanamsa coverage, aliases, and known gaps match implemented and tested behavior. | Astrology application developers comparing interoperability; documentation/release maintainers. |
| Phase 4 — Release stabilization and hardening | Maintainers can produce a verified release bundle with profiles, reports, checksums, artifact summaries, release notes, and audit results from a clean checkout. | Downstream users, package maintainers, release managers. |

## Workable-state invariants

At the end of every phase:

- the workspace builds and tests;
- public claims are no broader than implemented behavior;
- known gaps are documented in profiles or status docs;
- data and generated artifacts are reproducible or explicitly marked as fixtures;
- no new dependency violates the pure-Rust policy;
- the next phase can begin without architectural rework.
