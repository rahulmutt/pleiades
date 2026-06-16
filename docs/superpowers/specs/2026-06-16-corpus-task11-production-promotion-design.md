# Corpus Task 11 — Production Promotion — Design

**Date:** 2026-06-16
**Status:** Approved design, pending implementation plan
**Phase:** 1 — Production reference backend and corpus (active frontier)

## Summary

Promote the checked-in reference corpus from a **3-row scaffold** to a **real,
broad, validated production corpus** generated from the public `de440.bsp`
kernel. This is the "Task 11" regeneration the codebase already points to (the
slice headers literally say *"scaffold — regenerated at full breadth in
Task 11"* and the live gate test is `#[ignore]`d until it happens).

The corpus *definition*, *generator*, *gate*, and *verify-from-kernel* path
already exist and are merged (see
[2026-06-15-production-reference-corpus-design.md](2026-06-15-production-reference-corpus-design.md)).
This design executes the generation: pin the kernel identity, fix one
generator defect, produce and commit the real slices, make the fixture-golden
cross-check meaningful, extend reproduction to all slices, and flip the gate
live.

## Current state (verified)

- `corpus_spec.rs` is the single source of truth: slice roles
  (`boundary` / `interior` / `fast_cluster` / `holdout` / `fixture_golden`),
  per-body `max_gap_days` cadence, deterministic epoch grids, release vs.
  constrained body sets, and cross-check tolerances. `KERNEL_SHA256` is the
  placeholder `"<pinned-after-download>"`.
- `generate-spk-corpus <kernel> --emit-slices <dir>` (CLI →
  `pleiades-jpl::generate_slice` / `build_manifest`) reads a real `de440.bsp`
  through the pure-Rust SPK reader and emits the four backend slices + manifest;
  it is covered against synthetic kernels. `fixture_golden` is hand-populated
  and the generator refuses to produce it.
- `validate-corpus` (`pleiades-validate::corpus::production::run_corpus_gate`)
  enforces completeness, schema + provenance (rejects placeholder SHA),
  checksum drift, and the fixture-golden cross-check, all kernel-free.
- `corpus_regen.rs` regenerates **only the boundary slice** from a real kernel
  and diffs within 1 km.
- The checked-in slices are 3-row scaffolds with `checksum=0` and the
  placeholder SHA; `embedded_corpus_gate_passes` is `#[ignore]`d.
- The NAIF `de440.bsp` (119,799,808 bytes, public domain, supports range
  requests) is reachable; `sha256sum` is available.

## Defects this design also fixes

1. **Interior over-sampling.** `generate_slice(InteriorBackbone)` samples *every*
   body at the Moon's 30-day grid (cartesian product, ~122k rows ≈ 7 MB),
   contradicting `corpus_spec`'s per-body `max_gap_days` and over-claiming
   density for slow bodies.
2. **Fixture-golden cross-check is theater.** `fixture_golden.csv`'s epochs
   (J2000, JD 2500000) appear in no backend slice, so the cross-check finds zero
   overlap and passes trivially.

## Goals

- Pin the real `de440.bsp` SHA-256 in one place and stamp it through every
  artifact.
- Sample the interior backbone per body at its own spec'd cadence, deterministic
  and reproducible.
- Make the fixture-golden cross-check a real independent check against trusted
  Horizons values.
- Generate and commit the real corpus (~1.5 MB) with real checksums.
- Let a clean checkout **verify** kernel-free (gate green) and **reproduce**
  with the kernel (all slices, not just boundary).
- Sync docs/spec/plan and re-check the Phase 1 generation/verify/reproduce exit
  criteria.

## Non-goals (held to existing deferrals)

- No broad **public-data reader** for arbitrary external products — stays in the
  Phase 1 backlog.
- No asteroid-kernel adoption; no widening of body/channel/frame claims beyond
  what de440 geometry already supports.
- Output stays **mean geometric, geocentric, tropical-at-backend** (no apparent,
  light-time, topocentric, native sidereal — Phase 4).
- No change to the CSV schema (`epoch_jd,body,x_km,y_km,z_km`) or downstream
  report/backend-matrix/bundle machinery.

## Design

### 1. Kernel acquisition & SHA pinning (provenance)

- Download `de440.bsp` from
  `https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440.bsp`
  to a **gitignored** local path. The kernel is **not** committed (114 MB).
- Compute SHA-256 with `sha256sum`.
- Pin the hash in the single source of truth, `corpus_spec::KERNEL_SHA256`; the
  generator already stamps it into every slice header and the manifest.
- Record the same hash in `docs/spk-kernel-sourcing.md`.
- Confirm the backend's advertised coverage brackets 1600–2600 CE.

### 2. Interior generator refactor (per-body cadence)

- Generate `interior.csv` as each body sampled at its own
  `interior_backbone_epochs(body)`, not every body at the Moon grid.
- Emit in a deterministic, stable order: bodies in release-then-constrained
  order, epochs ascending within each body. Checksums and verify-from-kernel
  depend on this ordering being stable.
- Boundary, fast_cluster, and holdout keep their intentional shared-grid
  (all-bodies-at-each-epoch) sampling.
- Tests: assert slow bodies (Neptune) have far fewer interior rows than the
  Moon, and per-body gaps stay within `max_gap_days`.

### 3. Fixture-golden = real trusted anchors (Approach A)

- Add a small fixed set of **anchor epochs** (J2000 = 2451545.0 plus 1–2 trusted
  comparison epochs) to interior emission for the release bodies, so backend
  slices and `fixture_golden` overlap at known (body, epoch) pairs.
- Replace the scaffold `fixture_golden.csv` with **real trusted Horizons values**
  at those anchors, sourced from the existing checked-in reference snapshots
  (the repo already has exact-J2000 + early-2001 fixtures). Keep it
  hand-populated; the generator still refuses to produce it.
- Result: the cross-check compares de440-derived geometry against independent
  Horizons values within spec tolerances (Sun/planets < 50 km, Moon < 5 km;
  Pluto constrained, non-fatal).

### 4. Regenerate + commit

- Run `cargo run -p pleiades-cli -- generate-spk-corpus <kernel> --emit-slices
  crates/pleiades-jpl/data/corpus`.
- Commit the real `boundary.csv`, `interior.csv`, `fast_clusters.csv`,
  `holdout.csv`, `manifest.txt` (real checksums + pinned SHA), and the
  hand-populated real `fixture_golden.csv`. Expected total ~1.5 MB.

### 5. Verify-from-kernel extended to all slices

- Extend `corpus_regen.rs` from boundary-only to regenerate **every**
  backend-generated slice and diff against the checked-in CSV within tolerance,
  so the whole corpus is reproducible from the kernel, not just the boundary.

### 6. Flip the gate live

- Remove `#[ignore]` from `embedded_corpus_gate_passes` so `validate-corpus`
  runs against the real embedded corpus in normal CI. It now passes for a real
  reason and fails closed on body/role/schema/checksum/cross-check drift.

### 7. Docs / spec / plan sync

- Pinned SHA in `docs/spk-kernel-sourcing.md`.
- Update `PLAN.md`, `plan/status/01-*.md`, `plan/status/02-*.md`, and
  `plan/stages/01-production-reference-corpus.md` to reflect that the
  generation-pipeline + verify/reproduce exit criteria are met; the broad
  public-data-reader item remains open if not closed elsewhere.
- Remove "scaffold / Task 11" notes from slice headers (regeneration overwrites
  them) and any prose that calls the corpus a scaffold.
- Re-check Phase 1 exit criteria against the result.

## Testing & error handling

- Generator stays fail-closed: placeholder SHA rejected; `fixture_golden` must
  exist before `--emit-slices`.
- Gate fails closed on missing body/role, schema drift, checksum drift,
  non-finite / malformed rows, and cross-check tolerance breach.
- Synthetic-kernel unit tests continue to cover generation logic without the
  real kernel.
- The real kernel only gates the opt-in `corpus_regen` and `spk_full_kernel`
  tests (skipped via early return when `PLEIADES_DE_KERNEL` is unset).

## Exit criteria (this task)

- `cargo run -p pleiades-cli -- validate-corpus` passes against the committed
  real corpus, and `embedded_corpus_gate_passes` is no longer `#[ignore]`d.
- With `PLEIADES_DE_KERNEL` set, `corpus_regen` reproduces every backend slice
  within tolerance.
- `corpus_spec::KERNEL_SHA256` and `docs/spk-kernel-sourcing.md` carry the real
  de440 hash; no placeholder remains.
- Interior slice respects per-body cadence; fixture-golden cross-check exercises
  real overlap.
- `PLAN.md` and status/stage docs reflect the new state with no scaffold
  language remaining.
