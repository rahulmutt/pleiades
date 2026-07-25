# FU-9 Houses Quadrant/Projection Mutant Triage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drive the **quadrant/projection family and the non-catalog tail** of `crates/pleiades-houses/src/systems/mod.rs` from a **measured 23 surviving mutants to a predicted 3 documented equivalents**, and split the crate's 3,209-line test file into a per-family `systems/tests/` directory first.

**Architecture:** Fifth PR of the ~6-PR `pleiades-houses` FU-9 campaign (spec: `docs/superpowers/specs/2026-07-22-fu9-houses-mutant-triage-design.md`, **PR 5 addendum**; prior PRs: Foundation `2026-07-22-…-foundation-…md`, Great-circle `2026-07-23-…-greatcircle-…md`, Sector `2026-07-24-…-sector-…md`, Sunshine `2026-07-24-…-sunshine-…md`). **Tests-only** — no production file is modified. Two independent authorities are used side by side: **Swiss-Ephemeris corpus rows** for whole-cusp-array pins, and an **extended `houses-reference.py`** (published WGS-84 constants; a bisection root-finder that is a genuinely different formulation from the crate's Newton iteration) for the pure numeric helpers.

**Tech Stack:** Rust (stable, via mise), cargo-nextest, cargo-mutants 27.1.0, Python 3 (reference only).

## Global Constraints

- **Tests-only:** no production-code change anywhere in this PR. The whole diff is `crates/pleiades-houses/src/systems/tests.rs` → `crates/pleiades-houses/src/systems/tests/**`, the new tests, the reference-note extension, and the `docs/follow-ups.md` Progress entry. `crates/pleiades-houses/src/systems/mod.rs` is **not** edited (its `#[cfg(test)] mod tests;` line already resolves to the new directory).
- **No parity-gate change:** no `validate-*` file is touched; the mutants tier stays report-only.
- **No `#[mutants::skip]`:** residual documented-equivalent mutants are left visible with a written reachability argument, enumerated in `quadrant_family_equivalent_mutants_are_documented`.
- **Independence discipline:** every expected value comes from the SE corpus (`crates/pleiades-validate/data/houses-corpus/cusps.csv`, copied as literals with a provenance comment naming the row) or from the independent Python port — never from running the function under test and pinning its output.
- **`mise.toml` untouched.** Adding `-p pleiades-houses` to `[tasks.mutants]` is the **crate-completing PR 6** step, not this one.
- **Branch:** `fu9-houses-quadrant-mutant-triage` off `main` (already created; the PR 5 design addendum is committed on it as `3d87e4725`).
- All commands run through mise: prefix cargo invocations with `mise exec --`.
- **Run `mise exec -- cargo fmt --all` before every commit.** Array/float literals written by hand are routinely not rustfmt-clean and the CI fmt gate is blocking.
- **cargo-mutants + mise trust gotcha (operational):** cargo-mutants copies the workspace to `/tmp/cargo-mutants-workspace-*.tmp` and this environment's mise refuses the untrusted copied `mise.toml`. Prefix every `cargo mutants` invocation with `MISE_TRUSTED_CONFIG_PATHS=/tmp`, or the run aborts at "build failed in an unmutated tree".

## Measured baseline (2026-07-24, at `a8917919f`)

Authoritative whole-file command (output directed outside the repo so the working tree stays clean):

```bash
MISE_TRUSTED_CONFIG_PATHS=/tmp mise exec -- cargo mutants \
  --test-tool nextest --test-workspace=false --baseline run \
  -p pleiades-houses --file crates/pleiades-houses/src/systems/mod.rs \
  -o /tmp/pr5-baseline
```

**1,128 mutants tested in 31 min — 83 missed / 1,038 caught / 7 unviable.** The 83 decompose with no remainder: **32** are prior slices' documented equivalents reproduced identically (Foundation 13, Great-circle 8, Sector 6, Sunshine 5), **28** are `catalog_name` (PR 6), and **23** are this PR's work:

| Function | Survivors | Lines (`systems/mod.rs`) |
|----------|-----------|--------------------------|
| `topocentric_latitude` | 9 | 1680:39 (`/`→`%`, `/`→`*`), 1680:46 (`-`→`+`, `-`→`/`), 1680:64 (`*`→`+`, `*`→`/`), 1680:74 (`*`→`+`, `*`→`/`), 1681:29 (`+`→`-`) |
| `solve_placidian_cusp` | 6 | 1739:37 (`*`→`/`), 1739:49 (`+`→`-`), 1741:21 (`<`→`==`, `<`→`<=`), 1750:24 (`<`→`<=`), 1756:19 (`\|\|`→`&&`) |
| `regiomontanus_houses` | 5 | 947:42 (`*`→`+`), 948:32 (`*`→`/`), 948:49 (`*`→`/`), 948:67 (`-`→`+`), 949:32 (`*`→`/`) |
| `koch_houses` | 1 | 843:35 (`-`→`+`) |
| `validate_topocentric_observer` | 1 | 618:5 (`-> Ok(())`) |
| `midpoint_longitude` | 1 | 1790:5 (`-> Default::default()`) |

**Predicted outcome: `23 → 3 documented equivalents`** — `solve_placidian_cusp` 1741 `<`→`<=` and 1750 `<`→`<=` (measure-zero comparison boundaries) plus `validate_topocentric_observer` 618 (proven unreachable, see Task 6). All 20 others are killed. **The true split is confirmed by the scoped re-run in Task 7, not by this prediction.**

## Verified inputs

Every literal and every crafted geometry below was executed against the crate at `a8917919f` during planning and **reproduced bit-for-bit**; no value in this plan is a guess.

| Input | Crate result |
|-------|--------------|
| `solve_placidian_cusp(90.0, 61.0, 23.4392811, {11,12,2,3})` | `129.43521015898668`, `158.60483237251546`, `201.39516762748454`, `230.56478984101335` |
| `solve_placidian_cusp(330.0, 81.776_683_964_516_9, 23.4392811, 11)` | `Err(NumericalFailure / "placidian cusp iteration encountered a zero derivative")` |
| `solve_placidian_cusp(18.0, 78.0, 23.4392811, 11)` | `Err(NumericalFailure / "placidian cusp iteration failed to converge")` |
| `topocentric_latitude(40.0, Some(1000.0))` / `(-33.0, Some(500.0))` | `39.810640281732304` / `-32.82446610604598` |
| `koch_houses(...)` at `lat = 70°` | `Err(NumericalFailure / "koch house system is undefined within the polar circle")` |
| `calculate_houses(Topocentric, elevation = NaN)` | `Err(InvalidElevation / "observer elevation must be finite when provided")` |
| Corpus residuals at `HEAD` | Regiomontanus `c1_lat40` 0.0424″, `c2_lat55` 0.0941″, Sripati `c1_lat40` 0.0092″ — all comfortably inside a 1″ assertion |

**Mutation-triage TDD cycle** (differs from feature TDD — read once): a triage test *passes* on correct `HEAD` code (it pins intent against the independent reference); it *kills a mutant* by failing on the mutated tree. Each task: write the test → run `cargo nextest` to confirm it **passes** on `HEAD` (proving the literal and geometry are right) → Task 7 re-runs `cargo mutants -F` to confirm the survivors are **caught**.

## File structure

| File | Responsibility |
|------|----------------|
| `crates/pleiades-houses/src/systems/tests/mod.rs` | submodule declarations only |
| `…/tests/support.rs` | shared helpers: `observer`, `sample_request`, `assert_close_degrees`, `test_asc_mc`, and the new `assert_corpus_cusps` |
| `…/tests/request.rs` | `HouseRequest` / `HouseSnapshot` validation and summary-line tests (12) |
| `…/tests/dispatch.rs` | system availability, custom systems, high-latitude policy, house assignment (12) |
| `…/tests/trivial.rs` | Equal / EqualMidheaven / EqualAries / Vehlow / WholeSign / Sripati / Porphyry (8) |
| `…/tests/quadrant.rs` | Placidus, Koch, Topocentric, Regiomontanus, Campanus, Alcabitius, Carter, Morinus, Meridian/Axial (15) **+ every new test in Tasks 2–6** |
| `…/tests/greatcircle.rs` | Horizon, APC, Krusinski-Pisa-Goelzer (7 + 4 helpers) |
| `…/tests/sector.rs` | Gauquelin, Albategnius, Pullen SD/SR (6 + 1 helper) |
| `…/tests/sunshine.rs` | Sunshine / solar-arc (10 + 2 helpers) |
| `…/tests/primitives.rs` | `asc1`, `asc2`, `asc_mc_from`, `spherical_cotrans`, chart points, longitude helpers (20) |

Deleted: `crates/pleiades-houses/src/systems/tests.rs`.

---

### Task 1: Split `systems/tests.rs` into a per-family `systems/tests/` directory

A pure move: **no test body changes, no test renames, no additions**. 3,209 lines / 90 tests / 11 helpers today, with Tasks 2–6 about to add more. Matches AGENTS.md ("split it before adding more, not after") and the `pleiades-types` slice precedent (`src/tests.rs` → `src/tests/` with a `mod.rs` of declarations and `use crate::*;` per submodule).

**Files:**
- Create: `crates/pleiades-houses/src/systems/tests/mod.rs`
- Create: `crates/pleiades-houses/src/systems/tests/{support,request,dispatch,trivial,quadrant,greatcircle,sector,sunshine,primitives}.rs`
- Delete: `crates/pleiades-houses/src/systems/tests.rs`
- Unchanged: `crates/pleiades-houses/src/systems/mod.rs` (its existing `#[cfg(test)] mod tests;` resolves to `tests/mod.rs` automatically)

**Interfaces:**
- Produces: `support::{observer, sample_request, assert_close_degrees, test_asc_mc}`, all `pub(super)` so sibling submodules can use them. Later tasks add tests to `tests/quadrant.rs` and a helper to `tests/support.rs`.

- [ ] **Step 1: Record the pre-move test inventory**

Run:
```bash
mise exec -- cargo nextest list -p pleiades-houses 2>/dev/null | sort > /tmp/tests-before.txt
wc -l /tmp/tests-before.txt
```
Expected: a sorted list of every test in the crate. Keep the file — Step 5 diffs against it.

- [ ] **Step 2: Create the directory and the module root**

```bash
mkdir -p crates/pleiades-houses/src/systems/tests
```

Create `crates/pleiades-houses/src/systems/tests/mod.rs`:

```rust
//! Unit tests for the `systems` module, split by house-system formula family.
//! Relocated from the former monolithic `systems/tests.rs` per AGENTS.md
//! ("split a large file before adding to it") and the `pleiades-types`
//! precedent. Shared setup lives in `support`; family-local helpers stay in
//! the family file that uses them.

mod dispatch;
mod greatcircle;
mod primitives;
mod quadrant;
mod request;
mod sector;
mod sunshine;
mod support;
mod trivial;
```

- [ ] **Step 3: Move the shared helpers into `support.rs`**

Create `crates/pleiades-houses/src/systems/tests/support.rs` containing **verbatim** the current `tests.rs` lines 1–48 preamble bodies — `observer`, `sample_request`, `assert_close_degrees`, `test_asc_mc` — with the import header rewritten for the new depth and every helper made `pub(super)`:

```rust
use crate::systems::*;
use pleiades_types::{JulianDay, Latitude, TimeScale};

pub(super) fn observer() -> ObserverLocation {
    ObserverLocation::new(
        Latitude::from_degrees(0.0),
        Longitude::from_degrees(0.0),
        None,
    )
}

pub(super) fn sample_request(system: HouseSystem) -> HouseRequest {
    HouseRequest::new(
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        observer(),
        system,
    )
}

pub(super) fn assert_close_degrees(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1.0e-12,
        "expected {expected}, got {actual}"
    );
}

pub(super) fn test_asc_mc(angles: HouseAngles) -> AscMc {
    AscMc {
        ascendant: angles.ascendant,
        midheaven: angles.midheaven,
        descendant: angles.descendant,
        imum_coeli: angles.imum_coeli,
        armc: angles.midheaven,
        vertex: angles.ascendant,
        antivertex: angles.descendant,
        equatorial_ascendant: angles.ascendant,
        coascendant_koch: angles.ascendant,
        coascendant_munkasey: angles.ascendant,
        polar_ascendant: angles.descendant,
    }
}
```

`crate::systems::*` reaches the module's private items because `systems::tests::support` is a descendant of `systems`.

- [ ] **Step 4: Move each `#[test] fn` block, verbatim, to its family file**

Move each test — together with its `#[test]` attribute and any preceding doc comment — from `systems/tests.rs` into the file named below. Move the family-local helpers with their family. **Do not edit a single test body.** Start every created file with:

```rust
use super::support::*;
use crate::systems::*;
use pleiades_types::{Angle, CustomHouseSystem, JulianDay, Latitude, TimeScale};
```

then delete whichever imports clippy reports as unused in Step 5 (each file needs a different subset).

| File | Tests (by current line number in `systems/tests.rs`) |
|------|------------------------------------------------------|
| `request.rs` | 51, 88, 94, 110, 124, 138, 152, 343, 355, 373, 961, 1018 |
| `dispatch.rs` | 253, 401, 419, 641, 659, 773, 1075, 1095, 1114, 1169, 1174, 1210 |
| `trivial.rs` | 164, 183, 195, 224, 244, 1243, 1882, 1920 |
| `quadrant.rs` | 269, 278, 288, 313, 440, 492, 503, 531, 554, 609, 1292, 1368, 1425, 1482, 1539 |
| `greatcircle.rs` | 684, 787, 2236, 2282, 2368, 2396, 2502 + helpers `gc_instant` (2268), `gc_angles` (2272), `recompose_horizon` (2321), `recompose_krusinski` (2469) |
| `sector.rs` | 807, 840, 2559, 2620, 2691, 2707 + helper `assert_sector_cusps` (2543) |
| `sunshine.rs` | 2763, 2778, 2793, 2939, 2971, 2998, 3027, 3049, 3092, 3191 + helpers `recompose_sunshine` (2839), `sun_geom` (2918) |
| `primitives.rs` | 1589, 1608, 1632, 1667, 1684, 1703, 1734, 1759, 1778, 1797, 1868, 1903, 1911, 1937, 1948, 1960, 1985, 2018, 2079, 2114 |

12 + 12 + 8 + 15 + 7 + 6 + 10 + 20 = **90 tests**, the full inventory. Then delete the now-empty `crates/pleiades-houses/src/systems/tests.rs`.

- [ ] **Step 5: Prove the move is a no-op**

Run:
```bash
mise exec -- cargo fmt --all
mise exec -- cargo clippy -p pleiades-houses --all-targets --all-features -- -D warnings
mise exec -- cargo nextest list -p pleiades-houses 2>/dev/null | sort > /tmp/tests-after.txt
diff <(sed 's/systems::tests::[a-z_]*::/systems::tests::/' /tmp/tests-before.txt) \
     <(sed 's/systems::tests::[a-z_]*::/systems::tests::/' /tmp/tests-after.txt)
```
Expected: clippy clean, and the `diff` produces **no output** — identical test names once the newly-inserted family segment is normalised away. If clippy reports unused imports, delete exactly those imports and re-run.

- [ ] **Step 6: Run the full crate suite**

Run: `mise exec -- cargo nextest run -p pleiades-houses`
Expected: every test passes, and the count equals `wc -l < /tmp/tests-before.txt` from Step 1 — the move adds and removes nothing.

- [ ] **Step 7: Commit**

```bash
mise exec -- cargo fmt --all
git add crates/pleiades-houses/src/systems/
git commit -m "test(houses): split systems/tests.rs into a per-family tests/ directory

Pure move, no test body changes: 3,209 lines / 90 tests relocated into
systems/tests/{support,request,dispatch,trivial,quadrant,greatcircle,
sector,sunshine,primitives}.rs before the FU-9 PR 5 tests are added.
Verified a no-op by identical cargo-nextest inventories."
```

---

### Task 2: `topocentric_latitude` — 9 survivors → 0

`topocentric_latitude(latitude_deg, elevation_m)` (mod.rs:1656–1684) is the WGS-84 geodetic→geocentric reduction. All 9 survivors sit in the prime-vertical radius (1680) and the `+ elevation` term of `x` (1681). The existing `topocentric_latitude_uses_geocentric_correction` test never passes a non-zero elevation, so `(N + h)` and `(N - h)` are indistinguishable; and it does not pin the radius tightly enough to constrain the eccentricity terms.

**Files:**
- Modify: `docs/superpowers/specs/notes/2026-07-22-houses-reference.py` (append)
- Test: `crates/pleiades-houses/src/systems/tests/quadrant.rs` (append)

**Interfaces:**
- Consumes: `topocentric_latitude(latitude_deg: f64, elevation_m: Option<f64>) -> Result<Angle, HouseError>` (`pub(crate)`, reachable via `use crate::systems::*;`); `assert_close_degrees` from `support` (tolerance `1e-12`).
- Produces: nothing consumed by later tasks.

- [ ] **Step 1: Extend the independent reference**

Append to `docs/superpowers/specs/notes/2026-07-22-houses-reference.py`:

```python
# --- PR 5: topocentric_latitude (WGS-84 geodetic -> geocentric) -------------
# Constants from the published WGS-84 datum sheet, NOT read from the crate.
WGS84_A = 6_378_137.0
WGS84_INV_F = 298.257_223_563


def topocentric_latitude(lat_deg, h=0.0):
    """Prime-vertical form: phi' = atan2((N(1-e2)+h) sin phi, (N+h) cos phi)."""
    f = 1.0 / WGS84_INV_F
    e2 = f * (2.0 - f)
    phi = math.radians(lat_deg)
    s, c = math.sin(phi), math.cos(phi)
    n = WGS84_A / math.sqrt(1.0 - e2 * s * s)
    return math.degrees(math.atan2((n * (1.0 - e2) + h) * s, (n + h) * c))


def topocentric_latitude_closed_form(lat_deg):
    """Second, genuinely different formulation, exact for h = 0:
    tan(phi') = (1 - f)^2 tan(phi).  Cross-checks the prime-vertical form."""
    f = 1.0 / WGS84_INV_F
    return math.degrees(math.atan((1.0 - f) ** 2 * math.tan(math.radians(lat_deg))))


if __name__ == '__main__':
    for lat, h in ((40.0, 1000.0), (-33.0, 500.0)):
        print(f'topocentric_latitude({lat}, {h}) = {topocentric_latitude(lat, h)!r}')
    for lat in (40.0, 55.0, -33.0, 66.0):
        a = topocentric_latitude(lat)
        b = topocentric_latitude_closed_form(lat)
        print(f'  h=0 lat={lat}: prime-vertical={a!r} closed-form={b!r} diff={abs(a-b):.3e}')
```

Run: `python3 docs/superpowers/specs/notes/2026-07-22-houses-reference.py | tail -8`
Expected: `topocentric_latitude(40.0, 1000.0) = 39.810640281732304`, `topocentric_latitude(-33.0, 500.0) = -32.82446610604598`, and every `h=0` cross-check `diff=0.000e+00` — the two independent formulations agree exactly, which is what licenses the 1e-12 pins.

- [ ] **Step 2: Write the tests**

Append to `crates/pleiades-houses/src/systems/tests/quadrant.rs`:

```rust
/// FU-9: pins the WGS-84 reduction against an independent evaluation of the
/// published datum constants (`houses-reference.py::topocentric_latitude`).
///
/// The elevation MUST be non-zero: with `h = 0` the `(N + h)` term at
/// mod.rs:1681 is degenerate and its `+ -> -` mutant is invisible.
#[test]
fn topocentric_latitude_pins_the_wgs84_reduction_with_elevation() {
    assert_close_degrees(
        topocentric_latitude(40.0, Some(1000.0))
            .expect("finite elevation is accepted")
            .degrees(),
        39.810_640_281_732_304,
    );
    assert_close_degrees(
        topocentric_latitude(-33.0, Some(500.0))
            .expect("finite elevation is accepted")
            .degrees(),
        -32.824_466_106_045_98,
    );
}

/// FU-9: cross-checks the prime-vertical form against a second, genuinely
/// different published formulation, `tan(phi') = (1 - f)^2 * tan(phi)`, which
/// is exact at sea level. Constrains the eccentricity terms at mod.rs:1680
/// independently of the elevation pins above.
#[test]
fn topocentric_latitude_at_sea_level_matches_the_closed_form() {
    let flattening = 1.0 / 298.257_223_563;
    let one_minus_f_squared = (1.0 - flattening) * (1.0 - flattening);
    for latitude in [40.0_f64, 55.0, -33.0, 66.0] {
        let expected = (one_minus_f_squared * latitude.to_radians().tan())
            .atan()
            .to_degrees();
        assert_close_degrees(
            topocentric_latitude(latitude, None)
                .expect("absent elevation is accepted")
                .degrees(),
            expected,
        );
    }
}
```

- [ ] **Step 3: Confirm both tests PASS on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses topocentric_latitude`
Expected: PASS — including the two pre-existing `topocentric_latitude_*` tests.

- [ ] **Step 4: Commit**

```bash
mise exec -- cargo fmt --all
git add crates/pleiades-houses/src/systems/tests/quadrant.rs docs/superpowers/specs/notes/2026-07-22-houses-reference.py
git commit -m "test(houses): pin topocentric_latitude against independent WGS-84 reference

Kills the 9 FU-9 survivors at mod.rs:1680-1681 (prime-vertical radius and
the + elevation term) with non-zero-elevation pins from the published WGS-84
constants, cross-checked against the closed-form (1-f)^2 tan(phi) identity."
```

**Per-mutant margin table** (displacement at `lat = 40°, h = 1000 m`, versus the `1e-12` assertion tolerance — never aggregated, per the campaign discipline):

| Mutant | Mutated value | Displacement |
|--------|---------------|--------------|
| 1680:39 `/`→`%` | `39.99996875212803` | 1.89e-01 |
| 1680:39 `/`→`*` | `39.810640364178745` | 8.24e-08 |
| 1680:46 `-`→`+` | `39.810640364064724` | 8.23e-08 |
| 1680:46 `-`→`/` | `39.81117502426418` | 5.35e-04 |
| 1680:64 `*`→`+` | `39.81063327491272` | 7.01e-06 |
| 1680:64 `*`→`/` | `39.8106402231261` | 5.86e-08 |
| 1680:74 `*`→`+` | `39.81062823886763` | 1.20e-05 |
| 1680:74 `*`→`/` | `39.8106402231261` | 5.86e-08 |
| 1681:29 `+`→`-` | `39.81946447640497` | 8.82e-03 |

True minimum displacement **5.86e-08**, a ~5.9e4× margin over the tolerance.

---

### Task 3: Regiomontanus + Sripati SE corpus anchors — 6 survivors → 0

Two independent root causes, one test shape:

- `regiomontanus_houses` (5 survivors at 947–949): the only existing Regiomontanus test, `regiomontanus_campanus_and_koch_reduce_to_sidereal_phase_spacing_on_the_equator`, runs at **lat 0**, where `latitude.sin() = 0` and `latitude.cos() = 1` collapse three of the four factors. A non-equatorial pin restores them.
- `midpoint_longitude` (1 survivor at 1790): the existing `sripati_midpoints_follow_porphyry_segments` test asserts `sripati cusps == midpoint_longitude(...)` — **the same function on both sides** — so replacing it with `Default::default()` yields `0 == 0` and still passes. A Swiss-Ephemeris Sripati row breaks the circularity, because SE never calls our function.

**Files:**
- Modify: `crates/pleiades-houses/src/systems/tests/support.rs` (append the shared helper)
- Test: `crates/pleiades-houses/src/systems/tests/quadrant.rs` (Regiomontanus), `crates/pleiades-houses/src/systems/tests/trivial.rs` (Sripati)

**Interfaces:**
- Consumes: `calculate_houses(&HouseRequest) -> Result<HouseSnapshot, HouseError>`.
- Produces: `support::assert_corpus_cusps(label: &str, system: HouseSystem, latitude_deg: f64, expected: [f64; 12])` — `pub(super)`, available to every family file.

Scope note: the five pre-existing `*_match_swiss_ephemeris_corpus_*` tests keep their own inline closures. Retrofitting them onto the new helper is a separate maintainability change, deliberately **not** bundled into a triage PR.

- [ ] **Step 1: Add the shared corpus helper**

Append to `crates/pleiades-houses/src/systems/tests/support.rs`:

```rust
/// Asserts all twelve cusps against a Swiss-Ephemeris corpus row.
///
/// Expected values are copied from
/// `crates/pleiades-validate/data/houses-corpus/cusps.csv` (SE 2.10.03,
/// cross-checked against Astrolog 7.70) with the row named at the call site.
/// The crate does not read that file — it belongs to the tooling crate, and a
/// path dependency from a domain crate onto it would invert the layering.
///
/// Tolerance 1 arcsec, matching the existing in-crate corpus anchors; measured
/// residuals at `HEAD` are 0.009-0.094 arcsec.
pub(super) fn assert_corpus_cusps(
    label: &str,
    system: HouseSystem,
    latitude_deg: f64,
    expected: [f64; 12],
) {
    const TOLERANCE_ARCSEC: f64 = 1.0;

    let request = HouseRequest::new(
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        ObserverLocation::new(
            Latitude::from_degrees(latitude_deg),
            Longitude::from_degrees(0.0),
            None,
        ),
        system,
    );
    let snapshot = calculate_houses(&request).expect("corpus chart should compute");

    for (index, &expected_deg) in expected.iter().enumerate() {
        let actual = snapshot.cusps[index].degrees();
        let wrapped = (actual - expected_deg).rem_euclid(360.0);
        let signed = if wrapped > 180.0 {
            wrapped - 360.0
        } else {
            wrapped
        };
        let arcsec = signed.abs() * 3600.0;
        assert!(
            arcsec < TOLERANCE_ARCSEC,
            "{label} cusp {} = {actual:.6}° differs from SE {expected_deg:.6}° \
             by {arcsec:.4}″ (limit {TOLERANCE_ARCSEC}″)",
            index + 1,
        );
    }
}
```

- [ ] **Step 2: Write the Regiomontanus test**

Append to `crates/pleiades-houses/src/systems/tests/quadrant.rs`:

```rust
/// FU-9: the pre-existing Regiomontanus coverage runs at lat 0, where
/// `sin(lat) = 0` and `cos(lat) = 1` make three of the four factors at
/// mod.rs:947-949 degenerate. These two non-equatorial charts restore them.
#[test]
fn regiomontanus_cusps_match_swiss_ephemeris_corpus() {
    // houses-corpus/cusps.csv row c1_lat40 (JD 2451545.0, lat 40N, lon 0).
    assert_corpus_cusps(
        "Regiomontanus c1_lat40",
        HouseSystem::Regiomontanus,
        40.0,
        [
            17.706_103,
            57.771_262,
            81.547_840,
            99.611_088,
            119.384_226,
            149.836_713,
            197.706_103,
            237.771_262,
            261.547_840,
            279.611_088,
            299.384_226,
            329.836_713,
        ],
    );
    // houses-corpus/cusps.csv row c2_lat55 (JD 2451545.0, lat 55N, lon 0).
    assert_corpus_cusps(
        "Regiomontanus c2_lat55",
        HouseSystem::Regiomontanus,
        55.0,
        [
            28.505_186,
            72.373_244,
            88.608_626,
            99.611_088,
            112.251_819,
            138.090_289,
            208.505_186,
            252.373_244,
            268.608_626,
            279.611_088,
            292.251_819,
            318.090_289,
        ],
    );
}
```

- [ ] **Step 3: Write the Sripati test**

Append to `crates/pleiades-houses/src/systems/tests/trivial.rs`:

```rust
/// FU-9: `sripati_midpoints_follow_porphyry_segments` compares the crate's
/// Sripati cusps against `midpoint_longitude` — the same function on both
/// sides — so the `-> Default::default()` mutant at mod.rs:1790 makes both
/// sides `0` and still passes. Swiss Ephemeris never calls our function, so
/// this row breaks the circularity.
#[test]
fn sripati_cusps_match_swiss_ephemeris_corpus() {
    // houses-corpus/cusps.csv row c1_lat40 (JD 2451545.0, lat 40N, lon 0).
    assert_corpus_cusps(
        "Sripati c1_lat40",
        HouseSystem::Sripati,
        40.0,
        [
            1.356_934,
            31.356_934,
            58.658_595,
            85.960_257,
            115.960_257,
            148.658_595,
            181.356_934,
            211.356_934,
            238.658_595,
            265.960_257,
            295.960_257,
            328.658_595,
        ],
    );
}
```

- [ ] **Step 4: Confirm both tests PASS on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses regiomontanus_cusps_match sripati_cusps_match`
Expected: PASS, 2 tests. (Measured residuals during planning: Regiomontanus 0.0424″ / 0.0941″, Sripati 0.0092″ — all well inside the 1″ limit.)

- [ ] **Step 5: Commit**

```bash
mise exec -- cargo fmt --all
git add crates/pleiades-houses/src/systems/tests/
git commit -m "test(houses): anchor Regiomontanus and Sripati to the SE corpus

Kills the 5 regiomontanus_houses survivors (the only prior coverage ran at
lat 0, where three of four factors are degenerate) and the midpoint_longitude
survivor (the Sripati test compared the function against itself)."
```

---

### Task 4: `koch_houses` polar-circle guard — 1 survivor → 0

The 843:35 `-`→`+` mutant moves the polar-circle threshold from `90 - ε ≈ 66.56°` to `90 + ε ≈ 113.44°`, which no `|lat| ≤ 90` can reach, so the fail-closed guard never fires.

**This guard is unreachable through `calculate_houses`.** The catalog gives Koch `max_abs_latitude_deg = Some(66.0)` (`catalog/mod.rs:712–720`), which is *below* the polar circle: under `Strict` the request is rejected first, and under `SwissEphemerisFallback` Porphyry is substituted. Neither path enters `koch_houses`. The kill is therefore a direct call on the private function — the `apparent.rs` private-primitive precedent.

**Files:**
- Test: `crates/pleiades-houses/src/systems/tests/quadrant.rs` (append)

**Interfaces:**
- Consumes: `koch_houses(instant: Instant, observer: &ObserverLocation, obliquity: Angle, angles: HouseAngles) -> Result<[Longitude; 12], HouseError>` and `derive_angles(instant: Instant, observer: &ObserverLocation, obliquity: Angle) -> HouseAngles`, both private in `systems`.

- [ ] **Step 1: Write the test**

Append to `crates/pleiades-houses/src/systems/tests/quadrant.rs`:

```rust
/// FU-9: Koch is undefined inside the polar circle, where the Midheaven's
/// ascensional difference stops being real, and mod.rs:843 fails closed on
/// `|lat| >= 90 - obliquity`. The catalog caps Koch at |lat| 66°, *below* the
/// 66.56° polar circle, so `calculate_houses` rejects (Strict) or substitutes
/// Porphyry (SwissEphemerisFallback) before this guard is ever reached — the
/// guard is only observable by calling the private function directly.
#[test]
fn koch_houses_fails_closed_inside_the_polar_circle() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let observer = ObserverLocation::new(
        Latitude::from_degrees(70.0),
        Longitude::from_degrees(0.0),
        None,
    );
    let obliquity = Angle::from_degrees(23.4392811);
    let angles = derive_angles(instant, &observer, obliquity);

    let error = koch_houses(instant, &observer, obliquity, angles)
        .expect_err("Koch inside the polar circle must fail closed");
    assert_eq!(error.kind, HouseErrorKind::NumericalFailure);
    assert!(
        error.message.contains("undefined within the polar circle"),
        "unexpected message: {}",
        error.message
    );
}
```

- [ ] **Step 2: Confirm it PASSES on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses koch_houses_fails_closed_inside_the_polar_circle`
Expected: PASS. (Verified during planning: the crate returns `NumericalFailure / "koch house system is undefined within the polar circle"`.)

- [ ] **Step 3: Commit**

```bash
mise exec -- cargo fmt --all
git add crates/pleiades-houses/src/systems/tests/quadrant.rs
git commit -m "test(houses): pin Koch's polar-circle fail-closed guard

Kills the 843:35 survivor, which moves the threshold to an unreachable
113.44 degrees. Tested at the private seam because the catalog's 66-degree
Koch bound sits below the 66.56-degree polar circle, so no public request
reaches the guard."
```

---

### Task 5: `solve_placidian_cusp` — 6 survivors → 2 documented equivalents

Placidus solves `g(q) = cos(q/f) + tan(φ)·tan(δ(α)) = 0` by Newton iteration. The survivors are all *internal*: the two derivative terms, the zero-derivative guard, the convergence test, and the fail-closed exit.

The key insight, verified numerically during planning: pinning the **root** against an independent bisection at a tight tolerance kills the derivative mutants by itself. A mutated derivative still converges, but it stops when its own `|delta| < 1e-9`, leaving an error of order `1e-10`, whereas `HEAD`'s quadratic convergence lands within `~1e-14` of the true root. No divergence hunt is needed.

**Files:**
- Modify: `docs/superpowers/specs/notes/2026-07-22-houses-reference.py` (append)
- Test: `crates/pleiades-houses/src/systems/tests/quadrant.rs` (append)

**Interfaces:**
- Consumes: `solve_placidian_cusp(st_deg: f64, latitude_deg: f64, obliquity_deg: f64, house: usize) -> Result<Longitude, HouseError>` (private).

- [ ] **Step 1: Extend the independent reference**

Append to `docs/superpowers/specs/notes/2026-07-22-houses-reference.py`:

```python
# --- PR 5: Placidus cusps by BISECTION -------------------------------------
# Same published residual as the crate, but a genuinely different root-finder:
#   g(q) = cos(q/f) + tan(phi) * tan(delta(alpha)),  alpha = RAMC + q,
#   tan(delta) = sin(alpha) * tan(eps)
# The crate uses Newton; bisection cannot inherit a Newton-specific error.
PLACIDUS_SPEC = {11: (1.0 / 3.0, 1.0, False), 12: (2.0 / 3.0, 1.0, False),
                 2: (2.0 / 3.0, -1.0, True), 3: (1.0 / 3.0, -1.0, True)}


def placidus_residual(q, st, tan_lat, tan_obl, fraction):
    alpha = math.radians(st + q)
    return math.cos(math.radians(q / fraction)) + tan_lat * math.sin(alpha) * tan_obl


def placidus_cusp_bisection(st, lat_deg, obl_deg, house):
    fraction, sign, is_opposite = PLACIDUS_SPEC[house]
    tan_lat = math.tan(math.radians(lat_deg))
    tan_obl = math.tan(math.radians(obl_deg))
    seed = sign * fraction * 90.0
    lo = hi = None
    step, prev_q = 0.001, seed - 60.0
    prev = placidus_residual(prev_q, st, tan_lat, tan_obl, fraction)
    for i in range(1, int(120.0 / step) + 1):
        q = seed - 60.0 + i * step
        cur = placidus_residual(q, st, tan_lat, tan_obl, fraction)
        if (prev < 0) != (cur < 0):
            if lo is None or abs((prev_q + q) / 2 - seed) < abs((lo + hi) / 2 - seed):
                lo, hi = prev_q, q
        prev_q, prev = q, cur
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        if (placidus_residual(lo, st, tan_lat, tan_obl, fraction)
                * placidus_residual(mid, st, tan_lat, tan_obl, fraction)) <= 0:
            hi = mid
        else:
            lo = mid
    q = 0.5 * (lo + hi)
    ra = math.radians(st + q)
    lon = math.degrees(math.atan2(math.sin(ra),
                                  math.cos(ra) * math.cos(math.radians(obl_deg)))) % 360.0
    return (lon + 180.0) % 360.0 if is_opposite else lon


if __name__ == '__main__':
    for house in (11, 12, 2, 3):
        print(f'placidus house {house}: '
              f'{placidus_cusp_bisection(90.0, 61.0, 23.4392811, house)!r}')
```

Run: `python3 docs/superpowers/specs/notes/2026-07-22-houses-reference.py | tail -4`
Expected: `129.43521015898665`, `158.60483237251546`, `201.39516762748454`, `230.56478984101335`.

- [ ] **Step 2: Write the root-pinning test**

Append to `crates/pleiades-houses/src/systems/tests/quadrant.rs`:

```rust
/// FU-9: pins all four solved Placidus cusps against an independent BISECTION
/// root of the published residual (`houses-reference.py`), not against the
/// crate's own Newton output.
///
/// Geometry `RAMC = 90°, lat = 61°, eps = 23.4392811°` was chosen by search to
/// maximise the weakest derivative-mutant displacement. It also kills the two
/// derivative-term mutants at mod.rs:1739 without any divergence hunt: a
/// mutated derivative still converges, but stops at its own `|delta| < 1e-9`
/// leaving an error of order 1e-10, while HEAD's quadratic convergence lands
/// within ~1e-14 of the true root. Tolerance 1e-11 sits between the two.
#[test]
fn solve_placidian_cusp_matches_an_independent_bisection_root() {
    const TOLERANCE_DEG: f64 = 1.0e-11;

    let cases = [
        (11_usize, 129.435_210_158_986_65),
        (12, 158.604_832_372_515_46),
        (2, 201.395_167_627_484_54),
        (3, 230.564_789_841_013_35),
    ];

    for (house, expected) in cases {
        let cusp = solve_placidian_cusp(90.0, 61.0, 23.4392811, house)
            .expect("this geometry converges");
        let difference = (cusp.degrees() - expected).abs();
        assert!(
            difference < TOLERANCE_DEG,
            "house {house}: {} differs from the bisection root {expected} by {difference:e}",
            cusp.degrees(),
        );
    }
}
```

- [ ] **Step 3: Write the two fail-closed guard tests**

Append to `crates/pleiades-houses/src/systems/tests/quadrant.rs`:

```rust
/// FU-9: the zero-derivative guard at mod.rs:1741 fails closed when the Newton
/// derivative vanishes. `gp = (-(1/f)·sin(q/f) + tan(φ)·tan(ε)·cos(α)) / (180/π)`.
/// At the house-11 seed (`q = 30°`, so `q/f = 90°` and `sin = 1`) with
/// `RAMC = 330°` the cosine factor is `cos(360°) = 1`, so `gp` vanishes exactly
/// when `tan(φ)·tan(ε) = 1/f`. Latitude is a free test input, so solving that
/// equation for it gives 81.776_683_964_516_9°, where `|gp| ~ 1.2e-16 < 1e-12`.
///
/// This distinguishes the `<` -> `==` mutant, which disables the guard for
/// every genuinely near-zero derivative: HEAD reports "zero derivative", the
/// mutant falls through to the non-convergence exit and reports "failed to
/// converge". Both are `NumericalFailure`, so only the diagnostic message
/// separates them — this file already pins error-message text in five places
/// (`request.rs`), so pinning it here is the established practice.
#[test]
fn solve_placidian_cusp_fails_closed_on_a_vanishing_derivative() {
    let error = solve_placidian_cusp(330.0, 81.776_683_964_516_9, 23.4392811, 11)
        .expect_err("a vanishing derivative must fail closed");
    assert_eq!(error.kind, HouseErrorKind::NumericalFailure);
    assert!(
        error.message.contains("zero derivative"),
        "unexpected message: {}",
        error.message
    );
}

/// FU-9: the fail-closed exit at mod.rs:1756 is `!converged || !q.is_finite()`.
/// At lat 78° the product `|tan(φ)·tan(δ)|` exceeds 1 over much of the range,
/// so `cos(q/f) = -tan(φ)·tan(δ)` has no solution and the iteration oscillates
/// without converging while `q` stays **finite** (~39.6). That combination —
/// `converged == false`, `q` finite — is exactly what the `||` -> `&&` mutant
/// needs to slip through: with `&&` the guard is false and the function returns
/// a garbage `Ok` instead of failing closed.
#[test]
fn solve_placidian_cusp_fails_closed_when_the_iteration_does_not_converge() {
    let error = solve_placidian_cusp(18.0, 78.0, 23.4392811, 11)
        .expect_err("a non-converging geometry must fail closed");
    assert_eq!(error.kind, HouseErrorKind::NumericalFailure);
    assert!(
        error.message.contains("failed to converge"),
        "unexpected message: {}",
        error.message
    );
}
```

- [ ] **Step 4: Confirm all three tests PASS on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses solve_placidian_cusp`
Expected: PASS, 3 tests. (All three were executed against the crate during planning and reproduced exactly.)

- [ ] **Step 5: Commit**

```bash
mise exec -- cargo fmt --all
git add crates/pleiades-houses/src/systems/tests/quadrant.rs docs/superpowers/specs/notes/2026-07-22-houses-reference.py
git commit -m "test(houses): pin Placidus roots and both fail-closed guards

Kills 4 of the 6 solve_placidian_cusp survivors: the two derivative terms
(1739) via a 1e-11 pin against an independent bisection root, the
zero-derivative guard (1741 < -> ==) via a crafted vanishing-derivative
geometry, and the fail-closed exit (1756 || -> &&) via a non-converging
geometry whose q stays finite."
```

**Per-mutant margin table** (displacement at `RAMC = 90°, lat = 61°`, versus the `1e-11` tolerance):

| Mutant | House 11 | House 12 | House 2 | House 3 |
|--------|----------|----------|---------|---------|
| 1739:49 `+`→`-` | 3.41e-10 | 3.83e+01 | 3.83e+01 | 3.41e-10 |
| 1739:37 `*`→`/` | 5.73e-10 | 1.04e-09 | 1.04e-09 | 5.73e-10 |

True minimum displacement **3.41e-10**, a ~34× margin; `HEAD` agrees with the bisection root to **≤ 2.84e-14**, a ~352× margin on the passing side.

---

### Task 6: Document the residual equivalent mutants

Three survivors are expected to remain. Each is left **visible** (no `#[mutants::skip]`) with a written reachability argument, following `sector_equivalent_mutants_are_documented` and `sunshine_family_equivalent_mutants_are_documented`.

**Files:**
- Test: `crates/pleiades-houses/src/systems/tests/quadrant.rs` (append)

- [ ] **Step 1: Write the characterization test**

Append to `crates/pleiades-houses/src/systems/tests/quadrant.rs`:

```rust
/// FU-9 quadrant/projection residual: 3 surviving mutants, each an EQUIVALENT
/// MUTANT left visible (no `#[mutants::skip]`), enumerated with a reachability
/// argument. Confirmed by the authoritative scoped run recorded in
/// `docs/follow-ups.md`.
///
/// --- validate_topocentric_observer (1) ---
/// (VT-1) 618:5 `-> Ok(())`. `validated_obliquity` calls `validate_observer`
///   BEFORE `validate_topocentric_observer` (mod.rs:604-605), and
///   `validate_observer` maps `NonFiniteElevation` to
///   `HouseError { kind: InvalidElevation, message: "observer elevation must
///   be finite when provided" }` for EVERY system. A non-finite elevation is
///   `topocentric_latitude`'s only error path, and its message is the same
///   string, so the earlier validator produces a byte-identical error: no
///   input can reach this function in a state where it would return `Err`.
///   The function is redundant defensive validation. Deleting it is the real
///   fix but is a production change, out of scope for a tests-only slice.
///
/// --- solve_placidian_cusp (2) ---
/// (PL-1) 1741:21 `gp.abs() < 1e-12 -> <=`: differs only when `gp.abs()` is
///   exactly `1e-12`. Latitude is a free input, so the free-parameter lens was
///   applied: near the vanishing-derivative latitude, one ulp of latitude
///   moves `gp` by ~5e-15, while `1e-12` has an ulp of ~2e-28 — the reachable
///   `gp` values step straight past the boundary, ~1e14 times coarser than the
///   target. Measure-zero and unreachable.
/// (PL-2) 1750:24 `delta.abs() < 1e-9 -> <=`: doubly unreachable. It differs
///   only at `|delta| == 1e-9` exactly, and even there the output is
///   bit-identical: `q` has ALREADY been updated by the time the test runs, so
///   the only difference is one extra Newton step, whose correction is of
///   order `delta^2 ~ 1e-18` — far below `ulp(q) ~ 3.6e-15` at `q ~ 30`. The
///   extra iteration cannot change a single bit of `q`.
///
/// The live paths all three operators share are pinned by the sibling tests:
#[test]
fn quadrant_family_equivalent_mutants_are_documented() {
    // VT-1: the earlier validator wins, with the identical kind AND message.
    let request = HouseRequest::new(
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        ObserverLocation::new(
            Latitude::from_degrees(40.0),
            Longitude::from_degrees(0.0),
            Some(f64::NAN),
        ),
        HouseSystem::Topocentric,
    );
    let error = calculate_houses(&request).expect_err("a NaN elevation is rejected");
    assert_eq!(error.kind, HouseErrorKind::InvalidElevation);
    assert_eq!(error.message, "observer elevation must be finite when provided");

    // PL-1 / PL-2: the guards' live path — a physical geometry converges to Ok,
    // and the two reachable failure exits are pinned by the sibling fail-closed
    // tests above.
    assert!(solve_placidian_cusp(90.0, 61.0, 23.4392811, 11).is_ok());
}
```

- [ ] **Step 2: Confirm it PASSES on HEAD**

Run: `mise exec -- cargo nextest run -p pleiades-houses quadrant_family_equivalent_mutants_are_documented`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
mise exec -- cargo fmt --all
git add crates/pleiades-houses/src/systems/tests/quadrant.rs
git commit -m "test(houses): document the 3 quadrant-family equivalent mutants

validate_topocentric_observer 618 (an earlier validator returns a
byte-identical error), and the two measure-zero comparison boundaries in
solve_placidian_cusp (1741 <=, 1750 <=). All left visible, no mutants::skip."
```

---

### Task 7: Verify by mutation, record the outcome, and run full CI

- [ ] **Step 1: Scoped mutation re-run**

Run:
```bash
MISE_TRUSTED_CONFIG_PATHS=/tmp mise exec -- cargo mutants \
  --test-tool nextest --test-workspace=false --baseline run \
  -p pleiades-houses --file crates/pleiades-houses/src/systems/mod.rs \
  -F 'in (topocentric_latitude|solve_placidian_cusp|regiomontanus_houses|koch_houses|validate_topocentric_observer|midpoint_longitude)$' \
  -o /tmp/pr5-verify
```
Expected: `3 missed` — exactly `618:5`, `1741:21 <=`, and `1750:24 <=`. Inspect with `cat /tmp/pr5-verify/mutants.out/missed.txt`.

**If a survivor is NOT in that list**, it was mispredicted: re-read its line, craft a killing input the same way (independent reference or crafted branch), and re-run. **If one of the three predicted equivalents is instead reported caught**, delete its entry from `quadrant_family_equivalent_mutants_are_documented` and say so in Step 3. The measurement, not this plan, is authoritative.

- [ ] **Step 2: Whole-file confirmation re-run**

Run:
```bash
MISE_TRUSTED_CONFIG_PATHS=/tmp mise exec -- cargo mutants \
  --test-tool nextest --test-workspace=false --baseline run \
  -p pleiades-houses --file crates/pleiades-houses/src/systems/mod.rs \
  -o /tmp/pr5-final
```
Expected: `63 missed` (was 83) — the 32 prior-slice documented equivalents, the 28 `catalog_name` survivors left for PR 6, and this PR's 3. Confirms no prior slice regressed.

- [ ] **Step 3: Append the FU-9 Progress entry**

Add a `**Progress (2026-07-24) — houses Quadrant/projection…**` entry to the FU-9 section of `docs/follow-ups.md`, in the established format of the four prior houses entries. It must record: the re-measured whole-file baseline (1,128 tested / 83 missed) and its three-way decomposition; the `23 → 3` result with the per-function split; that the SE-corpus/reference hybrid was used and why; the two per-mutant margin tables from Tasks 2 and 5; the running documented-equivalent tally updated to **35** (32 prior + 3); and the following correction:

> **Correction to the Sector slice (PR 3).** That slice documented
> `solve_gauquelin_sector`'s `1327:21 <` → `==` as an equivalent mutant on the
> grounds that "the campaign does not pin error-message text". That premise is
> incorrect: `systems/tests.rs` already pinned error-message text in five
> places before this PR. PR 5 kills the structurally identical mutant in
> `solve_placidian_cusp` (1741 `<` → `==`) by asserting the message, so
> `solve_gauquelin_sector`'s GQ-1 is **killable the same way** and its
> equivalent classification should be withdrawn. Not done here — it is PR 3's
> territory and would change a merged slice's tally. **Follow-up:** add the
> message assertion to `solve_gauquelin_sector_fails_closed_on_nonconvergence`
> and drop GQ-1, taking the Sector residual from 6 to 5.

- [ ] **Step 4: Full CI**

Run: `mise run ci`
Expected: green — `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and the workspace test run all pass.

- [ ] **Step 5: Commit and open the PR**

```bash
mise exec -- cargo fmt --all
git add docs/follow-ups.md
git commit -m "docs(follow-ups): record FU-9 houses quadrant/projection triage (23 -> 3)"
git push -u origin fu9-houses-quadrant-mutant-triage
gh pr create --fill
```

---

## Acceptance criteria

- Scoped re-run reports `0 missed` beyond the documented equivalents; whole-file re-run drops 83 → 63.
- Every expected value traces to the SE corpus or the independent Python reference; none is the output of the function under test.
- Tests-only: `git diff --stat main` touches no `.rs` file outside `crates/pleiades-houses/src/systems/tests/`.
- No `#[mutants::skip]` anywhere; no `validate-*` file touched; `mise.toml` unchanged.
- `mise run ci` green.
- `docs/follow-ups.md` carries the Progress entry, the updated tally, and the Sector-slice correction.
