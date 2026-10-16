# FU-9 slice: `pleiades-fict` mutant triage (design)

**Status:** done · **Opened:** 2026-10-16 · **Issue:** #162 (slice 4, the last of
the queue) · **Crate:** `pleiades-fict`

## Goal and scope

Drive the surviving mutants in `pleiades-fict` to zero or a documented
equivalent, and make the crate safe to join the weekly mutants set. Two of its
mutants used to hang, which made cargo-mutants exit 3 (timeout) and fail the
report-only tier.

Tests assert against an independent reference: a value worked out by hand from
the cited formula, a textbook Kepler solution, a Meeus precession, or a row of
the committed Swiss Ephemeris corpus. No expected value was captured from this
crate's output.

Non-goals: no change to any parity gate, corpus or tolerance in
`pleiades-validate`, no `#[mutants::skip]`. The one production change is §1
below. The non-test, non-production edits are the CI wiring (`mise.toml`,
`mutants.yml`) and the roadmap row.

## Measured baseline and result

```bash
cargo mutants -p pleiades-fict --test-tool nextest --test-workspace=false \
  --baseline run -j 4 --timeout 120
```

| When | Mutants | Missed | Caught | Unviable | Timeouts |
|------|---------|--------|--------|----------|----------|
| 2026-07-25 roadmap | 308 | 148 | 147 | 11 | 2 |
| 2026-10-16 before (0f7453fa) | 310 | 137 | 158 | 13 | 2 |
| 2026-10-16 after | 312 | 5 | 294 | 13 | 0 |

The five survivors are the documented equivalents below. The count grows by two
because the rewritten `signed_longitude_delta` has a different set of mutation
sites from the two loops it replaced. All of its mutants are caught.

## §1 The one production change: loop-free `signed_longitude_delta`

The function used `while d > 180.0 { d -= 360.0 }` and
`while d < -180.0 { d += 360.0 }`. A mutant that flips either comparison (`>`
to `<`, `<` to `>`) never terminates, and so did any input of infinity. It is
now `d = (after - before) % 360.0` followed by one `if/else if` that adds or
subtracts 360.

**The results are the same as the loops on every finite input** (the equal
values, and the same bits except for one signed-zero case below).

- *Each step is exact.* `x - 360` for `x > 180` has magnitude below `|x|` and
  `360` is a multiple of `ulp(x)` for `|x| < 2^56`, so it is representable
  and the subtraction does not round. The loop therefore ends at exactly
  `d - 360 k` for the `k` that lands in `[-180, 180]`. The same holds for `x + 360`
  with `x < -180`.
- *`%` is exact too.* `fmod` returns the exact remainder with the dividend's
  sign, so `r = d - 360 k'` exactly, with `r` in `(-360, 360)`. One more `r - 360`
  (for `r > 180`) or `r + 360` (for `r < -180`) is exact by the argument above
  and lands in `[-180, 180]`.
- Both land on the unique representative of `d` modulo 360 in `[-180, 180]`, so
  the values are equal. The ties are the same: `+180` and `-180` are kept as
  they are by both (the guard is a strict comparison).
- *Boundary checks.* `180 -> 180`, `-180 -> -180`, `540 -> 180`
  (loop: `540 - 360`; new: `540 % 360 = 180`), `-540 -> -180`, `360 -> 0` and
  `-360 -> 0` in value, a tiny `d` (such as `1e-300`) is returned unchanged by
  both, and `d = 1e6` goes through about 2,778 exact subtractions in the loop and
  one exact `fmod` in the new code, both giving `1e6 - 360 * 2778`.
- *The one difference: the sign of a zero.* For `d = -360 k` (`k >= 1`) the loop
  ends at `+0.0` (`x + (-x)` rounds to `+0`), while `fmod(-360 k, 360)` is
  `-0.0`. They compare equal, and the sign cannot reach the output in the
  one caller: `motion` divides by a positive `full`, so `-0.0` only appears as
  a `-0.0` speed, and `after - before` of longitudes normalised to `[0, 360)`
  has magnitude below 360, so it never equals a multiple of 360 other than 0.
  (`0 - 0` is `+0` under round to nearest.)
