# Appendix 1 — Phase to Spec Map

| Phase | Primary spec coverage | Notes |
| --- | --- | --- |
| Phase 1 — Reference accuracy and request semantics | `requirements.md` FR-1, FR-2, FR-3, FR-7, FR-8, NFR-2, NFR-3; `backend-trait.md`; `backends.md`; `api-and-ergonomics.md`; `validation-and-testing.md` | Closes source-backed accuracy gaps, broadens reference evidence, and makes time/observer/apparentness/frame behavior either implemented or explicitly unsupported. |
| Phase 2 — Production compressed artifacts | `requirements.md` FR-8, FR-9, NFR-2, NFR-4; `data-compression.md`; `backends.md`; `validation-and-testing.md` | Turns the prototype packaged-data path into a deterministic 1500-2500 CE artifact with measured fit error and distribution metadata. |
| Phase 3 — Compatibility evidence and catalog truthfulness | `requirements.md` FR-4, FR-5, FR-6, FR-10, NFR-5; `astrology-domain.md`; `api-and-ergonomics.md`; `validation-and-testing.md` | Completes formula/reference evidence, aliases, constraints, custom-definition posture, and release-profile truthfulness for house and ayanamsa catalogs. |
| Phase 4 — Release hardening and publication | `SPEC.md` acceptance summary; `requirements.md` NFR-1 through NFR-6 and FR-6; `api-and-ergonomics.md`; `validation-and-testing.md`; `backends.md`; `data-compression.md` | Packages current evidence into release reports, checksums, documentation, CI gates, and reproducible bundles. |

## Cross-cutting spec obligations

- Pure Rust and crate naming: every phase.
- Layered architecture and backend/domain separation: every phase.
- Backend modularity and capability metadata: Phases 1-2 and Phase 4.
- Release compatibility profiles: Phases 3-4.
- Deterministic behavior and reproducibility: Phases 1, 2, and 4.
- Truthful known-gap reporting: every phase.
