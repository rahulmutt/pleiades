# Lunar Theory Policy

Status: current pre-source-selection baseline.

Pleiades' `pleiades-elp` crate currently uses a compact pure-Rust truncated
lunar baseline derived from published lunar position, node, and mean-point
formulas. It is intentionally explicit about its present scope so later
source-backed ELP work can replace it without redesigning the public API.

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
- apparent requests beyond mean geometric output
- topocentric observer requests
- non-TT/TDB input time scales
- non-tropical zodiac requests

The current request policy is intentionally narrow and explicit, and the same shape is surfaced in `pleiades-elp::LunarTheorySpecification.request_policy` so validation and release reports can read it without inferring policy from prose:

- supported coordinate frames: ecliptic and equatorial
- supported time scales: TT and TDB
- supported zodiac modes: tropical only
- supported apparentness: mean only
- topocentric observer support: false

## Provenance and license posture

- Source family: Meeus-style truncated analytical baseline, surfaced structurally as `LunarTheorySpecification.source_family` and rendered via the typed `LunarTheorySourceFamily` display label in release summaries.
- The current selection is also exposed as a one-entry catalog via `lunar_theory_catalog()` and `lunar_theory_catalog_summary()`, so future source-backed lunar-theory variants can be added without changing the reporting shape.
- The current source-selection fields are also grouped structurally via `LunarTheorySpecification::source_selection()`, which keeps the family, identifier, citation, provenance, redistribution, and license posture available as one typed record for backend-owned reporting.
- A compact `lunar_theory_source_summary()` helper and `lunar_theory_source_summary_for_report()` formatter expose the same source-selection record as a shorter release-facing provenance line when reports do not need the full specification string.
- Source identifier: `meeus-style-truncated-lunar-baseline`.
- Canonical citation: Jean Meeus, *Astronomical Algorithms*, 2nd edition,
  truncated lunar position and lunar node/perigee/apogee formulae adapted into
  a compact pure-Rust baseline.
- No vendored ELP coefficient files are used yet.
- The implementation is handwritten pure Rust, using published lunar position,
  node, and mean-point formulas as the current baseline.
- The current truncation policy is explicit: the baseline only covers the Moon,
  mean/true node, and mean apogee/perigee channels that validation exercises;
  it is not a full ELP coefficient selection.
- The current output units are explicit: angular values are reported in
  degrees and distance values, when present, are reported in astronomical
  units.
- The current redistribution posture is simple: there are no external
  coefficient-file redistribution constraints to track until a source-backed
  ELP selection is adopted.
- The current license/provenance note is intentionally conservative: the crate
  does not redistribute external coefficient tables, but any future source-
  backed lunar theory selection should document its own licensing and
  provenance review before it is treated as production data.
- The planned full ELP selection remains pending; when it lands, this policy
  should be updated with the chosen source identifier, citation, provenance
  notes, and any redistribution constraints.

## Validation posture

The current regression posture is intentionally small and deterministic:

- the published 1992-04-12 geocentric Moon example
- the published 1992-04-12 geocentric Moon RA/Dec example used to cross-check the shared mean-obliquity equatorial transform, now surfaced as a dedicated equatorial evidence summary in validation and backend-matrix reports
- a reference-only published 1992-04-12 apparent geocentric Moon comparison datum, surfaced so the current mean/apparent gap remains explicit while apparent requests are still rejected
- canonical J2000 checks for the Moon and lunar points
- published 1913-05-27 true-ascending-node and mean-ascending-node examples
- a published 1959-12-07 mean-ascending-node example
- a published 2021-03-05 mean-perigee example
- nearby high-curvature lunar-interval regression coverage, now surfaced as a dedicated continuity evidence slice in validation and backend-matrix reports
- explicit unsupported-body and unsupported-mode errors
- a structured validation window exposed as `LunarTheorySpecification.validation_window` alongside the prose range note
- a compact capability summary helper exposed as `lunar_theory_capability_summary()` for report generators that want structured counts and policy flags without parsing prose
- the selected source family is carried directly on `LunarTheorySpecification.source_family` so the current baseline can be audited without parsing the one-line summary

## Forward path

When a source-backed lunar-theory selection is added, this document should be
updated to describe:

- the selected coefficient/source family
- supported channels and date range
- any data-file or license constraints
- the measured error envelope used by validation and release reports
