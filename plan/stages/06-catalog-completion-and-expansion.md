# Phase 6 — Target Catalog Completion and Expansion

## Scope note

Phases 1-5 gate the first production release. Phase 6 is the **end-state**
completion work the specification commits to but does not require for the first
release. It is listed here because `SPEC.md`, `spec/requirements.md` FR-4/FR-5,
`spec/compatibility-catalog.md`, and `spec/roadmap.md` Phase 6 treat the full
target compatibility catalog as binding end-state scope that "must not be
narrowed" — so it is remaining spec-required work, not out-of-scope work. It does
not block phases 1-5 and must not broaden public claims before its own evidence
exists.

## Goal

Reach the enumerated end-state target compatibility catalog in
[`spec/compatibility-catalog.md`](../../spec/compatibility-catalog.md) and the
optional higher-level chart utilities in `spec/astrology-domain.md`, without any
public API or enum redesign.

## Current baseline

- The baseline-11 house systems and baseline-5 ayanamsas are present, plus
  broader descriptor catalogs and compatibility-profile reporting.
- Identifier models are already open to additional built-ins and aliases, so
  catalog growth should not require breaking redesign.
- Composite routing helpers exist for hybrid backend composition.

## Remaining implementation work

- Implement the remaining target house systems beyond the baseline 11 enumerated
  in `compatibility-catalog.md` (Equal-from-MC, Whole Sign / Equal from 0° Aries,
  Vehlow, Krusinski, APC, Sripati, Carter, Horizontal, Gauquelin sectors,
  Pullen SD, Pullen SR, Sunshine), each with formula, aliases, latitude/numerical
  constraints, and source provenance.
- Grow the built-in ayanamsa catalog from the baseline 5 toward the full Swiss
  Ephemeris `SE_SIDM_*` set referenced in `compatibility-catalog.md`, each with
  epoch/offset/formula metadata, aliases, near-equivalent handling, and
  provenance.
- Expand selected-asteroid coverage beyond Ceres/Pallas/Juno/Vesta where source
  evidence and backend metadata support release-grade claims.
- Add the optional higher-level chart utilities from `astrology-domain.md`
  "Derived Quantities": aspects and orb-ready angular separations, and dignities,
  built above the core domain layer.
- Expand composite backend routing where it improves accuracy, range, or speed
  without coupling the public API to a single backend family.

## Exit criteria

- Shipped built-in house systems and ayanamsas match the published target catalog
  for the release line, or the release compatibility profile states exactly which
  target entries remain unshipped as known gaps.
- Every newly shipped entry carries formula, provenance, alias, and constraint
  documentation and passes the Phase 5 compatibility audits.
- Optional chart utilities, where shipped, have rustdoc/API examples, tests, and
  truthful release-profile entries; where unshipped, they are not advertised.
