# Appendix 2 — Phase Workable-State Matrix

| Phase | Workable state |
| --- | --- |
| 1 — Production reference backend and corpus | Maintainers can reproduce or verify broad public reference inputs and know exactly which bodies, epochs, frames, channels, and evidence classes they validate. |
| 2 — Release-grade compressed ephemeris | Maintainers can regenerate and ship a 1600-2600 CE artifact whose measured errors and performance match the published profile. |
| 3 — Body/backend claim closure | Public body/backend claims are source-backed, artifact-backed, constrained, approximate, or unsupported with no ambiguous middle state. |
| 4 — Request-mode semantics | Every advanced request mode is implemented with documented assumptions and tests or rejected with a structured error. |
| 5 — Compatibility and release gates | A clean checkout can build, validate, benchmark, bundle, and verify a release without stale claims or hidden tooling. |
| 6 — Target catalog completion and expansion | Maintainers can ship additional `compatibility-catalog.md` house/ayanamsa entries and optional chart utilities incrementally; each release profile states exactly which target entries ship and which remain known gaps. |

## Notes

Phase numbers describe dependency order, not strict serialization. Work may
proceed in parallel when it does not broaden public claims before evidence
exists.
