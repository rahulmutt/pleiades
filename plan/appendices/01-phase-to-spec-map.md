# Appendix 1 — Phase to Spec Map

| Phase | Primary spec coverage | Notes |
| --- | --- | --- |
| 1 — Artifact accuracy and packaged-data production | `requirements.md` FR-8, FR-9, NFR-2, NFR-4; `data-compression.md`; `backends.md`; `validation-and-testing.md` | Turns the draft packaged-data fixture into a validated 1500-2500 CE artifact. |
| 2 — Reference/source corpus productionization | `requirements.md` FR-1, FR-2, FR-7, FR-8, NFR-3, NFR-4; `backend-trait.md`; `backends.md`; `validation-and-testing.md` | Provides documented public source inputs for validation and artifact fitting. |
| 3 — Body-model completion and claim boundaries | `requirements.md` FR-1, FR-2, FR-7, FR-8; `astrology-domain.md`; `backends.md` | Keeps Pluto, lunar points, lunar theory, and selected asteroid claims aligned with evidence. |
| 4 — Advanced request modes and policy | `requirements.md` FR-2, FR-3, FR-7, FR-10; `api-and-ergonomics.md`; `backend-trait.md` | Resolves UTC/Delta-T, apparent, topocentric, and native-sidereal behavior. |
| 5 — Compatibility catalog evidence | `requirements.md` FR-4, FR-5, FR-6, FR-10; `astrology-domain.md`; `api-and-ergonomics.md` | Keeps house/ayanamsa claims, aliases, custom definitions, constraints, and known gaps truthful. |
| 6 — Release gate hardening | `SPEC.md` acceptance summary; `requirements.md` NFR-1 through NFR-6 and FR-6; `validation-and-testing.md` | Makes generated release evidence reproducible and blocking. |

## Cross-phase coverage

- Pure Rust and crate layering: all phases.
- Backend modularity and capability metadata: Phases 2-4 and 6.
- Release compatibility profiles: Phases 3, 5, and 6.
- Deterministic generation and reproducibility: Phases 1, 2, and 6.
