# Phase 5 — Compatibility Catalog Evidence

## Goal

Ensure house-system and ayanamsa release claims satisfy `requirements.md` FR-4/FR-5/FR-6 and `spec/astrology-domain.md`.

## Starting point

The workspace has broad house and ayanamsa catalogs, aliases, descriptor validation, custom-definition handling, release-profile summaries, representative provenance surfaces, golden/reference tests for a latitude-sensitive topocentric house snapshot and baseline ayanamsa epoch offsets, and now validated report wrappers for the house-formula-family, latitude-sensitive house-system, house-code alias inventory, custom-definition ayanamsa sections, direct descriptor summary surfaces, and the compatibility-caveats/release-notes surfaces that expose those catalog slices. Remaining work is evidence depth and truthful status classification, not catalog scaffolding.

## Implementation goals

- Audit release-advertised house systems for formula source, assumptions, aliases, latitude constraints, numerical failure modes, and tests.
- Audit release-advertised ayanamsas for reference epochs, offsets, formulas, aliases, equivalence claims, and provenance.
- Keep descriptor-only, constrained, approximate, custom-only, and unsupported entries distinct from fully implemented built-ins.
- Extend validated summary wrappers to any remaining release-facing descriptor or alias surfaces that do not yet fail closed; the house-code alias inventory summary now validates before rendering, and the catalog inventory summary now also validates the release-profile identifiers before rendering.
- Completed: representative golden/reference tests now pin the latitude-sensitive topocentric house snapshot and baseline ayanamsa epoch offsets, complementing the existing alias-resolution coverage.
- Completed: compatibility-profile verification now also rejects stale release house-system canonical names alongside the existing ayanamsa and alias drift checks, keeping the release-specific catalog summary fail-closed.
- Make compatibility-profile verification fail on overstated catalog claims.
- Keep target-catalog identifiers extensible so future Swiss-Ephemeris-class breadth does not require public API redesign.

## Completion criteria

Phase 5 is complete when release profiles and verification reports accurately describe implemented house/ayanamsa behavior, aliases, constraints, custom definitions, known gaps, and unsupported entries.

## Out of scope

- Backend source-reader work.
- Artifact fitting or distribution.
- Release packaging mechanics.
