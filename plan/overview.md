# Plan Overview

`pleiades` has completed bootstrap and foundation work. The active plan tracks
only remaining production work against the specification.

## Active phases

1. **Production reference backend and corpus** — provide broad, reproducible
   public reference inputs for validation and artifact generation.
2. **Release-grade compressed ephemeris** — replace the draft packaged-data
   fixture with a 1500-2500 CE artifact that passes published thresholds.
3. **Body/backend claim closure** — settle release claims for Pluto, lunar
   theory/lunar points, selected asteroids, and backend capability metadata.
4. **Request-mode semantics** — implement or consistently reject UTC/Delta-T,
   apparent-place, topocentric, native-sidereal, and motion-output requests.
5. **Compatibility and release gates** — audit house/ayanamsa evidence and make
   release validation fail on stale or overstated claims.
6. **Target catalog completion and expansion** — end-state, post-first-release:
   ship the remaining `compatibility-catalog.md` house systems and ayanamsas and
   the optional chart utilities without API redesign.

## Current priority

Phase 1 remains the execution frontier. The repository has many release-facing
summary and verification surfaces, but the underlying reference corpus is still a
checked-in fixture set rather than a broad production source corpus or general
JPL/public-data reader.

## Cross-cutting rules

- Preserve pure-Rust, layered crate boundaries from `spec/architecture.md`.
- Keep unsupported modes as structured errors until implemented and validated.
- Keep release compatibility profiles truthful about exact built-ins, aliases,
  constraints, and known gaps.
- Generate validation/report artifacts from current code and source inputs, not
  manually-maintained prose.
