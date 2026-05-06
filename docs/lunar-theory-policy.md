# Lunar Theory Policy

Status: current first-release baseline.

Pleiades' `pleiades-elp` crate currently uses a compact pure-Rust truncated
lunar baseline derived from published lunar position, node, and mean-point
formulas. The first release keeps that compact posture intentionally explicit,
so later source-backed ELP work can replace it without redesigning the public
API.

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
- The current catalog also has a backend-owned `validate_lunar_theory_catalog()` helper that checks the selected-entry round trip plus case-insensitive uniqueness for identifiers, model names, family labels, and documented aliases, which keeps the current one-entry baseline honest while future source-backed catalog variants are added. A structured `lunar_theory_catalog_validation_summary()` helper and formatter now expose that validation state as typed data so backend-owned reporting can reuse the same catalog snapshot without rebuilding it from prose.
- The current source-selection fields are also grouped structurally via `LunarTheorySpecification::source_selection()`, and the crate now exposes `lunar_theory_source_selection()` as a free accessor for the currently selected record, which keeps the family, identifier, citation, provenance, redistribution, and license posture available as one typed record for backend-owned reporting. That same selection record now has a compact `summary_line()` / `Display` helper too, and the rendered line now includes the typed selected and family keys alongside the source identifier and family label, which gives future source-backed lunar-theory variants a stable typed string without reopening the longer validation-window summary. The selected source family is also surfaced separately through `lunar_theory_source_family_summary()` / `lunar_theory_source_family_summary_for_report()`, which keeps the Meeus-style family label available as its own typed release-facing line. The catalog layer also exposes `lunar_theory_catalog_entry_for_selection(...)` so callers can round-trip from a typed source selection back to the corresponding catalog entry without re-deriving the lookup key.
- Typed lookup helpers now exist for source identifier, model name, family label, and the documented short alias `Meeus-style truncated lunar baseline`; the crate now exposes `resolve_lunar_theory_by_key(...)` for callers that want to state the lookup intent explicitly, and `resolve_lunar_theory_by_alias(...)` for the short alias path without relying only on the generic matcher.
- A compact `lunar_theory_source_summary()` helper and `lunar_theory_source_summary_for_report()` formatter expose the same source-selection record as a shorter release-facing provenance line when reports do not need the full specification string; the compact line now also carries the current validation window so the evidence span stays visible without opening the longer specification text. The report-facing formatter validates the summary against the current lunar selection before rendering it, so future drift in the compact provenance fields will surface as an unavailable report line instead of a stale summary. The structured source-family enum is carried directly on the summary record so future source-backed variants can branch on typed data instead of reparsing labels. The source-selection, catalog-validation, and capability summary records also implement `summary_line()` and `Display`, so backend-owned reporting can render the typed summaries directly without reassembling the strings in the tooling layer.
- The compact `lunar_theory_capability_summary()` helper now includes the supported and unsupported lunar body lists alongside the policy counts, plus a catalog-validation status flag, so release-facing capability snapshots can show the exact channels covered by the current baseline and whether the one-entry catalog still round-trips cleanly without parsing the full specification string.
- The compact `lunar_theory_limitations_summary()` helper now inlines the current reference and equatorial error-envelope report strings, so the release-facing limitations line publishes the measured channel evidence directly instead of only naming the two envelope surfaces.
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
- A future full ELP selection remains available as follow-on work; when it
  lands, this policy should be updated with the chosen source identifier,
  citation, provenance notes, and any redistribution constraints.

## Validation posture

The current regression posture is intentionally small and deterministic:

- the published 1992-04-12 geocentric Moon example
- the published 1992-04-12 geocentric Moon RA/Dec example used to cross-check the shared mean-obliquity equatorial transform, now surfaced as a dedicated equatorial evidence summary in validation and backend-matrix reports
- a reference-only published 1992-04-12 apparent geocentric Moon comparison datum, surfaced so the current mean/apparent gap remains explicit while apparent requests are still rejected
- a reference-only published 1992-04-12 apparent geocentric Moon comparison datum now broadens the equatorial evidence slice, so the current validation posture carries an extra equatorial cross-check alongside the J2000-adjacent companion pair
- the release-facing date-range note also names the reference-only 1968-12-24 apparent geocentric Moon comparison datum so the compact lunar provenance line reflects the broader validation slice visible in reporting
- canonical J2000 checks for the Moon and lunar points
- release-facing lunar error envelopes for the reference and equatorial channels, surfaced through dedicated `lunar-reference-error-envelope-summary` and `lunar-equatorial-reference-error-envelope-summary` inspection commands
- published 1913-05-27 true-ascending-node and mean-ascending-node examples
- a published 1959-12-07 mean-ascending-node example
- a published 2021-03-05 mean-perigee example
- nearby high-curvature lunar-interval regression coverage, now surfaced as a dedicated continuity evidence slice in validation and backend-matrix reports, using a denser six-sample half-day window around J2000 for a slightly tighter continuity check
- explicit unsupported-body and unsupported-mode errors
- a structured validation window exposed as `LunarTheorySpecification.validation_window` alongside the prose range note
- a compact capability summary helper exposed as `lunar_theory_capability_summary()` for report generators that want structured counts and policy flags without parsing prose
- the current one-entry catalog and its validation state are also exposed through `lunar_theory_catalog_summary()` / `lunar_theory_catalog_summary_for_report()` and `lunar_theory_catalog_validation_summary()` / `lunar_theory_catalog_validation_summary_for_report()`, so release-facing tooling can inspect the lunar baseline catalog directly
- the selected source family is carried directly on `LunarTheorySpecification.source_family` so the current baseline can be audited without parsing the one-line summary

## Forward path

When a source-backed lunar-theory selection is added, this document should be
updated to describe:

- the selected coefficient/source family
- supported channels and date range
- any data-file or license constraints
- the measured error envelope used by validation and release reports
