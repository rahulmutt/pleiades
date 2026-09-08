# FU-9 slice — `pleiades-apsides` mutant triage (design)

**Status:** design approved · **Opened:** 2026-09-08 · **Follow-up:** FU-9 (new
post-baseline expansion slice) · **Crate:** `pleiades-apsides`

## Context

FU-9's original three-crate measured baseline (`pleiades-types`,
`pleiades-time`, `pleiades-apparent`) is **CLOSED**, and the first expansion
campaign — `pleiades-houses`, six PRs — is **COMPLETE** (see `docs/follow-ups.md`,
FU-9). FU-9 remains open as a standing posture entry: any expansion of the
report-only mutants tier to further `pleiades-*` crates opens a **new slice**,
not a continuation of either closed body. This is the second such expansion
slice and the first drawn from the next-campaign roadmap
(`docs/superpowers/specs/notes/2026-07-25-mutants-roadmap-baseline.md`).

`pleiades-apsides` is the roadmap's cleanest measured row — `223 tested /
33 missed`, 84.9%, the highest score of the four measured candidates and the
smallest surface. It is release-grade numeric code: the osculating Kepler apse
that serves `TrueApogee`/`TruePerigee` (Swiss Ephemeris `SE_OSCU_APOG`, "True
Black Moon Lilith") through `PackagedDataBackend`, guarded by the
`validate-lilith` parity gate. Choosing it continues FU-9's stated
numeric-survivors-first priority while closing a whole crate in one PR.

## Goal & scope

Drive surviving mutants in `pleiades-apsides` to
**0-or-documented-equivalent**, measured by the authoritative cargo-mutants
command, using intent-expressing white-box tests referenced to independent
authorities.

### Measured baseline (2026-09-08)

Whole-crate run at `7aefa946c`, cargo-mutants 27.1.0:

```bash
cargo mutants -p pleiades-apsides --test-tool nextest \
  --test-workspace=false --baseline run
```

```
223 mutants tested in 2m: 33 missed, 186 caught, 4 unviable
```

Exit code 2 (survivors found — the report-only tier's expected outcome). This
reproduces the roadmap row exactly, so no re-measurement discrepancy exists to
reconcile. All 33 survivors are in the crate's single `src/lib.rs`.

### Survivor classification

| Class | Line(s) | Count |
|-------|---------|-------|
| Non-finite / degeneracy guards (`\|\|`↔`&&`) | 76, 81, 104×3, 204, 248–251, 258 | 11 |
| Eccentricity-vector arithmetic (`c2` term + `e_hat` scaling) | 112×2, 114, 115×2, 116, 139×4 | 10 |
| `omega = 2π − omega` southern branch | 227×4 | 4 |
| Threshold comparisons | 122, 210×2, 226×2, 255×2 | 7 |
| Aphelion argument-of-latitude `ω + π` | 287 | 1 |
| **Total** | | **33** |

The four `unviable` mutants are all
`replace <fn> -> Result<…> with Ok(Default::default())` on the four functions;
none of the returned types implements `Default`, so they do not compile. They
are neither survivors nor kills and require no work.

### The dominant root cause is one degenerate geometry

Every existing test builds its state with `perigee_on_x_state`, which is
simultaneously:

- **apsidal** — `r·v = 0`, so `c2 = rv/mu` is exactly `0`, and
- **axis-aligned** — `e_hat = [1, 0, 0]`, so `e_hat[1]` and `e_hat[2]` are
  exactly `0`.

The first blinds the entire `c2` term: with `c2 == 0`, `c1*r[i] - c2*v[i]` is
bit-identical to `c1*r[i] + c2*v[i]`, and `rv/mu` is bit-identical to `rv*mu`
and `rv%mu`. That is six mutants, at lines 112 (×2), 114, 115 (×2) and 116.
The second blinds both
cross-component scalings at line 139: `-e_hat[1]*r_apo`, `e_hat[1]/r_apo`, and
`-e_hat[1]/r_apo` are all `0` (or `-0.0`, which compares equal), and likewise
for index 2. That is 4 more.

**Ten of the 33 survivors are one blind spot.** A single non-apsidal
(`ν ≈ 50°`), inclined, non-axis-aligned state kills all nine at once. This is
the same lesson the `topocentric.rs` slice recorded: rejected degenerate
geometries are written into the plan so they are not re-proposed.

### In scope

- `crates/pleiades-apsides/src/lib.rs` — the crate's only source file.

### Non-goals

- **No production behavior change**, except a behavior-preserving test
  relocation (its own commit, no runtime-result change). None of the four
  public functions needs a testability seam: all are already public and
  directly callable, and the one private helper (`to_ecliptic`) is reachable
  from an in-crate `#[cfg(test)]` module.