- *Non-finite input.* Infinity looped forever; `inf % 360` is NaN and a NaN
  delta is NaN under both. The new code returns NaN rather than hanging.

`validate-fictitious` is unchanged by the edit:

```
before: validate-fictitious: 570 rows — max residuals: lon 1.262", lat 0.710", dist 6.14e-6 AU
after:  validate-fictitious: 570 rows — max residuals: lon 1.262", lat 0.710", dist 6.14e-6 AU
```

Tests: `signed_longitude_delta_wraps_across_the_seam` (179 -> 179, 181 -> -179,
-181 -> 179, 180 -> 180, -180 -> -180, 540 -> 180, -540 -> -180, 719 -> -1, and
two real pairs across 0/360) and
`signed_longitude_delta_of_a_non_finite_difference_is_nan_not_a_hang` (NaN,
`+inf`, `-inf`).

## Why the mutants survived

- **The geometry was tested only for finiteness.** The inline tests asserted
  that `position()` returned `Some`, that a circular orbit has radius `a`, and
  that precession preserves distance. None pinned a coordinate, so every
  arithmetic operator in `state_at`, `spherical_to_cartesian`,
  `cartesian_to_ecliptic`, `orbital_plane_position` and `motion` could change
  without notice.
- **The test inputs hid whole terms.** Every shipped fictitious body has
  `c2 = 0` for every element, and most have zero inclination, node or
  eccentricity, so the 3-1-3 rotation and the `t * t` term are mathematically
  invisible on the table. The distance test used `(3, 0, 0)`, in which `y` and
  `z` vanish.
- **No test crossed a one-sided or fast case.** `motion`'s one-sided fallback
  divides by `0.5`, but the only test checked `is_some()`. With a symmetric
  window `full = 1.0`, so `x / full`, `x * full` and `x % 1.0` agree for any
  speed below one.
- **Real tolerances were too loose.** The identity test for the J2000 equinox
  used `1e-12`; the precession path moves a vector within a microday of J2000 by
  only 4e-13, so the identity-branch mutants passed.
- **Untested table plumbing.** `jd_from_token`'s three constants and the CSV
  rows were never compared with `seorbel.txt`; `fictitious_bodies()` and
  `fictitious_body_claims()` could return `vec![]`.
- **A mutant that hangs.** The loops above.

## Kills

