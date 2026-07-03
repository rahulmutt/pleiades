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

- All target house systems from `compatibility-catalog.md` are already shipped as
  built-ins: 25 built-in systems with real cusp formulas, 24 numerically gated by
  `validate-houses` (only Albategnius is not yet corpus-backed). The ayanamsa
  catalog holds 59 built-ins with 48 SE-gated by `validate-ayanamsa`; 11 remain
  descriptor-only (6 with no computation path). The `BASELINE_HOUSE_SYSTEMS` (12)
  and `BASELINE_AYANAMSAS` (5) constants still exist as code categories but no
  longer mark the edge of what is implemented or validated.
- Aspects / orb-ready angular separations are implemented in
  `pleiades-core::chart::aspects`; dignities are not.
- Identifier models are already open to additional built-ins and aliases, so
  catalog growth should not require breaking redesign.
- Composite routing helpers exist for hybrid backend composition.

## Remaining implementation work

- Corpus-gate Albategnius (the one built-in house system with a formula but no
  `validate-houses` corpus rows); it is beyond the SE 23-code target, so this is
  optional hardening rather than target-catalog completion.
- Finish the ayanamsa catalog: give the 11 remaining descriptor-only modes (6 of
  which currently have no computation path) real epoch/offset/formula metadata,
  provenance, and `validate-ayanamsa` gating, and add any SE `SE_SIDM_*` modes
  still absent from the 59 built-ins.
- Expand selected-asteroid coverage beyond Ceres/Pallas/Juno/Vesta where source
  evidence and backend metadata support release-grade claims.
- Add dignities from `astrology-domain.md` "Derived Quantities", built above the
  core domain layer. (Aspects / orb-ready angular separations are already
  implemented in `pleiades-core::chart::aspects`.)
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
