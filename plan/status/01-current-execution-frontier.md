# Status 1 — Current Execution Frontier

## Frontier

The active frontier is **Phase 1: Production compressed data**, with **Phase 2: Production reference inputs** as its main dependency.

The previous reference-surface and release-rehearsal cleanup work is treated as complete and is no longer listed as active implementation work.

## Why this frontier comes first

The current packaged-data artifact is reproducible and inspectable, but it is still a draft fixture with a calibrated fit envelope rather than a production-quality one. A first fit-quality slice landed on the Moon, a follow-on body-slice landed on mixed-order quadratic windows with longitude unwrapping across the bundled bodies, and the packaged-artifact reports now show a materially improved fit envelope while still exposing body/channel worst-segment intervals for the remaining work. A follow-on manifest slice has also landed: regeneration provenance now records encoded artifact size so the size accounting stays reproducible. The specification still requires a compressed 1500-2500 CE artifact with published accuracy measurements, deterministic generation, efficient random access, and clear stored/derived/unsupported output semantics.

Production artifacts also depend on trustworthy reference inputs. If existing fixtures cannot support the required fitting and validation thresholds, Phase 2 work should expand or replace them before artifact claims broaden.

## Immediate blockers

1. **Artifact fit quality** — current fit envelope improved substantially after the quadratic-window slice, but the remaining latitude/distance outliers still need to come down before production sign-off.
2. **Generation inputs** — checked-in JPL snapshots are useful evidence but may not be broad enough for production fitting over the advertised range and body set.
3. **Body claim boundaries** — Pluto, fuller lunar theory, lunar points, and selected asteroids must stay constrained or excluded unless source-backed validation supports stronger claims.
4. **Release claim enforcement** — release gates must fail closed on artifact threshold failures and stale compatibility/backend summaries.

## Recommended next slice

Implement a small artifact-quality slice:

- choose one body class or one high-error segment family;
- improve segment fitting, polynomial order, residual handling, or sample density;
- keep fit-threshold validation in lockstep with the calibrated draft artifact;
- keep the draft/prototype label until all advertised scopes pass;
- add or update regression tests that prevent fit-report drift.

Status update: the Moon residual-correction slice is complete, the bundled-body quadratic-window slice is complete, and the current draft thresholds are calibrated to the latest artifact. Next work should target the remaining high-error body classes or segment families.

## Parallel safe work

- Expand documented public reference inputs needed for artifact fitting.
- Audit house/ayanamsa entries whose release claims are stronger than their evidence.
- Keep request-policy docs and structured unsupported errors synchronized.
- Harden release-bundle checks without changing feature claims.