| Survivors | Site | Test (file) | Reference |
|-----------|------|-------------|-----------|
| 14 + 2 timeouts | `signed_longitude_delta` (the loops; rewritten) | `signed_longitude_delta_wraps_across_the_seam`, `..._non_finite_difference_is_nan_not_a_hang` (`backend/tests.rs`) | Hand cases across the seam; infinity gives NaN |
| 3 | `poly` (`+`→`-`, `+`→`*` twice) | `poly_is_c0_plus_c1_t_plus_c2_t_squared` (`elements/tests/reference.rs`) | `1 + 2*2 + 3*2*2 = 17` at t = 2, `1 - 2 + 3 = 2` at t = -1 |
| 28 | `state_at` lines 73-103 (time, `&&`, third-law motion, center, rotation) | `state_at_matches_the_textbook_kepler_solution` | 12 positions (4 element sets x 3 epochs) from a separate textbook solution: Kepler's equation by bisection, `x = a(cos E - e)`, `y = a sqrt(1-e^2) sin E`, 3-1-3 matrices |
| 3 | `jd_from_token` constants | `jd_tokens_resolve_to_the_seorbel_keywords_and_bare_julian_days` | `J1900` 2415020.0, `B1950` 2433282.42345905, `J2000` 2451545.0, bare JD |
| 2 | `fictitious_bodies`, `fictitious_body_claims` -> `vec![]` | `the_backend_lists_the_nineteen_fictitious_bodies_in_table_order` | 19 bodies, Cupido first, Waldemath last |
| 3 | `earth_heliocentric` `delete -` | `earth_is_the_negated_sun_vector` | Sun at (lon 120, lat 30, r 2) = (-sqrt3/2, 3/2, 1) by hand |
| 5 | `geocentric_ecliptic` line 80 (`-`→`+` x3, `-`→`/` x2) | `a_heliocentric_body_is_geocentricised_by_subtracting_earth` | Body (3,0,0), Earth (sqrt3/2, -3/2, -1): geocentric (3 - sqrt3/2, 3/2, 1), r = 2.7935, lon 35.1039, lat 20.9755 |
| 14 | `motion` (probe offset, `HALF * 2.0`, `/`, `-` on lon, lat, distance, delete arm) | `motion_is_the_central_difference_over_one_day`, `motion_across_the_longitude_seam_is_the_short_signed_arc`, `a_heliocentric_body_moves_against_a_fixed_earth`, `a_one_sided_probe_divides_by_half_a_day` | Central and one-sided differences of the textbook positions; one case straddles 360 |
| 36 | `spherical_to_cartesian` (27 constant tuples, 9 operators) | `spherical_to_cartesian_matches_the_hand_computed_components` | (30,0,2) = (sqrt3,1,0); (60,30,2); (359,-45,3); poles; unit sphere |
| 8 | `cartesian_to_ecliptic` | `cartesian_to_ecliptic_matches_the_hand_computed_spherical_triple`, `..._of_the_origin_is_latitude_zero_without_nan` | (1,2,2): r = 3, lon = atan2(2,1), lat = asin(2/3); mirrored and (3,-4,12) |
| 1 | `solve_kepler` `<`→`>` (line 8) | `solve_kepler_converges_for_the_high_eccentricity_orbits` | Substitute `E - e sin E = M`; Nibiru's e = 0.981092 |
| 4 | `orbital_plane_position` | `orbital_plane_position_is_a_cos_e_minus_e_and_a_sqrt_1_minus_e2_sin_e` | a = 2, e = 0.6, E = 1: x = 2 (cos 1 - 0.6), y = 1.6 sin 1 |
| 11 | `rotate_ecliptic_to_j2000` | `precession_to_j2000_matches_the_meeus_ecliptic_reference`, `an_equinox_within_a_microday_of_j2000_returns_the_vector_untouched`, `a_distant_equinox_is_precessed` | Meeus ch. 21 ecliptic precession (eqs. 21.5, 21.7), independent of `pleiades_apparent`; bit-exact identity near J2000 |
| (guard) | Table rows and real orbits | `the_table_carries_the_transcribed_seorbel_rows`, `geocentric_orbit_bodies_reproduce_corpus_rows` | The `seorbel.txt` rows the CSV quotes; four rows of `fictitious.csv` (WhiteMoon, Waldemath at 2415020.5 and 2450285.534482758) within the gate ceilings 0.65", 0.09", 3.5e-6 AU |

**What is not a corpus test.** The corpus
(`crates/pleiades-validate/data/fictitious-corpus/fictitious.csv`) holds
geocentric positions only, so a heliocentric element set cannot be checked
against it without an Earth position, and `pleiades-fict` has no Sun backend
(and must not depend on one). The corpus rows therefore pin the two
geocentric-orbit bodies (which need no Sun), and the heliocentric arithmetic is
pinned by hand-built element sets against the textbook reference. The elements
use only J2000 equinoxes, so the equinox rotation is the identity and the
precession is covered separately.

## Reference values

The Kepler/orientation references (`state_at`, `motion`, the assembled places)
were produced by an independent script that shares no code with the crate:
Kepler's equation solved by bisection (200 halvings), the textbook perifocal
position, and explicit `Rz(node) Rx(i) Rz(argp)` matrices. The script is not
committed; it reproduces from the formulas in the test comments. Spot values
that can be checked by hand are in the tests (`2 (cos 1 - 0.6)`, `sqrt3`,
`(1,2,2) -> r = 3`).