- **No parity-gate change.** The `validate-lilith` corpus, tolerances, and gate
  code are untouched. This slice adds *unit* coverage, not gate coverage.
- The mutants tier **stays report-only**. No mutation-score gate is introduced.

## Method

Unchanged from every prior FU-9 slice:

1. **Authoritative baseline:**
   `cargo mutants -p pleiades-apsides --test-tool nextest --test-workspace=false --baseline run`,
   output written outside the repo. Record `N tested, M missed, K caught,
   U unviable`.
2. **Classify** each survivor: numeric-formula, branch-unreached,
   validation guard, boundary comparison, or documented-equivalent.
3. **Add white-box tests asserting against an *independent* reference** —
   never the code's own output.
4. **Re-run** to confirm the residual is `0` or a documented equivalent.

## Reference strategy — analytic inverse + conic invariants

`pleiades-houses` used a Python reimplementation of the published `swehouse.c`
pipeline. That pattern is deliberately **not** reused here. The recurring
reviewer objection to it — a port of the same formulas the crate implements
satisfies anti-circularity but not conceptual independence
(`[[fu9-houses-reference-independence]]`) — would be at its sharpest in this
crate, whose ~80 lines of numeric logic *are* the textbook two-body formulas a
port would restate. This slice uses three levers that are independent by
construction:

### 1. Forward construction ≠ inverse extraction

Build state vectors **from** elements `(a, e, i, Ω, ω, ν)` via the published
Gaussian `P̂`/`Q̂` rotation basis, then assert that `apsides` and
`elements_from_state` recover the inputs. The forward map (elements → state,
a rotation of an in-plane conic point) is a genuinely different computation
from the inverse under test (state → eccentricity vector and specific orbital
energy). A sign flip or operator swap in the inverse breaks the round-trip; no
shared expression can mask it. The crate already has one such test
(`elements_from_state_round_trips_through_points`) — this generalizes it to
non-degenerate geometries and to `apsides` itself.

### 2. Conic invariants the code never computes

Each returned point is additionally pinned by defining properties of the
ellipse, none of which appears anywhere in `lib.rs`:

- **Bifocal sum** — `r₁ + r₂ = 2a` for every point on the ellipse, measuring
  from the occupied focus and from the empty focus at `2ae`. This directly
  cross-checks `points_from_elements`' `second_focus` branch against the
  ordinary aphelion branch.
- **Vis-viva** — `v² = μ(2/r − 1/a)`, an independent route to the semi-major
  axis from the same state.
- **Conic radius law** — `r = p/(1 + e·cos ν)` with `p = a(1 − e²)`, checked at
  the node arguments where `ν = ∓ω`.
- **Orthogonality** — `h = r × v` is perpendicular to both `r` and `v`, and
  `cos i = h_z/|h|`.

These are satisfied only by a correct ellipse. The crate has no path to satisfy
them accidentally, because it never evaluates them.

### 3. Published-constant recomputation

