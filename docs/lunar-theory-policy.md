# Lunar Theory Policy

Status: current pre-source-selection baseline.

Pleiades' `pleiades-elp` crate currently uses a compact pure-Rust analytical
lunar baseline derived from published lunar element and mean-point formulas.
It is intentionally explicit about its present scope so later source-backed ELP
work can replace it without redesigning the public API.

## Current baseline

The backend currently covers:

- the Moon
- mean node
- true node
- mean apogee
- mean perigee

The backend currently rejects:

- true apogee
- true perigee
- apparent requests
- topocentric observer requests
- non-TT input time scales
- non-tropical zodiac requests

## Provenance and license posture

- No vendored ELP coefficient files are used yet.
- The implementation is handwritten pure Rust, using published lunar element
  and mean-point formulas as the current baseline.
- The planned full ELP selection remains pending; when it lands, this policy
  should be updated with the chosen source identifier, provenance notes, and
  any redistribution constraints.

## Validation posture

The current regression posture is intentionally small and deterministic:

- canonical J2000 checks for the Moon and lunar points
- nearby high-curvature lunar-interval regression coverage
- explicit unsupported-body and unsupported-mode errors

## Forward path

When a source-backed lunar-theory selection is added, this document should be
updated to describe:

- the selected coefficient/source family
- supported channels and date range
- any data-file or license constraints
- the measured error envelope used by validation and release reports
