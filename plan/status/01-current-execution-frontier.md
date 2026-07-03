# Status 1 — Current Execution Frontier

## Frontier

Phases 1 and 2 are complete. The active frontier is **Phase 3 — body/backend
claim closure**.

Phase 2 is fully done: SP1 (dense de440-backed generation + accuracy baseline),
SP2 (heliocentric-planet reframe, all bodies sub-arcsec), and SP3 (published
accuracy thresholds + size/latency budgets + motion-derived output) have all
landed. The packaged artifact (ARTIFACT_VERSION 7, 1900–2100 CE) passes
per-body-class accuracy ceilings, the hard size gate (≤ 12 MB), and speed
ceilings; latency targets are tracked in `PACKAGED_BUDGETS`.

Phase 1 delivered: a real, broad, de440-sourced reference corpus (~25,659 data
rows across boundary, interior, fast-cluster, hold-out, and independent
fixture-golden slices) committed under `crates/pleiades-jpl/data/corpus/`, with
real non-zero checksums, a pinned kernel SHA-256, and a live fail-closed
`validate-corpus` gate. The broad public-data reader (`pleiades-jpl::ingest`:
Horizons vector-table, API JSON, generic CSV; optional live fetch behind
`horizons-fetch`) and the curated asteroid corpus (Tier A main-belt core
reproducible from `sb441-n16`, Tier B constrained set from Horizons over
1900-2100) are committed and gate-verified.

SP1 delivered: artifact generation is now rebased on a dense, de440-backed
within-span fit. Each of the 10 major bodies (Sun, Moon, Mercury-Pluto) is fit
by least-squares polynomials sampled densely from de440 within each per-body
segment span, kernel-gated behind `PLEIADES_DE_KERNEL` (same gate as
corpus_regen). Generation is byte-deterministic, verified by a kernel-gated
reproduce test (`crates/pleiades-data/tests/artifact_regen.rs`). The constrained
asteroid (433-Eros) is re-derived from the committed reference snapshot (absent
from de440 and sb441-n16), constrained to 1900-2100. A per-body accuracy
baseline vs the committed de440-derived hold-out is in
`crates/pleiades-data/src/accuracy_baseline.rs` and exposed via the
`packaged-artifact-accuracy-baseline-summary` CLI command.

SP2 delivered: heliocentric-planet reframe — outer-planet longitude errors
reduced from ~156″ (Uranus), ~90″ (Neptune), ~62″ (Pluto), ~11″ (Saturn),
~1.7″ (Jupiter) to sub-arcsec across all major bodies. ARTIFACT_VERSION bumped
to 6 (`StoredFrame` byte added to codec).

SP3 delivered: published per-body-class accuracy ceilings (thresholds.rs),
hard size gate (≤ 12,000,000 bytes), latency targets tracked in
`PACKAGED_BUDGETS`, and motion output (`SpeedPolicy::FittedDerivative`,
`Motion = Derived`) implemented and gated. ARTIFACT_VERSION is now 7.
Current packaged artifact: ~10.0 MB, 1900–2100 CE, all bodies pass published
accuracy ceilings.

## Why Phase 3 is next

Phase 1 established validated, reproducible public reference inputs. Phase 2
promoted the packaged artifact to release-grade with enforced accuracy, size, and
speed thresholds (SP1 + SP2 + SP3 complete). Phase 3 resolves remaining body
and backend claim ambiguity (Pluto, lunar theory limits, asteroid scope) before
broader release claims can be made.

## Current blockers

1. **Compatibility evidence** — avoid widening house, ayanamsa, asteroid, Pluto,
   or lunar claims before supporting audits and validation are complete.
2. **Body/backend claim closure (Phase 3)** — Pluto remains approximate/fallback
   in first-party algorithmic paths; lunar theory is compact Meeus-style baseline;
   broad asteroid release claims are not yet supported.

## Recommended next slice

Phase 3 — body/backend claim closure: resolve Pluto, constrain lunar/lunar-point
claims to the compact baseline, and promote selected asteroid support where source
evidence is sufficient.

## Phase 5 note

The Phase 5 compatibility-audit pair has landed: house-system numeric gate
(`validate-houses`, 138-row SE corpus over 6 charts × 23 systems, per-formula-family
ceilings set from measured residuals — tightest families ≤ 1–2″, Quadrant ≤ 12″,
SolarArc/Sunshine ≤ 66″ at the lat-66° bound) and ayanamsa numeric gate
(`validate-ayanamsa`, 480-row SE mean corpus, 48 gated modes across 4 classes —
OffsetDefined ≤ 3.0″, TrueStar/Galactic/FittedOffset ≤ 1.0″) are both done.
Remaining Phase 5 work: release-gate hardening and compatibility-profile
overclaim checks.

## Parallel-safe work

- Audit Pluto, lunar theory, and selected-asteroid body-claim boundaries (Phase 3).
- Harden release gates that check existing evidence without broadening claims.
