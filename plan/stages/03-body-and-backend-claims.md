# Phase 3 — Body and Backend Claim Completion

## Goal

Make every public body and backend capability claim match implemented algorithms, source evidence, and validation results.

## Starting point

Sun through Neptune have VSOP87-style source-backed paths. The Moon and lunar points use a compact Meeus-style baseline. Pluto is approximate/fallback-backed. Ceres, Pallas, Juno, Vesta, and selected custom asteroids have bounded fixture evidence but not broad release-grade support. The release summary now also surfaces the Pluto fallback posture so that release-facing claim boundaries stay aligned with the backend matrix.

## Implementation goals

- Decide Pluto status for the first production release: source-backed, artifact-backed, approximate with strict caveats, or excluded.
- Decide whether full ELP-style lunar coefficients are required before production release, and align lunar node/apogee/perigee claims with implemented formulas.
- Promote selected asteroids only where source coverage, backend support, and validation evidence justify release claims.
- Keep extensible custom/numbered body identifiers without implying generic asteroid coverage.
- Maintain backend capability matrices for supported bodies, ranges, frames, apparentness, observer support, accuracy class, and offline/data requirements.

## Completion criteria

- Backend metadata, CLI summaries, release profiles, and validation reports agree on body coverage and limitations.
- Unsupported or approximate bodies fail or warn through structured, documented policy rather than silent overclaiming.