Tolerances. State, place and speed assertions use `1e-9` (degrees or AU per
day) and `1e-10` for distance speeds; the reference solves Kepler's equation to
machine precision and the crate to `1e-14` rad, so the residual is about 1e-13.
The precession reference (Meeus, ecliptic form) and the crate (IAU-1976
equatorial `zeta/z/theta`) agree to about 7e-11 AU on the test vectors
(measured with a 1e-12 tolerance), so the Meeus test uses `1e-8`. The corpus
rows use the gate's own ceilings.

## Per-mutant displacement rows

Displacement is `|mutant - reference|` of the quantity the test asserts, worked
out for the test inputs by the same independent script with the operator at that
site swapped. A `NaN` row means the mutant produces a non-finite value, which
fails every `assert_close` (a comparison with NaN is false).

### `state_at` (AU, largest over the four element sets; tolerance 1e-9)

Epochs: `t = +0.5` is J2000 + 18262.5 d, `t = -0.3` is J2000 - 10957.5 d,
`dt = 1000` is J2000 + 1000 d. Each cell is the best of the four sets (one
failing assertion kills). Sites are `line:col` in `elements.rs`.

| Mutant | t = +0.5 | t = -0.3 | dt = 1000 |
|--------|----------|----------|-----------|
| 73:44 `/`→`%` | NaN | NaN | NaN |
| 73:44 `/`→`*` | NaN | NaN | NaN |
| 73:24 `-`→`+` | NaN | NaN | NaN |
| 73:24 `-`→`/` | 1.733 | 1.923 | 2.535 |
| 81:41 `&&`→`\|\|` | 0.4507 | 0.3289 | 0.02922 |
| 83:52 `/`→`*` | 7.545 | 8.847 | 7.022 |
| 83:57 `*`→`+` | 7.341 | 9.362 | 2.841 |
| 83:57 `*`→`/` | 3.086 | 4.442 | 0.5294 |
| 84:28 `==`→`!=` | 8.804 | 6.449 | 5.455 |
| 85:22 `/=`→`%=` | 8.804 | 6.449 | 5.455 |
| 85:22 `/=`→`*=` | 7.537 | 3.754 | 4.229 |
| 87:27 `+=`→`-=` | 4.443 | 8.284 | 9.356 |
| 87:35 `*`→`+` | 7.183 | 9.623 | 9.572 |
| 87:44 `-`→`+` | 1.63 | 3.415 | 8.038 |
| 99:26 `-`→`+` | 4.944 | 4.092 | 5.019 |
| 99:31 `*`→`+` | 1.603 | 1.586 | 1.73 |
| 100:26 `+`→`-` | 4.445 | 4.183 | 4.922 |
| 100:26 `+`→`*` | 3.45 | 5.468 | 7.524 |
| 100:21 `*`→`/` | 4.582 | 3.629 | 3.147 |
| 101:25 `-`→`+` | 8.956 | 7.43 | 7.408 |
| 101:35 `*`→`+` | 1.109 | 1.074 | 1.086 |
| 101:30 `*`→`+` | 4.66 | 3.814 | 3.82 |
| 101:30 `*`→`/` | 0.1732 | 0.1236 | 0.1165 |
| 102:25 `+`→`-` | 1.863 | 2.348 | 1.505 |
| 102:25 `+`→`*` | 2.809 | 3.763 | 4.83 |
| 102:20 `*`→`/` | 4.405 | 10.02 | 5.672 |
| 102:30 `*`→`/` | 0.1083 | 0.1505 | 0.1 |
| 103:20 `*`→`+` | 3.569 | 3.353 | 3.328 |

The true minimum is **0.02922** (81:41 `&&`→`||` at dt = 1000), eight orders
above the 1e-9 tolerance. 81:41 is the binding mutant: it needs a set with a
mean-anomaly T-term, and the margin depends on the third-law motion that the
mutant would add on top (small when `dt` is small). Over the other mutants the
minimum is **0.1** (102:30 `*`→`/` at dt = 1000).

### `poly` (hand values, exact)

`poly([1,2,3], t)`: 17 at t = 2 and 2 at t = -1.

