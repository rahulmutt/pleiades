# Appendix 1 — Phase to Spec Map

| Active phase | Primary spec coverage |
| --- | --- |
| Phase 1 — Production reference/source corpus | `requirements.md` FR-7/FR-8/NFR-3/NFR-4, `backends.md`, `validation-and-testing.md` reference comparison/release gates |
| Phase 2 — Production compressed ephemeris | `requirements.md` FR-9/NFR-2/NFR-4, `data-compression.md`, `backends.md` packaged-data backend |
| Phase 3 — Body and backend claim completion | `requirements.md` FR-1/FR-2/FR-7/FR-8, `astrology-domain.md` body model, `backend-trait.md` metadata/errors |
| Phase 4 — Advanced request modes | `requirements.md` FR-2/FR-3/FR-7/FR-10, `api-and-ergonomics.md`, `backend-trait.md` request/result/error model |
| Phase 5 — Compatibility and release readiness | `requirements.md` FR-4/FR-5/FR-6/NFR-1/NFR-3/NFR-4, `astrology-domain.md` catalogs, `validation-and-testing.md` release gates |

## Cross-phase coverage

All phases must preserve `architecture.md` layering, the pure-Rust requirement, first-party `pleiades-*` crate naming, deterministic behavior, and explicit release compatibility profiles.