`MU_EARTH_MOON_AU3_PER_DAY2` is pinned by recomputing
`(GM⊕ + GM☾) = 403503.2418 km³/s²` into AU³/day² from the documented
constants and unit conversions (`1 AU = 149597870.7 km`, `1 day = 86400 s`),
evaluated outside the code, to the tolerance the rustdoc's "tuned against the
`validate-lilith` gate" wording implies. The tolerance is stated and justified
in the test, not silently widened to whatever passes.

## Disposition of the 33 survivors

Hypotheses recorded so they are not re-litigated during triage. The **true
count is measured, not predicted** — the plan re-measures first and classifies
from the measurement.

| Survivors | Line(s) | Approach |
|-----------|---------|----------|
| 10 | 112×2, 114, 115×2, 116, 139×4 | One non-degenerate inclined geometry (above) |
| 4 | 227×4 | A `ω = 210°` case — no current test reaches the `peri_vec[2] < 0` branch at all, so all four operator swaps in `2.0 * π − omega` are unexercised rather than indistinguishable |
| 7 | 248–251, 255×2, 258 | Free-parameter guards on `points_from_elements`, which takes an explicit `KeplerianElements` struct. A single input — `a = -inf` with every other field finite — kills all four `&&` mutants at once (`Err(NonFinite)` → `Err(UnboundOrbit)`). `e = MIN_ECCENTRICITY` and `e = 1e-7` kill 255; `(e=1.5, a=2.0)` and `(e=0.5, a=-2.0)` kill 258. |
| 2 | 76, 104:27 | **Overflow lens** — components finite but the squared norm overflows to `+inf`. `to_ecliptic([1e200; 3])` then yields a *finite* lon/lat (`p[i]/inf = 0`), so the mutant returns `Ok` where the original errors. Per `[[fu9-guard-equivalence-overflow-lens]]`. |
| 1 | 104:62 | The `mu <= 0.0` arm — a **negative μ** with finite non-zero position: `Err(NonFinite)` → `Ok`. |
| 1 | 204 | Radial motion (`h = 0` exactly). Requires care: a plain radial state has `e == 1.0` exactly, so `r_peri = a(1−e) = 0` and `to_ecliptic` fails *inside* `apsides()` before line 204 is reached. A crafted `vx` leaves `e` one ulp below 1, keeping `r_peri ≠ 0`. |
| 2 | 210×2 | `<`→`<=` by a state sitting **exactly** on `n_mag == 1e-12·h_mag`; `*`→`/` by a tiny-but-supra-threshold inclination (`i = 1e-10°`), which separates `1e-12·h_mag` from `1e-12/h_mag`. |
| 2 | 122, 226 (`<`→`==`) | Reachable boundaries — see the crafted eccentricity state below and the `ω = 210°` southern branch above. |
| **4** | 81, 104:43, 226 (`<`→`<=`), 287 | **Documented equivalents** — arguments below |

### Documented equivalents (measured: 4)

Each is left **visible with a written reachability argument**, never
`#[mutants::skip]`-suppressed — the established posture, since a function-level
skip would blanket-suppress that function's numeric mutants.

- **104:43** — the second `||` in
  `!r_mag.is_finite() || r_mag == 0.0 || !mu.is_finite() || mu <= 0.0`. Rust
  binds `&&` tighter than `||`, so the mutant is `a || (b && c) || d`, not
  `((a || b) && c) || d`; its only distinguishing region is
  `{r_mag == 0.0 exactly, μ finite, μ > 0}`. There `μ/r_mag = +inf` forces
  `c1 = -inf`, and every sub-case poisons `e`: all-zero `pos` gives
  `-inf · 0 = NaN`; an all-subnormal `pos` whose squared norm underflows to
  `0.0` gives `∓inf` components and `e = inf`; mixed cases give `NaN`. All
  three fail `!e.is_finite()`, so **both** branches return `Err(NonFinite)`.
  The `r_mag == 0.0` arm is redundant with the downstream finiteness check.