| Mutant | t = 2 | t = -1 |
|--------|-------|--------|
| 63:10 `+`→`-` (`c0 - c1 t + c2 t^2`) | 9 (disp 8) | 6 (disp 4) |
| 63:21 `+`→`-` | -7 (disp 24) | -4 (disp 6) |
| 63:21 `+`→`*` (`c0 + c1 t c2 t t`) | 49 (disp 32) | -5 (disp 7) |

The true minimum is **4** (63:10 at t = -1).

### `earth_heliocentric` and `geocentric_ecliptic`

Earth = (0.8660, -1.5, -1.0); the displacement of each component under
`delete -` is `2|c|`: **1.732**, **3.0**, **2.0** (tolerance 1e-12).

For `geocentric_ecliptic` (body (3,0,0), Earth as above) the displacement is
given as the largest of (lon deg, lat deg, r AU), tolerance 1e-9:

| Mutant (line 80) | disp (lon, lat, r) | largest |
|------------------|--------------------|---------|
| x `-`→`+` | 13.9, 7.418, 1.472 | 13.9 |
| x `-`→`/` | 11.69, 6.138, 1.112 | 11.69 |
| y `-`→`+` | 289.8, 0, 0 | 289.8 |
| y `-`→`/` | 35.1, 4.133, 0.4369 | 35.1 |
| z `-`→`+` | 0, 41.95, 0 | 41.95 |

The true minimum is **11.69** (x `-`→`/`).

### `motion` (deg/day, deg/day, AU/day; tolerance 1e-9 and 1e-10)

Five cases: `geo lin` (geocentric linear mean anomaly at J2000 + 1000 d, speeds
1.825, -0.5623, 0.01209), `seam` (JD 2452598.8125, 2.977, 0.7684, -0.02408),
`helio` (heliocentric, Earth fixed, 0.06069, -0.008933, 0.0008199),
and the two one-sided cases (`after`, `before`; divisor 0.5). The cell is the
displacement of the component the mutant changes (the largest, for the
whole-motion mutants). `n/a` means the mutant does not act in that arm.

| Mutant | geo lin | seam | helio | one-sided after | one-sided before |
|--------|---------|------|-------|-----------------|------------------|
| 105:64 `+`→`-` | 3.65 | 5.955 | 0.1214 | n/a | n/a |
| 111:61 `*`→`+` | 1.095 | 1.786 | 0.03641 | n/a | n/a |
| 111:61 `*`→`/` | 5.476 | 8.932 | 0.1821 | n/a | n/a |
| 117:91 `/`→`%` | 1 | 2 | 0 | 0.02091 | 0.02091 |
| 117:91 `/`→`*` | 0 | 0 | 0 | 0.03136 | 0.03136 |
| 118:80 `/`→`%` | 0 | 0 | 0 | 0.002743 | 0.002744 |
| 118:80 `/`→`*` | 0 | 0 | 0 | 0.004115 | 0.004116 |
| 118:51 `-`→`+` | 19.66 | 27.9 | 2.284 | 27.66 | 27.65 |
| 118:51 `-`→`/` | 1.619 | 0.1765 | 1.001 | 2.006 | 2.006 |
| 120:48 `/`→`%` | 0 | 0 | 0 | 0.0002622 | 0.0002629 |
| 120:48 `/`→`*` | 0 | 0 | 0 | 0.0003933 | 0.0003943 |
| 120:43 `-`→`+` | 3.69 | 2.96 | 10.13 | 29.88 | 29.88 |
| 120:43 `-`→`/` | 0.9945 | 1.008 | 0.9993 | 2 | 2 |

Zeros are real: with a symmetric window `full = 1.0`, so `x / 1`, `x * 1` and
(for `|x| < 1`) `x % 1` agree. Those four `/` mutants on the lat and distance
speeds, and the `*` mutant on the lon speed, are killed **only** by the
one-sided cases, which is why `a_one_sided_probe_divides_by_half_a_day` exists.
The true minimum is **2.622e-4** (120:48 `/`→`%`, one-sided cases),
six orders above the 1e-10 tolerance on the distance speed. The other
non-zero mutants' minimum is **0.002743** (118:80 `/`→`%`).
The 120:13 `delete match arm` mutant turns the distance speed into `None`, which
`.unwrap()` rejects.

