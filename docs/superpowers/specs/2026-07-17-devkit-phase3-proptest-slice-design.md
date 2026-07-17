# Devkit Phase 3 Slice — proptest for `pleiades-types` + `pleiades-compression`

**Date:** 2026-07-17
**Status:** Approved design, pending implementation plan
**Parent:** [`2026-07-17-devkit-adoption-design.md`](./2026-07-17-devkit-adoption-design.md) — Phase 3, first slice

## Goal

Land the first slice of Phase 3 (Testing upgrades) from the devkit-adoption
design: property-based tests, via `proptest`, on the two crates where an
**invariant oracle** most cleanly beats the repo's existing **derived**
(Swiss-Ephemeris / JPL-parity) oracles — `pleiades-types` and
`pleiades-compression`. This establishes the reusable proptest harness and
conventions (workspace dependency, case budget, regression-file policy, CI
wiring) on the cleanest targets, so later slices inherit them.

Aligns with the `devkit:testing-practices` skill: property-based testing with an
invariant oracle is the right form when a rule holds for all inputs
(round-trips, idempotence, range invariants), and AGENTS.md already sanctions
"property tests for invariants and conversions where appropriate."

## Non-goals

- No `pleiades-time` or `pleiades-houses` property tests — deferred to a
  follow-up slice, because their properties carry tolerance (time round-trips)
  and geometry / high-latitude (house cusps) subtleties that risk turning a
  bounded slice into an open-ended investigation.
- No `cargo-fuzz` or `cargo-mutants` — later Phase 3 slices.
- No new CI task or workflow — property tests are ordinary `#[test]` functions
  under the existing blocking-tier `mise run test` (nextest).
- No change to `release-gate`, numeric gates, or the two-tier CI split.

## Scope decision (why this slice, why these two crates)

Phase 3 bundles three independent tools. Per AGENTS.md's small-reviewable-unit
rule, "next item" is one slice, not the whole phase. proptest is chosen first:
blocking-tier, no new toolchain (a pure-Rust dev-dependency), pure additive
value. Among proptest's four candidate crates, `pleiades-types` and
`pleiades-compression` expose **exact** invariant / round-trip oracles — a
property either holds or exposes a genuine bug, with no tolerance tuning —
making them the smallest, safest way to prove the harness.

## Harness & infrastructure (established once, reused by later slices)

- Add `proptest` to `[workspace.dependencies]` in the root `Cargo.toml`,
  version-constrained; the exact resolution is pinned by `Cargo.lock` per
  AGENTS.md's pinning rule. proptest is pure Rust — no C toolchain, honoring the
  no-mandatory-native-deps stance.
- Each crate opts in under `[dev-dependencies]` with
  `proptest = { workspace = true }`. proptest is a **library dev-dependency**
  (Cargo/`Cargo.lock`-pinned), not a tool — `developer-environment`'s mise-first
  rule governs tools, not crate dependencies.
- **Test layout:** co-located per AGENTS.md. `proptest! { … }` blocks live in
  each module's co-located test file (extend `angles`' tests; extend
  `frame_recombine`'s `#[cfg(test)] mod tests`). Kept white-box as unit tests —
  not converted to black-box integration tests to move them.
- **Case budget:** proptest's default of 256 cases/property matches the parent
  design's ~256 target; no per-test config required. Total added runtime is a
  handful of properties × 256 × two crates — trivially inside the blocking-tier
  ≤ 10-minute budget.
- **CI wiring: none.** `proptest!` expands to `#[test]` functions, so
  `mise run test` (nextest, blocking) already collects and runs them.
- **Determinism / regression policy (fail-closed culture):** ensure
  `proptest-regressions/` is **not** gitignored, so any counterexample proptest
  discovers is committed and replayed deterministically on every later run — a
  found counterexample must stay found. Verify `.gitignore` and add a keep-rule
  if the directory would otherwise be ignored.

## Properties — `pleiades-types` (`crates/pleiades-types/src/angles.rs`)

Invariant oracle. All strategies generate finite inputs (see edge conditions).

1. `normalized_0_360` result ∈ `[0, 360)` for all finite inputs.
2. `normalized_0_360` is **idempotent**: `x.norm().norm() == x.norm()`.
3. `normalized_0_360` **congruence**: result ≡ input (mod 360).
4. `normalized_signed` result ∈ `[-180, 180)`, and is idempotent.
5. degree↔radian round-trip: `Angle::from_radians(a.radians()) ≈ a` within
   floating-point tolerance; `Angle::from_degrees(d).degrees() == d` exactly.
6. `Longitude::from_degrees` output is always ∈ `[0, 360)` (constructor
   normalizes on the way in).

## Properties — `pleiades-compression` (`crates/pleiades-compression/src/frame_recombine.rs`)

Invariant oracle. Generalizes the existing single-point
`velocity_round_trips_through_cartesian` unit test across the input space.

1. `cartesian_au_to_ecliptic ∘ ecliptic_to_cartesian_au ≈ id` within tolerance
   (longitude compared mod 360).
2. `cartesian_state_to_spherical ∘ spherical_state_to_cartesian ≈ id` for
   **both position and velocity** components.
3. Geo/helio inverse:
   `heliocentric_from_geocentric(geocentric_from_heliocentric(p, sun), sun) ≈ p`.
4. Artifact `decode(encode(a)) == a` — included **with a fallback**: if a bounded
   `Artifact` generation strategy proves heavy to build, this property drops to
   the follow-up slice; the three round-trips above still ship the harness.

## Honest edge-condition handling (bounded in strategies, not chased as bugs)

Each is boundable, which is precisely why these two crates were chosen over
time/houses:

- **Non-finite inputs:** `f64::rem_euclid` on ±∞ yields NaN. Strategies
  generate finite values in reasonable ranges; the properties assume finite
  inputs.
- **Poles:** latitude ≈ ±90° makes longitude ill-conditioned. Bound the
  latitude strategy away from the exact poles, or compare in Cartesian space.
- **Zero radius/distance:** `cartesian_au_to_ecliptic` special-cases `r == 0`
  (longitude is undefined and returned as 0). Bound the distance strategy to
  `> ε`.

## Acceptance criteria

1. `mise run test` passes with the new property tests included.
2. `mise run fmt` and `mise run lint` are clean.
3. The measured blocking-tier runtime delta is recorded and comfortably within
   the ≤ 10-minute budget.
4. `proptest-regressions/` is committable (not gitignored), verified.
5. A one-line pointer is added (AGENTS.md or `spec/validation-and-testing.md`)
   naming property tests as the invariant-oracle layer — single-sourced, no
   duplication of the properties themselves.
6. `pleiades-time` and `pleiades-houses` remain explicitly out of scope and
   noted as the next slice.