- **81** — `!longitude_deg.is_finite() || !latitude_deg.is_finite()`. Line 76
  has already established that `r` is finite and non-zero, so `p` is
  componentwise finite; `atan2` is total on finite inputs and `p[2]/r` lies in
  `[-1, 1]`, making `asin` total. No input makes exactly one of the two
  non-finite, so `||` and `&&` cannot be distinguished. This is the
  `nutation`/`topocentric`/`precession` shared-poison shape.
- **226 `<` → `<=`** — the two differ only when `peri_vec[2] == 0.0` exactly,
  i.e. when the perihelion lies in the reference plane. With `i` non-degenerate
  (line 210 has already rejected `i ≈ 0`), that means `ω ∈ {0, π}`, where
  `cos_omega` is `±1` and `acos` returns exactly `0` or `π`. The mutated branch
  then yields `2π − 0 = 2π` and `2π − π = π`, and the subsequent
  `(node_deg + omega.to_degrees()).rem_euclid(360.0)` maps `2π` and `0` to the
  same longitude. Modular lens, per
  `[[fu9-equivalence-free-param-and-modular-lenses]]`.
- **287 `+` → `-`** — `in_plane(ω + π, …)` versus `in_plane(ω − π, …)`. The two
  arguments differ by exactly `2π`, so `cos`/`sin` agree to within rounding.
  **Measured** at `Ω = 40°, i = 10°, a = 2, e = 0.2`: the resulting longitudes
  differ by `0` (at `ω = 30°`), `1.42e-14°` (at `ω = 210°`) and `2.84e-14°`
  (at `ω = 0°`), and the latitudes by at most `2.44e-15°` — i.e. below
  `1e-10` arcsec. No tolerance justifiable against an independent reference
  reaches that; an assertion tight enough to kill it would be pinning the
  code's own output, which is the failure mode this tier exists to avoid.

### The crafted eccentricity boundary (122) — resolved, killable

**122 — `e < MIN_ECCENTRICITY` → `<=`** was open at design time and is now
**settled: killable.** The concern was that `e` here is *derived*, not a free
parameter — `norm(c1·r − c2·v)` with `c1 = (v² − μ/r)/μ`. Near `e ≈ 1e-6` that
is a catastrophic cancellation, so the reachable `e` values sit on a grid of
roughly `2.2e-10` relative spacing, about `1e6` times coarser than the
`~2.1e-22` absolute precision needed to land on `MIN_ECCENTRICITY` exactly.
Two searches confirmed the difficulty is real (~160k and then ~11.1M
`(r, v, μ)` neighbour trials, no hit) — but both swept `r_mag` over **powers of
two**, which makes the final `fl(c1 · rx)` product exact and therefore coarse.

Sweeping `r_mag` over **consecutive doubles** gives that multiply an
independent rounding, and the step in the exact product per `r_mag` ulp
(`≈ 2.2e-22`) is comparable to `ulp(1e-6)` — so hits exist at roughly `1e-6`
density. One was found and **verified against the crate**:

```
pos = [f64::from_bits(0x400000000005a740), 0.0, 0.0]   // 2.0000000001645333
vel = [0.0, f64::from_bits(0x3fe6a09f244b3b60), 0.0]   // 0.70710713471076400
mu  = 1.0
=> apsides(...) = Ok, eccentricity bit-identical to MIN_ECCENTRICITY
```

The original returns `Ok` (since `1e-6 < 1e-6` is false); the `<=` mutant
returns `Err(DegenerateOrbit)`. The test carries an in-test precondition
`assert_eq!(aps.eccentricity, MIN_ECCENTRICITY)` proving the crafted input
lands on the boundary exactly, per `[[fu9-jd-grid-representability]]`.

**Recorded so it is not re-derived:** the power-of-two `r_mag` family is a
dead end. Do not re-run it.

## Structure change

