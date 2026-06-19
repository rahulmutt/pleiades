# Status 1 — Current Execution Frontier

## Frontier

Phase 1 is complete. The active frontier is Phase 2 — release-grade compressed
ephemeris. SP1 (dense de440-backed generation source + accuracy baseline) has
landed; SP2 (accuracy tuning) is next.

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
corpus_regen). ARTIFACT_VERSION is now 5 (per-body segment count widened
u16→u32 to hold the dense Moon's ~91k segments); the regenerated artifact is
~201,873 segments / ~49.78 MB. Generation is byte-deterministic, verified by a
kernel-gated reproduce test (`crates/pleiades-data/tests/artifact_regen.rs`).
The constrained asteroid (433-Eros) is re-derived from the committed reference
snapshot (absent from de440 and sb441-n16), constrained to 1900-2100. A
per-body accuracy baseline vs the committed de440-derived hold-out is in
`crates/pleiades-data/src/accuracy_baseline.rs` and exposed via the
`packaged-artifact-accuracy-baseline-summary` CLI command. Measured result:
inner bodies + Sun + Moon are sub-arcsec (essentially exact vs de440); outer
planets are draft-level (Uranus ~156″, Neptune ~90″, Pluto ~62″, Saturn ~11″,
Jupiter ~1.7″). The regeneration corrected a real ~0.54° Moon error the old
snapshot-fit artifact carried. The artifact remains explicitly draft-grade; SP1
measured accuracy but did not enforce thresholds or tune spans/degrees.

## Why this comes first

The specification requires the 1600-2600 CE compressed data product to be
reproducible from public inputs and validated against measured error envelopes.
Phase 1 established those public inputs. SP1 rebased generation onto them and
measured the per-body accuracy envelope. The artifact cannot be promoted to
production-grade until SP2 (tuning) and SP3 (thresholds/budgets) pass.

## Current blockers

1. **Outer-planet accuracy** — SP1 measured draft-level outer-planet error
   (Uranus ~156″, Neptune ~90″, Pluto ~62″, Saturn ~11″, Jupiter ~1.7″); SP2
   must tune per-body spans and polynomial degrees against the measured baseline
   before thresholds can be enforced.
2. **Accuracy thresholds and size/latency budgets** — SP3 must define and
   enforce published accuracy thresholds per body class and channel, and set
   size/latency budgets (draft baseline: ~49.78 MB, decode ~197 ms, single
   lookup ~1.7 ms).
3. **Compatibility evidence** — avoid widening house, ayanamsa, asteroid, Pluto,
   or lunar claims before supporting audits and validation are complete.

## Recommended next slice

SP2 — accuracy tuning: tune per-body segment spans and polynomial degrees
against the measured SP1 baseline, prioritising outer planets, then enforce
published thresholds and size/latency budgets in SP3.

## Parallel-safe work

- Improve compressed-artifact fitting behind draft labels.
- Audit Pluto, lunar theory, and selected-asteroid body-claim boundaries.
- Audit house and ayanamsa descriptor entries against source/provenance evidence.
- Harden release gates that check existing evidence without broadening claims.