### `spherical_to_cartesian` (AU; tolerance 1e-12)

Best (largest) displacement over the test inputs (30,0,2), (60,30,2),
(359,-45,3), (77,90,4), (77,-90,4):

| Mutant | Best margin |
|--------|-------------|
| 173:11 `*`→`+` | 1.586 |
| 173:11 `*`→`/` | 1.469e16 |
| 173:23 `*`→`+` | 1.366 |
| 173:23 `*`→`/` | 2.598 |
| 174:11 `*`→`+` | 3.897 |
| 174:11 `*`→`/` | 6.365e16 |
| 174:23 `*`→`+` | 2.141 |
| 174:23 `*`→`/` | 121.5 |
| 175:11 `*`→`+` | 7 |
| 27 constant tuples `(±1/0, ±1/0, ±1/0)` | at least 4 (the pole input is (0, 0, ±4)) |

The true minimum is **1.366** (173:23 `*`→`+`).

### `cartesian_to_ecliptic` (r in AU, tolerance 1e-12)

| Mutant | (1,2,2) | (3,-4,12) | best |
|--------|---------|-----------|------|
| 180:28 `+`→`-` | 2 | NaN | 2 |
| 180:28 `+`→`*` | 1.472 | 47 | 47 |
| 180:20 `+`→`*` | 0.1716 | 3.971 | 3.971 |
| 180:16 `*`→`/` | 0 | 0.3114 | 0.3114 |
| 180:24 `*`→`/` | 0.5505 | 0.5903 | 0.5903 |
| 180:32 `*`→`+` | 0 | 6 | 6 |
| 185:12 `/`→`%` | NaN (`asin(2)`) | NaN (`asin(12)`) | NaN |
| 182:20 `==`→`!=` | lat 41.81 -> 0 | | caught; origin gives NaN |

The true minimum is **0.3114** (180:16, which is invisible for `x = 1` and is the
reason for the (3,-4,12) input).

### `orbital_plane_position` (a = 2, e = 0.6, E = 1; tolerance 1e-12)

| Mutant | Displacement |
|--------|--------------|
| 26:49 `-`→`+` | 2.4 (x) |
| 27:18 `*`→`/` | 0.7573 (y) |
| 27:25 `-`→`+` | 0.6163 (y) |
| 27:29 `*`→`+` | NaN (`sqrt(1 - 1.2)`) |

The true minimum numeric displacement is **0.6163** (27:25).

### `rotate_ecliptic_to_j2000` (AU; tolerance 1e-8 for the Meeus rows, bit-exact for the identity rows)

Largest component displacement, over the four (vector, equinox) rows:

| Mutant | J1900 A | J1900 B | B1950 A | B1950 B |
|--------|---------|---------|---------|---------|
| 12:28 `+`→`-` | 0.1054 | 0.4422 | 0.1047 | 0.4383 |
| 12:28 `+`→`*` | 0.196 | 0.1239 | 0.1946 | 0.1228 |
| 12:32 `*`→`+` | 0.1873 | 0.6635 | 0.186 | 0.6577 |
| 17:18 `/`→`%` | 0.1783 | 2.036 | 0.1783 | 2.018 |
| 17:18 `/`→`*` | 0.436 | 2.036 | 0.436 | 2.018 |
| 17:29 delete `-` | 1.217 | 3.831 | 1.208 | 3.831 |
| 24:11 `*`→`/` | 4.827 | 5.684 | 4.826 | 5.683 |

(The line numbers are those of the current `frame.rs`; the 12 and 17 rows were
the baseline's `12:28`, `12:32`, `17:18`, `17:29`, and the z component of the
output is `24:11`.) The true minimum is **0.1228** (12:28 `+`→`*`, B1950 B).

