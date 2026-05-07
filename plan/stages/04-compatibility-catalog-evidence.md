# Phase 4 — Compatibility Catalog Evidence

## Goal

Ensure release compatibility profiles truthfully represent house-system and ayanamsa behavior required by `requirements.md` FR-4/FR-5/FR-6 and `spec/astrology-domain.md`.

## Starting point

The workspace has broad house and ayanamsa catalogs, aliases, descriptor validation, baseline coverage, release-specific entries, custom-definition labels, and representative provenance summaries. Remaining work is evidence depth and truthful status classification, not catalog scaffolding.

## Implementation goals

- Audit release-advertised house systems for formula source, assumptions, aliases, latitude/numerical constraints, and test coverage.
- Audit release-advertised ayanamsas for reference epochs, offsets, formulas, aliases, and provenance.
- Keep descriptor-only, approximate, custom-definition-only, constrained, and unsupported entries distinct from fully implemented built-ins.
- Add golden/reference tests for representative house and ayanamsa outputs, especially latitude-sensitive and alias-sensitive cases.
- Make compatibility-profile verification fail if shipped claims overstate implemented behavior.
- Keep target compatibility catalog language open to future Swiss-Ephemeris-class breadth without breaking public identifiers.

## Completion criteria

Phase 4 is complete when release profiles and verification reports match implemented house/ayanamsa behavior, including aliases, custom definitions, known gaps, and failure modes.

## Out of scope

- Backend source-reader and artifact-generation work.
- Final release packaging mechanics.