Relocate the ~180-line inline `#[cfg(test)] mod tests { … }` from `lib.rs` to
`crates/pleiades-apsides/src/tests.rs`, leaving `#[cfg(test)] mod tests;` in
`lib.rs`. This is AGENTS.md's "keep large inline test suites out of the file
under test" rule and follows the `thresholds.rs` precedent from the houses
campaign's PR 6. White-box unit tests stay unit tests — they are **not**
converted to black-box integration tests, since `to_ecliptic` is private.
Its own commit, no behavior change.

## Weekly-tier expansion (`[tasks.mutants]`)

Add `-p pleiades-apsides` to `mise.toml`'s `[tasks.mutants]` so the weekly
report-only tier regression-checks the crate going forward — the "make it
stick" step, mirroring how the baseline three and `pleiades-houses` are
enumerated.

Projected weekly total `2,620 + 223 = 2,843` mutants (~1.09× the current
projection). At the campaign's measured throughput this is roughly **+2 minutes**
of testing on the measured ~24-25m job wall-clock; `timeout-minutes: 90` is
retained unchanged. This is a **projection, not a measurement** — the next
scheduled run's actual wall-clock supersedes it, and the calibration comment in
`.github/workflows/mutants.yml` is updated to cite this slice.

## Acceptance criteria

- `crates/pleiades-apsides/src/lib.rs` reaches **0 surviving mutants or
  documented equivalents**, confirmed by the authoritative command; the
  whole-crate re-run reports `0 missed` or only the documented equivalents.
- Every expected value derives from an **independent reference** (forward
  construction, conic invariant, or published constant), not the code's own
  output.
- A **per-mutant margin table** — one row per mutant × geometry, stating the
  true minimum displacement, never aggregated
  (`[[fu9-margin-table-per-mutant-rows]]`). Where a kill is an exact
  equality or error-variant pin rather than a scalar displacement, that is
  disclosed as such and no margin is fabricated.
- **No parity gate touched** — `validate-lilith` corpus, tolerances, and gate
  code unchanged.
- Mutants tier **stays report-only**; no score gate introduced.
- `mise run ci` green (fmt + clippy `-D warnings` + workspace test), with
  `cargo fmt` run before **every** commit
  (`[[fu9-houses-plan-literals-not-rustfmt-clean]]`).
- `[tasks.mutants]` includes `-p pleiades-apsides`.
- An **FU-9 progress note** appended to `docs/follow-ups.md` in the established
  format, framed as a new post-baseline expansion slice, with any documented
  equivalents added to the running campaign tally (currently `45`).
- The roadmap note is updated to mark `pleiades-apsides` as triaged, so the
  three remaining measured rows stay an accurate queue.

## Risks & mitigations

- **The 122 boundary may be genuinely unreachable.** *Mitigation:* treated as
  measured-not-predicted above; the plan states its search bounds so an
  equivalence claim is falsifiable.
- **Reference circularity.** *Mitigation:* the forward/inverse split plus conic
  invariants the code never evaluates — structurally more independent than the
  Python-port pattern, chosen specifically to avoid the objection recorded in
  `[[fu9-houses-reference-independence]]`.
- **Crafted-input representability.** Boundary pins must hit their thresholds
  exactly. *Mitigation:* per `[[fu9-jd-grid-representability]]`, expected values
  are computed through the same full-magnitude arithmetic the code uses, with an
  in-test precondition assert that the crafted input lands on the boundary.
- **Cross-crate blast radius.** `pleiades-core`, `pleiades-events`, and
  `pleiades-data` consume this crate, so any testability seam would ripple.
  *Mitigation:* none is anticipated — all four functions are public and the one
  private helper is in-crate testable. If one proves necessary it lands as a
  separate no-op commit, per the `apparent.rs`/`aberration.rs` precedent.
- **Predicting equivalents instead of measuring.** *Mitigation:* the
  dispositions above are hypotheses; the plan re-measures the survivor list
  first and classifies from that measurement.