The identity-branch mutants (9:20 `-`→`+` and `-`→`/`, 9:45 `<`→`==` and `<`→`>`)
are killed two ways. For the `>` and `==` mutants the J1900 and B1950 rows turn
into the identity, a displacement of **0.01463** (B1950 A) to **0.04894**
(J1900 B). For the `-` mutants, which route J2000 through the precession path,
the displacement at J2000 +- 5e-7 d is **3.99e-13 to 6.64e-13** AU (the
precession shift of half a microday), asserted bit for bit by
`an_equinox_within_a_microday_of_j2000_returns_the_vector_untouched`. At exactly
J2000 the round trip alone moves the vector by at most 2.2e-16, so that input
alone cannot kill them; the offset inputs are why the test exists. These kills
rest on exact floating-point equality and are the least comfortable in this
slice; they hold because IEEE arithmetic is deterministic, and the mutated
path cannot reproduce `(1.2, -0.7, 0.4)` and `(-2.0, 1.5, -1.1)` to the bit at
both offsets.

### `solve_kepler` `<`→`>` (residual `|E - e sin E - M|`)

Starting Newton at `E = M` for `e >= 0.8` does not converge for Nibiru-like
eccentricities. Residuals of the mutant on the test grid: **1.147e6** (e =
0.981092, M = 7.5 deg), **44.42** (17 deg), **7.877e6** (343 deg) and **19.28**
(e = 0.99, 15 deg); the tolerance is 1e-12. The true minimum is **19.28**.

## Documented equivalents

- **`frame.rs:9:45` `<`→`<=`.** The branch is `(equinox_jd - J2000_JD).abs() <
  1e-6`. `<=` differs only if the difference equals `fl(1e-6)` exactly. Any
  `equinox_jd` within 1e-6 of J2000 lies on the grid of `ulp = 2^-31` (JD 2451545
  is in `[2^21, 2^22)`), so the difference is a multiple of `2^-31` (about
  4.66e-10); `fl(1e-6) = 4722366482869645 * 2^-72` is odd times `2^-72` and is not
  a multiple of `2^-31`. No input reaches the tie, so the two comparisons agree.
- **`kepler.rs:8:33` `<`→`<=` and `<`→`==`.** The line only chooses the start of
  the Newton iteration (`E = M` below `e = 0.8`, `E = pi` otherwise). `<=` changes
  the start only at `e == 0.8` exactly, and `==` starts at `pi` for every
  `e < 0.8`. Newton converges to the same root from either start for these
  (a scan of `e in {0, 0.1 ... 0.79, 0.8}` over 721 values of M converges within
  64 iterations from both), and the loop stops when the step is below 1e-14, so
  the returned `E` differs by rounding only. Making them differ would need an
  observation of the iteration count, which the function does not expose.
- **`kepler.rs:16:24` `<`→`<=` and `<`→`==`.** The line is the stopping rule
  `delta.abs() < 1e-14`. `<=` differs only when `|delta|` is exactly `1e-14`, and
  the next step would then move `E` by about `delta^2 = 1e-28`, below an ulp.
  `==` never stops early (only an exact zero step), so the loop runs all 64
  iterations; at a converged `E` each further Newton step changes it by less
  than the stopping tolerance, so the result differs by at most rounding noise
  of order 1e-14 rad, which is the solver's own declared tolerance.

All five are visible in the report (no `#[mutants::skip]`).

## Notes for follow-up

- The `kepler.rs` and `frame.rs` equivalents are all about stopping or branch
  tolerances. Exposing the iteration count (`solve_kepler` returning the number
  of steps) would let a test pin them, at the cost of a production API for tests
  only; not done.
- The independent reference script is not committed. If the references need
  regeneration, rebuild them from the formulas cited in the test comments
  (perifocal Kepler position, `Rz Rx Rz`, Meeus 21.5 and 21.7).
- The signed-zero difference in §1 (`-360 k` gives `-0.0` rather than `+0.0`) is
  unreachable from normalised longitudes. If a caller ever passes unnormalised
  longitudes, add `+ 0.0` after the remainder to restore `+0.0`.
