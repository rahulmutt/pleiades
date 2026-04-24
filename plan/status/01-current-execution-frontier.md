# Status 1 — Current Execution Frontier

This document answers a practical question that the stage documents alone do not answer cleanly:

**given the current repository state, what should maintainers focus on next?**

The sequential plan in `plan/stages/` remains the source of truth for overall ordering. This status note sits on top of that sequence and identifies the current frontier.

## Current stage posture

As of 2026-04-23, the project has materially completed the foundational work described in Stages 1 through 5 and is operating inside **Stage 6 — Compatibility expansion and release hardening**.

That means the repository already has:

- a managed Rust workspace and crate layout,
- shared domain and backend contracts,
- an end-to-end chart MVP,
- a narrow source-backed validation path,
- a packaged-data backend and artifact validation flow,
- a release-bundle verifier with duplicate-manifest-entry regression coverage for representative release-artifact fields.

The primary planning problem is no longer "how do we get a first useful product?" It is now:

1. how to close the remaining compatibility gaps without destabilizing the API,
2. how to keep release-facing compatibility claims precise,
3. how to make validation, packaged artifacts, and release bundles routine and auditable.

## Recommended priority order inside Stage 6

Work inside Stage 6 should generally be sequenced in this order:

1. **Release-surface integrity first**
   - compatibility profile stays current
   - validation summaries stay reproducible
   - release bundle verification stays strict, including the canonical manifest checksum sidecar format and regular-file enforcement
   - maintainer docs stay aligned with actual commands
2. **Compatibility breadth second**
   - add remaining house-system and ayanamsa breadth in small, reviewable batches
   - publish aliases, constraints, and caveats in the same change
   - avoid breadth additions that outpace tests or release-profile updates
3. **Backend and distribution refinement third**
   - improve packaged-data coverage and artifact provenance
   - refine fallback/composite routing behavior
   - expand reference-backed slices only where they improve validation or artifact generation
4. **Higher-level chart conveniences last**
   - add optional helpers only after they clearly sit on stable lower-level contracts
   - keep them out of the way of backend/domain layering rules

## What should count as the next meaningful milestone

The next milestone should not be a large "finish Stage 6" umbrella. It should be a small release-hardening increment that leaves the project easier to audit.

Good examples include:

- tightening one release-bundle verification gap,
- completing one coherent compatibility batch with aliases and tests,
- improving one packaged-artifact reproducibility path,
- adding one missing compact release cross-reference that makes the audit path easier to navigate,
- publishing one missing capability or accuracy summary that release consumers need.

## Anti-patterns to avoid at the current frontier

At this point in the project, the highest-risk mistakes are:

- adding compatibility breadth without updating the release compatibility profile,
- adding new release artifacts without documenting how they are generated and verified,
- letting compact summaries drift away from the full reports they summarize,
- adding convenience APIs that hide time scale, apparentness, or reference-frame assumptions,
- expanding packaged-data coverage faster than validation evidence can support.

## Required reading before Stage 6 changes

For any Stage 6 slice, reread:

1. [SPEC.md](../../SPEC.md)
2. [spec/requirements.md](../../spec/requirements.md)
3. [spec/astrology-domain.md](../../spec/astrology-domain.md)
4. [spec/api-and-ergonomics.md](../../spec/api-and-ergonomics.md)
5. [spec/validation-and-testing.md](../../spec/validation-and-testing.md)
6. [plan/stages/06-compatibility-expansion-and-release-hardening.md](../stages/06-compatibility-expansion-and-release-hardening.md)
7. [plan/checklists/01-stage-gates.md](../checklists/01-stage-gates.md)
8. [plan/checklists/02-release-artifacts.md](../checklists/02-release-artifacts.md)

## Definition of progress at the current frontier

A Stage 6 change is real progress when it improves at least one of these without weakening the others:

- shipped compatibility breadth,
- clarity of release-facing coverage/caveats,
- reproducibility of validation and artifacts,
- confidence in public API stability,
- maintainer ability to assemble and verify a release from repository-managed tooling.
