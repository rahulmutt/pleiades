# Appendix 1 — Phase to Spec Map

This appendix maps remaining implementation phases to the normative specification set.

| Phase | Primary spec coverage | Notes |
| --- | --- | --- |
| Phase 1 — Production ephemeris accuracy | `requirements.md` FR-1, FR-2, FR-3, FR-7, FR-8, NFR-2, NFR-3; `backend-trait.md`; `backends.md`; `validation-and-testing.md` | Converts preliminary/sample backends into validated source-backed implementations with clear time, frame, apparentness, and error semantics. |
| Phase 2 — Reproducible compressed artifacts | `requirements.md` FR-8, FR-9, NFR-2, NFR-4; `data-compression.md`; `backends.md`; `validation-and-testing.md` | Builds deterministic 1500-2500 CE artifacts and a packaged backend whose claims are backed by measured fit errors. |
| Phase 3 — Compatibility catalog completion | `requirements.md` FR-4, FR-5, FR-6, FR-10, NFR-5; `astrology-domain.md`; `api-and-ergonomics.md`; `validation-and-testing.md` | Completes and validates house/ayanamsa catalogs, aliases, failure modes, and release-profile truthfulness. |
| Phase 4 — Release stabilization and hardening | `SPEC.md` acceptance summary; `requirements.md` NFR-1 through NFR-6; `api-and-ergonomics.md`; `validation-and-testing.md`; `backends.md` | Packages evidence into release profiles, reports, checksums, documentation, CI gates, and reproducible bundles. |

## Cross-cutting spec obligations

- Pure Rust/no mandatory C/C++ dependencies: all phases.
- First-party crate prefix and layering: all phases.
- Backend modularity and domain/backend separation: Phases 1-3.
- Release compatibility profiles: Phases 3-4.
- Deterministic behavior and reproducibility: Phases 1, 2, and 4.
