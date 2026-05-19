# Phase 5 — Compatibility Catalog Evidence

## Goal

Ensure house-system and ayanamsa release claims satisfy `requirements.md` FR-4/FR-5/FR-6 and `spec/astrology-domain.md`.

## Starting point

The workspace has broad house and ayanamsa catalogs, aliases, descriptor validation, custom-definition handling, release-profile summaries, representative provenance surfaces, golden/reference tests for a latitude-sensitive topocentric house snapshot and baseline ayanamsa epoch offsets, and now validated report wrappers for the house-formula-family, latitude-sensitive house-system, house-code alias inventory, custom-definition ayanamsa sections, direct descriptor summary surfaces, and the compatibility-caveats/release-notes surfaces that expose those catalog slices. Remaining work is evidence depth and truthful status classification, not catalog scaffolding.

## Implementation goals

- Audit release-advertised house systems for formula source, assumptions, aliases, latitude constraints, numerical failure modes, and tests.
- Audit release-advertised ayanamsas for reference epochs, offsets, formulas, aliases, equivalence claims, and provenance.
- Keep descriptor-only, constrained, approximate, custom-only, and unsupported entries distinct from fully implemented built-ins.
- Extend validated summary wrappers to any remaining release-facing descriptor or alias surfaces that do not yet fail closed; the house-code alias inventory summary now validates before rendering, the catalog inventory summary now also validates the release-profile identifiers before rendering, the release-profile identifiers summary now routes through a shared validated helper, and the core façade now exposes a validated catalog-inventory helper for report surfaces.
- Completed: representative golden/reference tests now pin the latitude-sensitive topocentric house snapshot and baseline ayanamsa epoch offsets, complementing the existing alias-resolution coverage.
- Completed: compatibility-profile verification now also rejects stale release house-system canonical names alongside the existing ayanamsa and alias drift checks, keeping the release-specific catalog summary fail-closed.
- Completed: the release-specific house-system and ayanamsa canonical-name summary commands now route through explicit validated helpers before formatting, so the public release-facing aliases fail closed if the catalog posture drifts.
- Completed: the core façade now also exports matching release-specific house-system and ayanamsa canonical-name report helpers, and the validation surfaces reuse those public wrappers so the release-facing catalog summaries share one public path.
- Completed: the house-code alias inventory now also routes through a shared validated report helper in core, and the backend-matrix report path uses it directly so the alias surface fails closed without redoing the current-profile validation plumbing in each report layer.
- Completed: the compatibility-profile and release-summary renderers now re-validate the target-house and target-ayanamsa scope prose before emitting release-facing text, so the rendered catalog posture fails closed if those scope sections drift.
- Completed: the compatibility-profile verification summary now exposes a validated compact summary-line helper, so direct release-facing verification rendering fails closed instead of relying on the raw formatter.
- Completed: the ayanamsa provenance summary now validates before rendering in release-facing report surfaces, so representative provenance examples fail closed if the sample notes drift.
- Completed: the ayanamsa reference offsets summary now also routes through a validated report helper, so the zero-point evidence line fails closed before the release-facing summary is rendered.
- Completed: compatibility-profile verification now requires the canonical release summary text exactly, so overstated catalog claims fail closed even when they reuse the baseline/release split wording.
- Keep target-catalog identifiers extensible so future Swiss-Ephemeris-class breadth does not require public API redesign.

## Completion criteria

Phase 5 is complete when release profiles and verification reports accurately describe implemented house/ayanamsa behavior, aliases, constraints, custom definitions, known gaps, and unsupported entries.

## Out of scope

- Backend source-reader work.
- Artifact fitting or distribution.
- Release packaging mechanics.
