# Appendix 2 — Phase Workable-State Matrix

| Phase | Workable state after completion | Primary users unblocked |
| --- | --- | --- |
| Phase 1 — Reference accuracy and request semantics | Maintainers can compute and validate release-claimed body positions with documented source provenance, tolerances, and explicit unsupported-mode behavior. | Backend implementers, validation maintainers, chart API users evaluating accuracy, artifact-generation maintainers. |
| Phase 2 — Production compressed artifacts | Maintainers can regenerate, decode, validate, benchmark, and query a deterministic 1500-2500 CE artifact whose measured errors fit the published profile. | Application developers needing offline packaged data; release maintainers validating artifacts. |
| Phase 3 — Compatibility evidence and catalog truthfulness | Maintainers can publish compatibility profiles whose house/ayanamsa coverage, aliases, constraints, custom definitions, and known gaps match implemented and tested behavior. | Astrology application developers comparing interoperability; documentation and release maintainers. |
| Phase 4 — Release hardening and publication | Maintainers can produce a verified release bundle with profiles, reports, checksums, artifact summaries, release notes, audits, and documentation from a clean checkout. | Downstream users, package maintainers, release managers. |

## Workable-state invariants

- Every phase preserves pure Rust default build/test workflows.
- Every phase preserves backend/domain layering.
- Every phase keeps unsupported modes explicit rather than silently approximating them.
- Every phase removes completed tasks from active plan docs instead of accumulating historical progress notes.
