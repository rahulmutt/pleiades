# SP-4 Planetary Nodes and Apsides Implementation Plan

> **Post-execution reconciliation (2026-07-07):** SP-4 shipped a **184-row** corpus with `EXPECTED_ROWS = 184`, not the 325/309 stated below in Tasks 7–8. During execution the offline backend chain proved unable to serve small-body (SE 15–20) osculating states, and vendored Swiss Ephemeris `swe_nod_aps` was found to reject fictitious bodies (SE 40–58) upstream (enabling branch commented out), so both categories were dropped; the 20 barycentric rows were kept but generated with `SEFLG_SWIEPH` (DE431) because Moshier cannot produce barycentric positions. Final corpus = 72 mean + 6 mean-fopoint + 80 osculating + 20 barycentric + 6 oscu-fopoint = **184**. The durable rationale lives in the corpus manifest/CSV header, `nod_aps_thresholds.rs`, `docs/follow-ups.md` FU-6/FU-7, and the compatibility-profile summary. The row counts in Tasks 7–8 below are the original (superseded) plan.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Swiss-Ephemeris `swe_nod_aps()` analogue — `EventEngine::nod_aps` returning ascending/descending node, perihelion, and aphelion (or second focus) with positions + speeds for the full SE body set — gated by a fail-closed `validate-nod-aps` SE-parity gate.

**Architecture:** `pleiades-apsides` generalizes to full two-body element/point geometry (`elements_from_state`, `points_from_elements`). `pleiades-events` gains a `mean_elements` table module (SE's VSOP87 mean elements, mean equinox of date) and a `nod_aps` module that assembles per-body states from the caller's backend chain by finite-difference sampling (SE's own technique), rotates them to the true ecliptic of date, forms the four points, re-centers them geocentrically, applies annual aberration, and derives point speeds by evaluating the whole pipeline at ±0.5 day. A committed SE corpus (`tools/se-nodaps-reference`) and a fail-closed `validate-nod-aps` gate enforce parity.

**Tech Stack:** Rust workspace crates only (no new runtime deps); `libswisseph-sys 0.1.2` + `swisseph 0.1.1` for the out-of-workspace reference tool; fnv1a64 checksum-guarded committed corpus.

## Global Constraints

- **No new runtime dependency:** `pleiades-events` gains only the workspace dep `pleiades-apsides`; `pleiades-apsides` stays std-only. Spec §Architecture.
- **API surface:** an engine function on `EventEngine<B>`, NOT new `CelestialBody` variants and NOT chart-layer routing. Spec §Non-goals.
- **Output:** geocentric **true-ecliptic-of-date** longitude/latitude/distance + central-difference speeds for each of the four points. Corpus flags pin the semantics (see Reconciliation §R3). Spec §Public API.
- **Fail closed:** unsupported body×method → typed error, never silent fallback; corpus drift → checksum failure; degenerate geometry → typed error, no NaNs. Spec §Error handling.
- **Window:** 1900–2100 TDB (`WINDOW_START_JD`/`WINDOW_END_JD`), shrunk by a 1.0-day sampling margin at each end. Spec §Error handling.
- **Compatibility profile bump 0.7.9 → 0.7.10; API-stability profile unchanged at 0.2.2** (`pleiades-events` is unpublished; `pleiades-apsides` changes are additive). Spec §Phasing.
- **fnv1a64** is the repo checksum scheme (`pleiades_apparent::fnv1a64` in-workspace; byte-identical local copy in the tool).
- **Ceilings are measured, not promised:** provisional generous ceilings first, then pinned at ~1.4× measured maxima per category (house-gate style). Spec §Validation.

## Design notes / errata vs. spec

These are corrections and refinements from pre-plan research against the vendored SE C source (`libswisseph-sys-0.1.2/libswisseph/swecl.c`, `swe_nod_aps` at swecl.c:5064-5643). They are **binding**.

**R1 — Mean coverage includes the Sun; Earth is unsupported.** SE's mean branch covers Sun..Neptune + Earth (swecl.c:5154-5155). For `ipl=SE_SUN` it uses **Earth's elements** (`ipl_to_elem[0] = 2`) and returns the geocentric "Black Sun" points by **negating** the heliocentric Earth-orbit point (swecl.c:5482-5484). `CelestialBody` has **no Earth variant**, so Earth is not addressable — `CelestialBody::Sun` carries SE's Sun semantics and that is the whole story. The spec's "Mean = Moon + Mercury–Neptune only" becomes "Moon + Sun + Mercury–Neptune".

**R2 — Moon mean path reuses the backend's `MeanNode`/`MeanPerigee` longitudes** (served release-grade by `ElpBackend`), plus SE's fixed scalars `MOON_MEAN_INCL = 5.1453964°`, `MOON_MEAN_ECC = 0.054900489`, `MOON_MEAN_DIST = 384_400_000 m` (sweph.h:260-262) — instead of porting `swi_mean_lunar_elements`. The ELP-vs-SE mean-longitude difference is a measured cross-theory residual; if the measured Moon-mean ceiling comes out unacceptably wide (> ~120″), open a follow-up to port SE's `corr_mean_node`/`corr_mean_apog` corrections — do not block this branch on it.

**R3 — Corpus semantics: SE default output minus gravitational deflection.** SE applies the full apparent chain to the returned points (precession + nutation + light deflection + annual aberration; swecl.c:5434-5629). Pleiades omits gravitational deflection project-wide, so the corpus is generated with `iflag = SEFLG_MOSEPH | SEFLG_SPEED | SEFLG_NOGDEFL` and our pipeline applies: recenter → annual aberration → true-of-date output. The Moon skips aberration (SE disables it for the geocentric Moon, swecl.c:5118-5122).

**R4 — Nodes are referred to the ecliptic-of-date plane; "true vs mean" is only a longitude origin shift.** Nutation does not tilt the ecliptic plane — the mean- and true-of-date ecliptics are the same plane with equinox origins Δψ apart. So the J2000→true-of-date state rotation is: spherical precession (`precess_ecliptic_j2000_to_date`) + `λ += Δψ` (the same construction `ephemeris.rs::heliocentric_longitude_deg` uses), applied per sample at the sample's own time.

**R5 — Speeds come from re-running the position pipeline at t±0.5 day**, not from SE's three-sample scheme, mirroring `pleiades-data`'s `osculating_apsis_motion`. Speed residuals vs SE's speed columns get their own measured ceilings.

**R6 — API takes `Instant`, not `jd_tdb: f64`** (spec showed `jd_tdb`), matching every other `EventEngine` method.

**R7 — Small-body corpus coverage is the six SE-numbered small bodies** (Chiron 15, Pholus 16, Ceres 17, Pallas 18, Juno 19, Vesta 20), because SE resolves them from the `seas_18.se1` ephemeris file, which is **pinned by SHA-256 but not committed** (de440-kernel pattern). The other 30 Tier-A asteroids run through the identical code path (`Custom("asteroid", …)` via the JPL/SPK backend) but have no SE reference rows; this is a documented, honest coverage bound (the gate `log`s it in its summary line).

**R8 — Sun/Earth node semantics are pinned by the corpus.** Earth's mean elements have zero node/inclination; SE zeroes the node slots for Earth-element bodies (swecl.c:5436-5439). Task 4 implements the mechanical closed-form output first; Task 9 compares against the generated Sun rows and, **iff** SE emits zeros for the Sun's node columns, adds the same zeroing (a `body == Sun` early-return of zeroed node points).

**R9 — `OsculatingBarycentric` falls back to heliocentric for bodies with heliocentric distance ≤ 6 AU** (swecl.c:5245), exactly like SE — it is not an error. The Sun-to-SSB offset is computed from the giant-planet heliocentric vectors weighted by `1/plmass` (Jupiter–Pluto), since no backend serves barycentric states.

**R10 — SE method bit values** (swephexp.h:291-294): MEAN=1, OSCU=2, OSCU_BAR=4, FOPOINT=256. The corpus `method` column stores 1/2/4; `fopoint` is a separate 0/1 column (tool ORs 256 in when 1).

## File structure

**Modified — `crates/pleiades-apsides/src/lib.rs`** (~220 → ~450 lines, stays one file): add `KeplerianElements`, `OrbitalPoints`, `elements_from_state`, `points_from_elements`, `ApsidesError::DegenerateNode`. Existing `apsides()` (lilith path) untouched.

**`crates/pleiades-events/`:**
- Create `src/mean_elements.rs` — SE VSOP87 mean-element tables + `plmass` + evaluation + μ helpers.
- Create `src/nod_aps.rs` — public types, engine methods, state assembly, frame rotation, aberration, recentering.
- Modify `src/lib.rs` — `mod mean_elements; mod nod_aps;` + `pub use nod_aps::{ApsisConvention, NodApsMethod, NodApsPoint, NodesApsides};`
- Modify `src/error.rs` — `UnsupportedNodAps { detail }`, `DegenerateNodAps { detail }` variants.
- Modify `src/ephemeris.rs` — promote `spherical_to_cartesian` to `pub(crate)`; add `read_mean_longitude` (distance-optional read).
- Modify `Cargo.toml` — dep `pleiades-apsides`; dev-deps `pleiades-vsop87`, `pleiades-elp`, `pleiades-jpl`, `pleiades-fict`.
- Create `tests/nod_aps.rs` — integration tests over the full routing chain.

**`crates/pleiades-validate/`:**
- Create `src/nod_aps_thresholds.rs`, `src/nod_aps_validation.rs`.
- Create `data/nod-aps-corpus/nod-aps.csv` + `manifest.txt` (committed, generated by the tool).
- Modify `src/lib.rs` (module decls + re-export), `src/render/cli.rs` (gate runner + match arm + help), `src/tests/validate_gates.rs` (alias/help tests), `Cargo.toml` (add `pleiades-vsop87`/`pleiades-elp`/`pleiades-jpl`/`pleiades-fict` if absent — `pleiades-events` and `pleiades-data` are already deps via the crossings gate).

**Create `tools/se-nodaps-reference/`** — `Cargo.toml`, `src/main.rs`, `data/seorbel.txt` (copied from `tools/se-fictitious-reference/data/seorbel.txt`).

**Docs closeout:** `crates/pleiades-core/src/compatibility/mod.rs` (0.7.10 + checksum + summary), version-string tests, `README.md`, `PLAN.md`, `plan/status/02-next-slice-candidates.md`.

---

### Task 1: `pleiades-apsides` — general element/point geometry

**Files:**
- Modify: `crates/pleiades-apsides/src/lib.rs`

**Interfaces:**
- Consumes: existing `ApsisPoint`, `ApsidesError`, `dot`, `norm`, `to_ecliptic`, `MIN_ECCENTRICITY`.
- Produces (used by Tasks 4–5):
  - `pub struct KeplerianElements { pub node_deg: f64, pub peri_lon_deg: f64, pub incl_deg: f64, pub eccentricity: f64, pub semi_major_au: f64 }`
  - `pub struct OrbitalPoints { pub ascending: ApsisPoint, pub descending: ApsisPoint, pub perihelion: ApsisPoint, pub aphelion: ApsisPoint, pub eccentricity: f64, pub semi_major_au: f64 }`
  - `pub fn elements_from_state(pos_au: [f64; 3], vel_au_per_day: [f64; 3], mu: f64) -> Result<KeplerianElements, ApsidesError>`
  - `pub fn points_from_elements(elements: &KeplerianElements, second_focus: bool) -> Result<OrbitalPoints, ApsidesError>`
  - `ApsidesError::DegenerateNode` (new variant).

- [ ] **Step 1: Write the failing tests** (append to the existing `#[cfg(test)] mod tests`)

```rust
    // An inclined orbit with perihelion placed at argument-of-perihelion ω from
    // the +x-aligned ascending node: Ω = 40°, i = 10°, ω = 30°, e = 0.2, a = 2 AU.
    fn inclined_elements() -> KeplerianElements {
        KeplerianElements {
            node_deg: 40.0,
            peri_lon_deg: 70.0, // ϖ = Ω + ω
            incl_deg: 10.0,
            eccentricity: 0.2,
            semi_major_au: 2.0,
        }
    }

    #[test]
    fn points_from_elements_places_nodes_and_apsides() {
        let el = inclined_elements();
        let pts = points_from_elements(&el, false).unwrap();
        let p = el.semi_major_au * (1.0 - el.eccentricity * el.eccentricity);
        let omega = (el.peri_lon_deg - el.node_deg).to_radians();
        // Nodes lie in the reference plane at Ω / Ω+180 with the ellipse radius
        // at true anomaly ∓ω (argument of latitude 0 / π).
        assert!((pts.ascending.longitude_deg - 40.0).abs() < 1e-9);
        assert!(pts.ascending.latitude_deg.abs() < 1e-12);
        let r_asc = p / (1.0 + el.eccentricity * omega.cos());
        assert!((pts.ascending.distance_au - r_asc).abs() < 1e-12);
        assert!((pts.descending.longitude_deg - 220.0).abs() < 1e-9);
        let r_dsc = p / (1.0 - el.eccentricity * omega.cos());
        assert!((pts.descending.distance_au - r_dsc).abs() < 1e-12);
        // Perihelion at r = a(1−e), latitude sin β = sin i · sin ω.
        assert!((pts.perihelion.distance_au - 2.0 * 0.8).abs() < 1e-12);
        let beta = (el.incl_deg.to_radians().sin() * omega.sin()).asin().to_degrees();
        assert!((pts.perihelion.latitude_deg - beta).abs() < 1e-9);
        // Aphelion opposite at r = a(1+e).
        assert!((pts.aphelion.distance_au - 2.0 * 1.2).abs() < 1e-12);
        assert!((pts.aphelion.latitude_deg + beta).abs() < 1e-9);
    }

    #[test]
    fn second_focus_replaces_aphelion_distance_only() {
        let el = inclined_elements();
        let apo = points_from_elements(&el, false).unwrap().aphelion;
        let foc = points_from_elements(&el, true).unwrap().aphelion;
        assert!((foc.distance_au - 2.0 * el.semi_major_au * el.eccentricity).abs() < 1e-12);
        assert!((foc.longitude_deg - apo.longitude_deg).abs() < 1e-9);
        assert!((foc.latitude_deg - apo.latitude_deg).abs() < 1e-9);
    }

    #[test]
    fn elements_from_state_round_trips_through_points() {
        // Build the state at perihelion of the inclined orbit analytically,
        // then recover the elements and check the perihelion point matches.
        let el = inclined_elements();
        let mu = 2.959e-4;
        let a = el.semi_major_au;
        let e = el.eccentricity;
        let r_peri = a * (1.0 - e);
        let v_peri = (mu / a * (1.0 + e) / (1.0 - e)).sqrt();
        let node = el.node_deg.to_radians();
        let incl = el.incl_deg.to_radians();
        let omega = (el.peri_lon_deg - el.node_deg).to_radians();
        // In-plane basis: P̂ toward perihelion, Q̂ 90° ahead in the motion.
        let rot = |u: f64, s: f64| -> [f64; 3] {
            [
                s * (u.cos() * node.cos() - u.sin() * incl.cos() * node.sin()),
                s * (u.cos() * node.sin() + u.sin() * incl.cos() * node.cos()),
                s * (u.sin() * incl.sin()),
            ]
        };
        let pos = rot(omega, r_peri);
        let vel = rot(omega + core::f64::consts::FRAC_PI_2, v_peri);
        let got = elements_from_state(pos, vel, mu).unwrap();
        assert!((got.node_deg - el.node_deg).abs() < 1e-6, "node {}", got.node_deg);
        assert!((got.incl_deg - el.incl_deg).abs() < 1e-6);
        assert!((got.eccentricity - e).abs() < 1e-9);
        assert!((got.semi_major_au - a).abs() < 1e-9);
        assert!((got.peri_lon_deg - el.peri_lon_deg).abs() < 1e-6);
    }

    #[test]
    fn zero_inclination_state_is_degenerate_node() {
        let mu = 2.959e-4;
        let (pos, vel) = perigee_on_x_state(2.0, 0.2, mu);
        let err = elements_from_state(pos, vel, mu).unwrap_err();
        assert_eq!(err, ApsidesError::DegenerateNode);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p pleiades-apsides`
Expected: FAIL — `KeplerianElements`, `points_from_elements`, `elements_from_state`, `DegenerateNode` not found.

- [ ] **Step 3: Implement** (append to `lib.rs`; add the `DegenerateNode` variant to the existing `ApsidesError`)

Add to `ApsidesError` (with doc comment, before `NonFinite`):

```rust
    /// Inclination below the conditioning floor (node direction ill-defined).
    DegenerateNode,
```

Append:

```rust
/// Osculating Keplerian elements of an elliptical orbit, referred to the input
/// state's frame: longitude of ascending node Ω, longitude of perihelion
/// ϖ = Ω + ω, inclination i (all degrees), eccentricity, semi-major axis (AU).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeplerianElements {
    /// Longitude of the ascending node Ω, degrees in `[0, 360)`.
    pub node_deg: f64,
    /// Longitude of perihelion ϖ = Ω + ω (ω in-plane), degrees in `[0, 360)`.
    pub peri_lon_deg: f64,
    /// Inclination to the reference plane, degrees in `[0, 180)`.
    pub incl_deg: f64,
    /// Eccentricity (`0 < e < 1` for a bound orbit).
    pub eccentricity: f64,
    /// Semi-major axis, AU.
    pub semi_major_au: f64,
}

/// The four singular orbital points of an ellipse: both nodes and both apsides.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrbitalPoints {
    /// Point on the ellipse at the ascending node.
    pub ascending: ApsisPoint,
    /// Point on the ellipse at the descending node.
    pub descending: ApsisPoint,
    /// Near apsis (perihelion/perigee).
    pub perihelion: ApsisPoint,
    /// Far apsis (aphelion/apogee) — or the ellipse's second (empty) focus at
    /// distance `2ae` in the same direction when requested.
    pub aphelion: ApsisPoint,
    /// Eccentricity of the ellipse.
    pub eccentricity: f64,
    /// Semi-major axis, AU.
    pub semi_major_au: f64,
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Osculating elements from a state vector. Frame-agnostic: Ω/ϖ/i are referred
/// to the input frame's reference plane and +x origin of longitude.
pub fn elements_from_state(
    pos_au: [f64; 3],
    vel_au_per_day: [f64; 3],
    mu: f64,
) -> Result<KeplerianElements, ApsidesError> {
    // Reuse the eccentricity/energy machinery (and its error taxonomy).
    let aps = apsides(pos_au, vel_au_per_day, mu)?;
    let h = cross(pos_au, vel_au_per_day);
    let h_mag = norm(h);
    if !h_mag.is_finite() || h_mag == 0.0 {
        return Err(ApsidesError::NonFinite);
    }
    // Node vector n = ẑ × h points to the ascending node.
    let n = [-h[1], h[0], 0.0];
    let n_mag = norm(n);
    if n_mag < 1e-12 * h_mag {
        return Err(ApsidesError::DegenerateNode);
    }
    let node_deg = n[1].atan2(n[0]).to_degrees().rem_euclid(360.0);
    let incl_deg = (h[2] / h_mag).acos().to_degrees();
    // ϖ = Ω + ω where ω is the in-plane angle from the node to perihelion; the
    // perihelion direction is the apsides() perigee point's unit vector.
    let peri = aps.perigee;
    let peri_vec = {
        let lon = peri.longitude_deg.to_radians();
        let lat = peri.latitude_deg.to_radians();
        [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()]
    };
    let n_hat = [n[0] / n_mag, n[1] / n_mag, 0.0];
    let cos_omega = dot(n_hat, peri_vec).clamp(-1.0, 1.0);
    let mut omega = cos_omega.acos();
    if peri_vec[2] < 0.0 {
        omega = 2.0 * core::f64::consts::PI - omega;
    }
    Ok(KeplerianElements {
        node_deg,
        peri_lon_deg: (node_deg + omega.to_degrees()).rem_euclid(360.0),
        incl_deg,
        eccentricity: aps.eccentricity,
        semi_major_au: aps.semi_major_au,
    })
}

/// The four orbital points from Keplerian elements, in the elements' frame.
/// With `second_focus`, the aphelion slot instead carries the empty focus at
/// distance `2ae` (Swiss Ephemeris `SE_NODBIT_FOPOINT`); direction unchanged.
pub fn points_from_elements(
    elements: &KeplerianElements,
    second_focus: bool,
) -> Result<OrbitalPoints, ApsidesError> {
    let e = elements.eccentricity;
    let a = elements.semi_major_au;
    if !(e.is_finite() && a.is_finite() && elements.node_deg.is_finite()
        && elements.peri_lon_deg.is_finite() && elements.incl_deg.is_finite())
    {
        return Err(ApsidesError::NonFinite);
    }
    if e < MIN_ECCENTRICITY {
        return Err(ApsidesError::DegenerateOrbit);
    }
    if e >= 1.0 || a <= 0.0 {
        return Err(ApsidesError::UnboundOrbit);
    }
    let node = elements.node_deg.to_radians();
    let incl = elements.incl_deg.to_radians();
    let omega = (elements.peri_lon_deg - elements.node_deg).to_radians();
    let p = a * (1.0 - e * e);
    // In-plane point at argument-of-latitude u (angle from the ascending node),
    // rotated into the reference frame.
    let in_plane = |u: f64, r: f64| -> [f64; 3] {
        [
            r * (u.cos() * node.cos() - u.sin() * incl.cos() * node.sin()),
            r * (u.cos() * node.sin() + u.sin() * incl.cos() * node.cos()),
            r * (u.sin() * incl.sin()),
        ]
    };
    // At the ascending node u = 0 and ν = −ω (so cos ν = cos ω); descending
    // node u = π, ν = π − ω.
    let r_asc = p / (1.0 + e * omega.cos());
    let r_dsc = p / (1.0 - e * omega.cos());
    let apo_dist = if second_focus { 2.0 * a * e } else { a * (1.0 + e) };
    Ok(OrbitalPoints {
        ascending: to_ecliptic(in_plane(0.0, r_asc))?,
        descending: to_ecliptic(in_plane(core::f64::consts::PI, r_dsc))?,
        perihelion: to_ecliptic(in_plane(omega, a * (1.0 - e)))?,
        aphelion: to_ecliptic(in_plane(omega + core::f64::consts::PI, apo_dist))?,
        eccentricity: e,
        semi_major_au: a,
    })
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-apsides`
Expected: PASS (all new + 3 pre-existing tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-apsides/src/lib.rs
git commit -m "feat(apsides): SP-4 general element/point geometry (elements_from_state, points_from_elements)"
```

---

### Task 2: `pleiades-events` — public types, errors, scaffold

**Files:**
- Create: `crates/pleiades-events/src/nod_aps.rs` (types only in this task)
- Modify: `crates/pleiades-events/src/lib.rs`, `src/error.rs`, `src/ephemeris.rs`, `Cargo.toml`

**Interfaces:**
- Produces (used by Tasks 3–6): `NodApsMethod { Mean, Osculating, OsculatingBarycentric }`, `ApsisConvention { Aphelion, SecondFocus }`, `NodApsPoint`, `NodesApsides`, `EventError::{UnsupportedNodAps, DegenerateNodAps}`, `pub(crate) fn spherical_to_cartesian`, `pub(crate) fn read_mean_longitude`.

- [ ] **Step 1: Write failing tests** (in `error.rs` tests + new `nod_aps.rs` tests)

Append to `error.rs` `#[cfg(test)] mod tests`:

```rust
    #[test]
    fn nod_aps_errors_render_their_detail() {
        let err = EventError::UnsupportedNodAps { detail: "mean elements for Pluto".into() };
        assert!(err.to_string().contains("mean elements for Pluto"));
        let err = EventError::DegenerateNodAps { detail: "node ill-defined".into() };
        assert!(err.to_string().contains("node ill-defined"));
    }
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p pleiades-events nod_aps_errors`
Expected: FAIL — variants not found.

- [ ] **Step 3: Implement**

`error.rs` — add variants to `EventError` (before the closing brace) and `Display` arms:

```rust
    /// A body/method combination `nod_aps` does not support (fail-closed).
    UnsupportedNodAps {
        /// Human-readable explanation.
        detail: String,
    },
    /// The orbit geometry was too degenerate to form nodes/apsides.
    DegenerateNodAps {
        /// Human-readable explanation.
        detail: String,
    },
```

```rust
            EventError::UnsupportedNodAps { detail } => {
                write!(f, "unsupported nod_aps request: {detail}")
            }
            EventError::DegenerateNodAps { detail } => {
                write!(f, "degenerate nod_aps geometry: {detail}")
            }
```

`ephemeris.rs` — change `fn spherical_to_cartesian` to `pub(crate) fn spherical_to_cartesian` (same body), and add below `read_mean_ecliptic`:

```rust
/// Mean/J2000 geocentric ecliptic longitude only — for bodies whose backends
/// legitimately omit distance (the mean lunar points: `MeanNode`,
/// `MeanPerigee`). Latitude is read and discarded; distance is not required.
pub(crate) fn read_mean_longitude<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<f64, EventError> {
    let result = backend
        .position(&request(body, julian_day))
        .map_err(|e| EventError::Backend(e.to_string()))?;
    let ecliptic = result.ecliptic.ok_or(EventError::MissingCoordinates {
        body_label,
        julian_day,
    })?;
    Ok(ecliptic.longitude.degrees())
}
```

`Cargo.toml` — under `[dependencies]` add `pleiades-apsides = { workspace = true }`; under `[dev-dependencies]` add:

```toml
pleiades-vsop87 = { workspace = true }
pleiades-elp = { workspace = true }
pleiades-jpl = { workspace = true }
pleiades-fict = { workspace = true }
```

(Check root `Cargo.toml [workspace.dependencies]` lists all five; add any missing entry in the sibling style, e.g. `pleiades-apsides = { path = "crates/pleiades-apsides", version = "0.2.0" }` — copy the exact version pattern used by the existing `pleiades-apsides` entry, which `pleiades-data` already consumes.)

Create `src/nod_aps.rs` with the public types (engine methods come in Task 4):

```rust
//! Planetary/lunar orbital nodes and apsides — Swiss Ephemeris `swe_nod_aps`
//! analogue. See `EventEngine::nod_aps`.

use crate::error::EventError;

/// How the orbit is modeled — Swiss Ephemeris `SE_NODBIT_*` analogues.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodApsMethod {
    /// Mean orbital elements (`SE_NODBIT_MEAN`). Moon + Sun + Mercury–Neptune.
    Mean,
    /// Osculating ellipse from the instantaneous state (`SE_NODBIT_OSCU`).
    Osculating,
    /// Osculating ellipse about the solar-system barycenter for bodies beyond
    /// ~6 AU heliocentric distance (`SE_NODBIT_OSCU_BAR`); inside 6 AU this
    /// falls back to the heliocentric ellipse, matching Swiss Ephemeris.
    OsculatingBarycentric,
}

/// What the fourth point means — aphelion or the ellipse's second focus
/// (`SE_NODBIT_FOPOINT`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApsisConvention {
    /// Far apsis at distance `a(1+e)`.
    Aphelion,
    /// Second (empty) focus at distance `2ae`, same direction as the aphelion.
    SecondFocus,
}

/// One orbital point: geocentric true-ecliptic-of-date position and
/// central-difference speeds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodApsPoint {
    /// Ecliptic longitude, degrees in `[0, 360)`, true equinox of date.
    pub longitude_deg: f64,
    /// Ecliptic latitude, degrees in `[-90, 90]`.
    pub latitude_deg: f64,
    /// Geocentric distance, AU.
    pub distance_au: f64,
    /// dλ/dt, degrees/day (central difference over ±0.5 day).
    pub longitude_speed_deg_per_day: f64,
    /// dβ/dt, degrees/day.
    pub latitude_speed_deg_per_day: f64,
    /// d(distance)/dt, AU/day.
    pub distance_speed_au_per_day: f64,
}

/// The four orbital points returned by [`EventEngine::nod_aps`]
/// (ascending node, descending node, perihelion, aphelion-or-focus).
///
/// [`EventEngine::nod_aps`]: crate::EventEngine::nod_aps
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodesApsides {
    /// Ascending-node point.
    pub ascending: NodApsPoint,
    /// Descending-node point.
    pub descending: NodApsPoint,
    /// Near apsis (perihelion; perigee for the Moon).
    pub perihelion: NodApsPoint,
    /// Far apsis — or second focus under [`ApsisConvention::SecondFocus`].
    pub aphelion: NodApsPoint,
    /// The method that actually served the request.
    pub method: NodApsMethod,
}
```

`lib.rs` — add `mod mean_elements;` and `mod nod_aps;` to the module list (create an empty `src/mean_elements.rs` containing only `//! SE mean orbital elements (filled by the mean-elements task).` so the crate compiles) and:

```rust
pub use nod_aps::{ApsisConvention, NodApsMethod, NodApsPoint, NodesApsides};
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p pleiades-events`
Expected: PASS (new error test + all pre-existing).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events Cargo.toml
git commit -m "feat(events): SP-4 nod_aps public types + error variants + scaffold"
```

---

### Task 3: `mean_elements` — SE VSOP87 mean-element tables

**Files:**
- Modify: `crates/pleiades-events/src/mean_elements.rs`

**Interfaces:**
- Produces (used by Tasks 4–5):
  - `pub(crate) fn mean_elements_of_date(index: usize, jd_tdb: f64) -> pleiades_apsides::KeplerianElements`
  - `pub(crate) fn elem_index(body: &CelestialBody) -> Option<usize>` — Sun→2 (Earth's elements), Mercury→0, Venus→1, Mars→3, Jupiter→4, Saturn→5, Uranus→6, Neptune→7; everything else `None`.
  - `pub(crate) fn mu_au3_day2(body: &CelestialBody) -> f64` — solar μ with the SE mass ratio for Sun/Mercury–Pluto, geocentric Earth+Moon μ for the Moon, bare solar μ otherwise.
  - `pub(crate) const EARTH_MOON_MASS_RATIO: f64`, `pub(crate) const SUN_MASS_RATIO: [f64; 9]`, `pub(crate) const MOON_MEAN_INCL_DEG/MOON_MEAN_ECC/MOON_MEAN_SEMA_AU: f64`.

- [ ] **Step 1: Write failing tests** (inline module)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j2000_elements_are_the_constant_terms() {
        // t = 0 at J2000 ⇒ every polynomial collapses to its constant column.
        let m = mean_elements_of_date(0, 2_451_545.0);
        assert!((m.node_deg - 48.330893).abs() < 1e-12);
        assert!((m.peri_lon_deg - 77.456119).abs() < 1e-12);
        assert!((m.incl_deg - 7.004986).abs() < 1e-12);
        assert!((m.eccentricity - 0.20563175).abs() < 1e-12);
        assert!((m.semi_major_au - 0.387098310).abs() < 1e-12);
    }

    #[test]
    fn one_century_later_mercury_node_advances() {
        // t = 1: 48.330893 + 1.1861890 + 0.00017587 + 0.000000211
        let m = mean_elements_of_date(0, 2_451_545.0 + 36525.0);
        assert!((m.node_deg - 49.517258081).abs() < 1e-8, "node {}", m.node_deg);
    }

    #[test]
    fn mu_uses_se_mass_ratios() {
        use pleiades_types::CelestialBody;
        let gms = SUN_GM_AU3_DAY2;
        let mercury = mu_au3_day2(&CelestialBody::Mercury);
        assert!((mercury / gms - (1.0 + 1.0 / 6_023_600.0)).abs() < 1e-12);
        // Asteroids/fictitious: massless.
        let chiron = mu_au3_day2(&CelestialBody::Custom(
            pleiades_types::CustomBodyId::new("asteroid", "2060-Chiron"),
        ));
        assert!((chiron - gms).abs() < 1e-30);
        // Moon: geocentric Earth+Moon μ, far smaller than solar.
        let moon = mu_au3_day2(&CelestialBody::Moon);
        assert!(moon < 1e-8 && moon > 1e-10, "moon mu {moon}");
    }
}
```

- [ ] **Step 2: Run to verify failure** — `cargo test -p pleiades-events mean_elements`; Expected: FAIL (items not found).

- [ ] **Step 3: Implement** (replace file contents)

```rust
//! Swiss Ephemeris mean orbital elements for Mercury–Neptune (+ Earth, used by
//! the Sun per SE semantics), transcribed verbatim from `swecl.c:5000-5063`
//! ("mean elements for Mercury - Neptune from VSOP87 (mean equinox of date)").
//! Rows: 0 Mercury, 1 Venus, 2 Earth, 3 Mars, 4 Jupiter, 5 Saturn, 6 Uranus,
//! 7 Neptune. Columns: polynomial coefficients in `t = (jd_tdb − J2000)/36525`.
//! Angles in degrees, semi-major axis in AU, referred to the MEAN equinox and
//! ecliptic of date.

use pleiades_apsides::KeplerianElements;
use pleiades_types::CelestialBody;

pub(crate) const J2000_JD: f64 = 2_451_545.0;

#[rustfmt::skip]
const EL_NODE: [[f64; 4]; 8] = [
    [ 48.330893,  1.1861890,  0.00017587,  0.000000211],  // Mercury
    [ 76.679920,  0.9011190,  0.00040665, -0.000000080],  // Venus
    [  0.0,       0.0,        0.0,         0.0        ],  // Earth
    [ 49.558093,  0.7720923,  0.00001605,  0.000002325],  // Mars
    [100.464441,  1.0209550,  0.00040117,  0.000000569],  // Jupiter
    [113.665524,  0.8770970, -0.00012067, -0.000002380],  // Saturn
    [ 74.005947,  0.5211258,  0.00133982,  0.000018516],  // Uranus
    [131.784057,  1.1022057,  0.00026006, -0.000000636],  // Neptune
];
#[rustfmt::skip]
const EL_PERI: [[f64; 4]; 8] = [
    [ 77.456119,  1.5564775,  0.00029589,  0.000000056],
    [131.563707,  1.4022188, -0.00107337, -0.000005315],
    [102.937348,  1.7195269,  0.00045962,  0.000000499],
    [336.060234,  1.8410331,  0.00013515,  0.000000318],
    [ 14.331309,  1.6126668,  0.00103127, -0.000004569],
    [ 93.056787,  1.9637694,  0.00083757,  0.000004899],
    [173.005159,  1.4863784,  0.00021450,  0.000000433],
    [ 48.123691,  1.4262677,  0.00037918, -0.000000003],
];
#[rustfmt::skip]
const EL_INCL: [[f64; 4]; 8] = [
    [  7.004986,  0.0018215, -0.00001809,  0.000000053],
    [  3.394662,  0.0010037, -0.00000088, -0.000000007],
    [  0.0,       0.0,        0.0,         0.0        ],
    [  1.849726, -0.0006010,  0.00001276, -0.000000006],
    [  1.303270, -0.0054966,  0.00000465, -0.000000004],
    [  2.488878, -0.0037363, -0.00001516,  0.000000089],
    [  0.773196,  0.0007744,  0.00003749, -0.000000092],
    [  1.769952, -0.0093082, -0.00000708,  0.000000028],
];
#[rustfmt::skip]
const EL_ECCE: [[f64; 4]; 8] = [
    [  0.20563175,  0.000020406, -0.0000000284, -0.00000000017],
    [  0.00677188, -0.000047766,  0.0000000975,  0.00000000044],
    [  0.01670862, -0.000042037, -0.0000001236,  0.00000000004],
    [  0.09340062,  0.000090483, -0.0000000806, -0.00000000035],
    [  0.04849485,  0.000163244, -0.0000004719, -0.00000000197],
    [  0.05550862, -0.000346818, -0.0000006456,  0.00000000338],
    [  0.04629590, -0.000027337,  0.0000000790,  0.00000000025],
    [  0.00898809,  0.000006408, -0.0000000008, -0.00000000005],
];
#[rustfmt::skip]
const EL_SEMA: [[f64; 4]; 8] = [
    [  0.387098310,  0.0,           0.0,           0.0],
    [  0.723329820,  0.0,           0.0,           0.0],
    [  1.000001018,  0.0,           0.0,           0.0],
    [  1.523679342,  0.0,           0.0,           0.0],
    [  5.202603191,  0.0000001913,  0.0,           0.0],
    [  9.554909596,  0.0000021389,  0.0,           0.0],
    [ 19.218446062, -0.0000000372,  0.00000000098, 0.0],
    [ 30.110386869, -0.0000001663,  0.00000000069, 0.0],
];

/// Sun/planet mass ratios (`swecl.c:5051-5062`): Mercury, Venus, Earth+Moon,
/// Mars, Jupiter, Saturn, Uranus, Neptune, Pluto.
pub(crate) const SUN_MASS_RATIO: [f64; 9] = [
    6_023_600.0, 408_523.719, 328_900.5, 3_098_703.59, 1_047.348_644,
    3_497.901_8, 22_902.98, 19_412.26, 136_566_000.0,
];

/// SE physical constants (sweph.h:260-279).
pub(crate) const EARTH_MOON_MASS_RATIO: f64 = 81.300_569_074_190_62;
const AUNIT_M: f64 = 1.495_978_707_00e11;
const HELGRAVCONST_M3_S2: f64 = 1.327_124_400_179_87e20;
const GEOGCONST_M3_S2: f64 = 3.986_004_48e14;
pub(crate) const MOON_MEAN_INCL_DEG: f64 = 5.145_396_4;
pub(crate) const MOON_MEAN_ECC: f64 = 0.054_900_489;
pub(crate) const MOON_MEAN_SEMA_AU: f64 = 384_400_000.0 / AUNIT_M;

/// Solar GM in AU³/day², from SE's HELGRAVCONST.
pub(crate) const SUN_GM_AU3_DAY2: f64 =
    HELGRAVCONST_M3_S2 / (AUNIT_M * AUNIT_M * AUNIT_M) * (86_400.0 * 86_400.0);
/// Geocentric Earth+Moon GM in AU³/day², from SE's GEOGCONST.
pub(crate) const GEO_GM_AU3_DAY2: f64 = GEOGCONST_M3_S2
    / (AUNIT_M * AUNIT_M * AUNIT_M)
    * (86_400.0 * 86_400.0)
    * (1.0 + 1.0 / EARTH_MOON_MASS_RATIO);

/// Row index into the element tables for a mean-capable body; `None` for
/// bodies without SE mean elements. The Sun maps to Earth's elements
/// (`ipl_to_elem[0] = 2`, swecl.c:5063).
pub(crate) fn elem_index(body: &CelestialBody) -> Option<usize> {
    match body {
        CelestialBody::Sun => Some(2),
        CelestialBody::Mercury => Some(0),
        CelestialBody::Venus => Some(1),
        CelestialBody::Mars => Some(3),
        CelestialBody::Jupiter => Some(4),
        CelestialBody::Saturn => Some(5),
        CelestialBody::Uranus => Some(6),
        CelestialBody::Neptune => Some(7),
        _ => None,
    }
}

/// Mass-table index for μ: like `elem_index` but including Pluto (row 8).
fn mass_index(body: &CelestialBody) -> Option<usize> {
    match body {
        CelestialBody::Pluto => Some(8),
        other => elem_index(other),
    }
}

/// GM for osculating-element formation (SE `Gmsm`, swecl.c:5258/5266):
/// solar μ scaled by `(1 + m_body/m_sun)` where a mass ratio exists, bare
/// solar μ for massless small/fictitious bodies, geocentric μ for the Moon.
pub(crate) fn mu_au3_day2(body: &CelestialBody) -> f64 {
    if *body == CelestialBody::Moon {
        return GEO_GM_AU3_DAY2;
    }
    match mass_index(body) {
        Some(i) => SUN_GM_AU3_DAY2 * (1.0 + 1.0 / SUN_MASS_RATIO[i]),
        None => SUN_GM_AU3_DAY2,
    }
}

fn poly(c: &[f64; 4], t: f64) -> f64 {
    c[0] + t * (c[1] + t * (c[2] + t * c[3]))
}

/// Mean elements at `jd_tdb`, referred to the mean equinox/ecliptic of date.
pub(crate) fn mean_elements_of_date(index: usize, jd_tdb: f64) -> KeplerianElements {
    let t = (jd_tdb - J2000_JD) / 36_525.0;
    KeplerianElements {
        node_deg: poly(&EL_NODE[index], t).rem_euclid(360.0),
        peri_lon_deg: poly(&EL_PERI[index], t).rem_euclid(360.0),
        incl_deg: poly(&EL_INCL[index], t),
        eccentricity: poly(&EL_ECCE[index], t),
        semi_major_au: poly(&EL_SEMA[index], t),
    }
}
```

- [ ] **Step 4: Run** — `cargo test -p pleiades-events mean_elements`; Expected: PASS.
- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/src/mean_elements.rs
git commit -m "feat(events): SP-4 SE VSOP87 mean-element tables + mass ratios + mu helpers"
```

---

### Task 4: Position pipeline + Mean path + `nod_aps` entry point

**Files:**
- Modify: `crates/pleiades-events/src/nod_aps.rs`

**Interfaces:**
- Consumes: Task 1 geometry, Task 3 tables, `crate::crossings::EventEngine`, `crate::ephemeris::{read_mean_ecliptic, read_mean_longitude, spherical_to_cartesian}`, `pleiades_apparent::{nutation::nutation, precess_ecliptic_j2000_to_date}`.
- Produces:
  - `pub fn nod_aps(&self, body: CelestialBody, instant: Instant, method: NodApsMethod, convention: ApsisConvention) -> Result<NodesApsides, EventError>` — Mean fully working after this task; Osculating returns `UnsupportedNodAps` until Task 5.
  - `pub fn nod_aps_default(&self, body: CelestialBody, instant: Instant, convention: ApsisConvention) -> Result<NodesApsides, EventError>`.
  - Internal: `fn points_at(&self, body, jd, method, convention) -> Result<[RawPoint; 4], EventError>`, `struct RawPoint { lon_deg, lat_deg, dist_au }`, plus frame/aberration helpers reused by Task 5.

- [ ] **Step 1: Write the failing tests** (inline in `nod_aps.rs`; these need only the Sun-serving `LinearSunMoon` or pure functions)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::crossings::EventEngine;
    use pleiades_backend::test_backend::LinearSunMoon;
    use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};

    fn tdb(jd: f64) -> Instant {
        Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
    }

    #[test]
    fn lunar_point_bodies_are_rejected() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_545.0));
        let err = engine
            .nod_aps(CelestialBody::MeanNode, tdb(2_451_545.0),
                     NodApsMethod::Osculating, ApsisConvention::Aphelion)
            .unwrap_err();
        assert!(matches!(err, EventError::UnsupportedNodAps { .. }));
    }

    #[test]
    fn mean_method_for_pluto_is_rejected() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_545.0));
        let err = engine
            .nod_aps(CelestialBody::Pluto, tdb(2_451_545.0),
                     NodApsMethod::Mean, ApsisConvention::Aphelion)
            .unwrap_err();
        assert!(matches!(err, EventError::UnsupportedNodAps { .. }));
    }

    #[test]
    fn out_of_window_and_margin_fail_closed() {
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_545.0));
        for jd in [2_400_000.5, crate::error::WINDOW_START_JD + 0.25] {
            let err = engine
                .nod_aps(CelestialBody::Mars, tdb(jd),
                         NodApsMethod::Mean, ApsisConvention::Aphelion)
                .unwrap_err();
            assert!(matches!(err, EventError::OutOfWindow { .. }), "jd {jd}");
        }
    }

    #[test]
    fn wrap_delta_handles_the_zero_crossing() {
        assert!((wrap_delta(359.5, 0.5) - 1.0).abs() < 1e-12);
        assert!((wrap_delta(0.5, 359.5) + 1.0).abs() < 1e-12);
        assert!((wrap_delta(10.0, 20.0) - 10.0).abs() < 1e-12);
    }

    #[test]
    fn aberration_shifts_by_v_over_c_transverse() {
        // Point on +x, observer velocity on +y: shift ≈ |v|/c radians toward +y.
        let v = 0.0172; // ~Earth orbital speed, AU/day
        let p = aberrate([2.0, 0.0, 0.0], [0.0, v, 0.0]);
        let expected = (v / LIGHT_SPEED_AU_PER_DAY).atan();
        let got = p[1].atan2(p[0]);
        assert!((got - expected).abs() < 1e-9);
        // Distance preserved.
        let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        assert!((r - 2.0).abs() < 1e-12);
    }
}
```

- [ ] **Step 2: Run to verify failure** — `cargo test -p pleiades-events nod_aps`; Expected: FAIL.

- [ ] **Step 3: Implement.** Add to `nod_aps.rs` (below the types):

```rust
use crate::crossings::EventEngine;
use crate::ephemeris::{read_mean_ecliptic, read_mean_longitude, spherical_to_cartesian};
use crate::mean_elements::{
    elem_index, mean_elements_of_date, mu_au3_day2, EARTH_MOON_MASS_RATIO,
    MOON_MEAN_ECC, MOON_MEAN_INCL_DEG, MOON_MEAN_SEMA_AU,
};
use pleiades_apparent::nutation::nutation;
use pleiades_apparent::precess_ecliptic_j2000_to_date;
use pleiades_apsides::{points_from_elements, ApsisPoint, KeplerianElements};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{CelestialBody, Instant};

/// Speed of light, AU/day.
const LIGHT_SPEED_AU_PER_DAY: f64 = 173.144_632_674_240_3;
/// Half-span for point-speed central differences (mirrors pleiades-data's
/// osculating-apsis motion pattern).
const SPEED_HALF_SPAN_DAYS: f64 = 0.5;
/// The full pipeline samples at up to `jd ± (0.5 + 2·dt)`; keep 1 day clear of
/// the window edges.
const WINDOW_MARGIN_DAYS: f64 = 1.0;

/// One point's position triple before speed assembly.
#[derive(Clone, Copy, Debug)]
struct RawPoint {
    lon_deg: f64,
    lat_deg: f64,
    dist_au: f64,
}

/// Shortest signed longitude difference `b − a` in degrees.
fn wrap_delta(a: f64, b: f64) -> f64 {
    let mut d = (b - a).rem_euclid(360.0);
    if d > 180.0 {
        d -= 360.0;
    }
    d
}

fn norm3(a: [f64; 3]) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn add3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale3(a: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn cartesian_to_raw(p: [f64; 3]) -> RawPoint {
    let r = norm3(p);
    RawPoint {
        lon_deg: p[1].atan2(p[0]).to_degrees().rem_euclid(360.0),
        lat_deg: (p[2] / r).asin().to_degrees(),
        dist_au: r,
    }
}

/// First-order annual aberration: rotate the unit direction by v⊥/c, keep the
/// distance (SE applies `swi_aberr_light` to each returned point).
fn aberrate(p: [f64; 3], v_obs_au_day: [f64; 3]) -> [f64; 3] {
    let r = norm3(p);
    let u = scale3(p, 1.0 / r);
    let shifted = add3(u, scale3(v_obs_au_day, 1.0 / LIGHT_SPEED_AU_PER_DAY));
    scale3(shifted, r / norm3(shifted))
}

fn apsis_to_raw(p: &ApsisPoint) -> RawPoint {
    RawPoint { lon_deg: p.longitude_deg, lat_deg: p.latitude_deg, dist_au: p.distance_au }
}

impl<B: EphemerisBackend> EventEngine<B> {
    /// Nodes and apsides of a body's orbit — Swiss Ephemeris `swe_nod_aps`
    /// analogue. Returns the ascending node, descending node, perihelion, and
    /// aphelion (or second focus) as geocentric true-ecliptic-of-date
    /// positions with ±0.5-day central-difference speeds.
    ///
    /// Fail-closed: `Mean` outside Moon/Sun/Mercury–Neptune, lunar-point
    /// bodies, and out-of-window instants (including a 1-day sampling margin)
    /// return typed errors.
    pub fn nod_aps(
        &self,
        body: CelestialBody,
        instant: Instant,
        method: NodApsMethod,
        convention: ApsisConvention,
    ) -> Result<NodesApsides, EventError> {
        let jd = instant.julian_day.days();
        self.check_window(jd)?;
        // The speed/state sampling reaches jd ± (0.5 + 2·dt); enforce a hard
        // margin so every backend read stays in-window.
        if !((crate::error::WINDOW_START_JD + WINDOW_MARGIN_DAYS)
            ..=(crate::error::WINDOW_END_JD - WINDOW_MARGIN_DAYS))
            .contains(&jd)
        {
            return Err(EventError::OutOfWindow { julian_day: jd });
        }
        if matches!(
            body,
            CelestialBody::MeanNode | CelestialBody::TrueNode
                | CelestialBody::MeanApogee | CelestialBody::TrueApogee
                | CelestialBody::MeanPerigee | CelestialBody::TruePerigee
        ) {
            return Err(EventError::UnsupportedNodAps {
                detail: format!("{body} is itself a node/apsis body"),
            });
        }
        let p0 = self.points_at(&body, jd, method, convention)?;
        let pm = self.points_at(&body, jd - SPEED_HALF_SPAN_DAYS, method, convention)?;
        let pp = self.points_at(&body, jd + SPEED_HALF_SPAN_DAYS, method, convention)?;
        let assemble = |i: usize| NodApsPoint {
            longitude_deg: p0[i].lon_deg,
            latitude_deg: p0[i].lat_deg,
            distance_au: p0[i].dist_au,
            longitude_speed_deg_per_day: wrap_delta(pm[i].lon_deg, pp[i].lon_deg)
                / (2.0 * SPEED_HALF_SPAN_DAYS),
            latitude_speed_deg_per_day: (pp[i].lat_deg - pm[i].lat_deg)
                / (2.0 * SPEED_HALF_SPAN_DAYS),
            distance_speed_au_per_day: (pp[i].dist_au - pm[i].dist_au)
                / (2.0 * SPEED_HALF_SPAN_DAYS),
        };
        Ok(NodesApsides {
            ascending: assemble(0),
            descending: assemble(1),
            perihelion: assemble(2),
            aphelion: assemble(3),
            method,
        })
    }

    /// Swiss Ephemeris `method = 0` default: mean elements where they exist
    /// (Moon, Sun, Mercury–Neptune), osculating otherwise.
    pub fn nod_aps_default(
        &self,
        body: CelestialBody,
        instant: Instant,
        convention: ApsisConvention,
    ) -> Result<NodesApsides, EventError> {
        let method = if body == CelestialBody::Moon || elem_index(&body).is_some() {
            NodApsMethod::Mean
        } else {
            NodApsMethod::Osculating
        };
        self.nod_aps(body, instant, method, convention)
    }

    /// Positions of the four points at one instant (no speeds).
    fn points_at(
        &self,
        body: &CelestialBody,
        jd: f64,
        method: NodApsMethod,
        convention: ApsisConvention,
    ) -> Result<[RawPoint; 4], EventError> {
        match method {
            NodApsMethod::Mean => self.mean_points_at(body, jd, convention),
            NodApsMethod::Osculating | NodApsMethod::OsculatingBarycentric => {
                self.osculating_points_at(body, jd, method, convention)
            }
        }
    }

    /// Geocentric Sun vector in the MEAN ecliptic of date (no Δψ), plus the
    /// observer (Earth) velocity in the same frame for aberration.
    fn sun_geo_mean_of_date(&self, jd: f64) -> Result<([f64; 3], [f64; 3]), EventError> {
        let at = |jd: f64| -> Result<[f64; 3], EventError> {
            let (lon, lat, dist) =
                read_mean_ecliptic(&self.backend, CelestialBody::Sun, "Sun", jd)?;
            let p = precess_ecliptic_j2000_to_date(lon, lat, jd)
                .map_err(|e| EventError::Backend(format!("precession failed: {e}")))?;
            Ok(spherical_to_cartesian(p.longitude_deg, p.latitude_deg, dist))
        };
        let s0 = at(jd)?;
        let sm = at(jd - SPEED_HALF_SPAN_DAYS)?;
        let sp = at(jd + SPEED_HALF_SPAN_DAYS)?;
        // Earth heliocentric velocity = −d(geocentric Sun)/dt.
        let v_obs = scale3(sub3(sp, sm), -1.0 / (2.0 * SPEED_HALF_SPAN_DAYS));
        Ok((s0, v_obs))
    }

    /// Mean-element points. Built in the mean ecliptic of date, recentered
    /// geocentric, aberrated (except the Moon), then shifted to the true
    /// equinox (`λ += Δψ`).
    fn mean_points_at(
        &self,
        body: &CelestialBody,
        jd: f64,
        convention: ApsisConvention,
    ) -> Result<[RawPoint; 4], EventError> {
        let second_focus = convention == ApsisConvention::SecondFocus;
        let elements = if *body == CelestialBody::Moon {
            let node =
                read_mean_longitude(&self.backend, CelestialBody::MeanNode, "MeanNode", jd)?;
            let peri = read_mean_longitude(
                &self.backend, CelestialBody::MeanPerigee, "MeanPerigee", jd)?;
            // Backend lunar-point longitudes are J2000-frame at the boundary;
            // bring them to the mean equinox of date like any body read.
            let node = precess_ecliptic_j2000_to_date(node, 0.0, jd)
                .map_err(|e| EventError::Backend(format!("precession failed: {e}")))?
                .longitude_deg;
            let peri = precess_ecliptic_j2000_to_date(peri, 0.0, jd)
                .map_err(|e| EventError::Backend(format!("precession failed: {e}")))?
                .longitude_deg;
            KeplerianElements {
                node_deg: node,
                peri_lon_deg: peri,
                incl_deg: MOON_MEAN_INCL_DEG,
                eccentricity: MOON_MEAN_ECC,
                semi_major_au: MOON_MEAN_SEMA_AU,
            }
        } else {
            let idx = elem_index(body).ok_or_else(|| EventError::UnsupportedNodAps {
                detail: format!("no SE mean elements for {body}; use Osculating"),
            })?;
            mean_elements_of_date(idx, jd)
        };
        let pts = points_from_elements(&elements, second_focus).map_err(|e| {
            EventError::DegenerateNodAps { detail: format!("{body} mean elements: {e:?}") }
        })?;
        let in_plane = [pts.ascending, pts.descending, pts.perihelion, pts.aphelion];
        let dpsi_deg = nutation(jd)
            .map_err(|e| EventError::Backend(format!("nutation failed: {e}")))?
            .delta_psi_arcsec
            / 3600.0;
        let mut out = [RawPoint { lon_deg: 0.0, lat_deg: 0.0, dist_au: 0.0 }; 4];
        if *body == CelestialBody::Moon {
            // Geocentric orbit: points are already geocentric; SE disables
            // aberration for the geocentric Moon.
            for (o, p) in out.iter_mut().zip(in_plane.iter()) {
                let mut raw = apsis_to_raw(p);
                raw.lon_deg = (raw.lon_deg + dpsi_deg).rem_euclid(360.0);
                *o = raw;
            }
            return Ok(out);
        }
        let (sun_geo, v_obs) = self.sun_geo_mean_of_date(jd)?;
        for (o, p) in out.iter_mut().zip(in_plane.iter()) {
            let helio = spherical_to_cartesian(p.longitude_deg, p.latitude_deg, p.distance_au);
            // Sun: SE negates the heliocentric Earth-orbit point to produce
            // the geocentric "Black Sun" point (swecl.c:5482-5484). Planets:
            // heliocentric point + geocentric Sun = geocentric point.
            let geo = if *body == CelestialBody::Sun {
                scale3(helio, -1.0)
            } else {
                add3(helio, sun_geo)
            };
            let aberrated = aberrate(geo, v_obs);
            let mut raw = cartesian_to_raw(aberrated);
            raw.lon_deg = (raw.lon_deg + dpsi_deg).rem_euclid(360.0);
            *o = raw;
        }
        Ok(out)
    }

    /// Placeholder until the osculating task lands.
    fn osculating_points_at(
        &self,
        body: &CelestialBody,
        _jd: f64,
        _method: NodApsMethod,
        _convention: ApsisConvention,
    ) -> Result<[RawPoint; 4], EventError> {
        Err(EventError::UnsupportedNodAps {
            detail: format!("osculating nod_aps for {body} not yet implemented"),
        })
    }
}
```

- [ ] **Step 4: Run** — `cargo test -p pleiades-events`; Expected: PASS (the placeholder keeps osculating requests fail-closed, so all Task-4 tests pass).
- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/src/nod_aps.rs
git commit -m "feat(events): SP-4 nod_aps entry point + mean-element path (planets, Sun, Moon)"
```

---

### Task 5: Osculating path

**Files:**
- Modify: `crates/pleiades-events/src/nod_aps.rs` (replace the placeholder `osculating_points_at`)

**Interfaces:**
- Consumes: `elements_from_state` (Task 1), `mu_au3_day2`/`SUN_MASS_RATIO`/`EARTH_MOON_MASS_RATIO` (Task 3), frame helpers (Task 4).
- Produces: working `NodApsMethod::Osculating` / `OsculatingBarycentric` for Sun, Moon, Mercury–Pluto, `Ceres|Pallas|Juno|Vesta`, `Custom("asteroid", …)`, and the 19 fictitious variants — i.e. any body the backend chain serves.

- [ ] **Step 1: Write the failing test** (append inline; uses only `LinearSunMoon`, real-data tests come in Task 6)

```rust
    #[test]
    fn osculating_moon_on_linear_backend_is_degenerate_not_panicking() {
        // LinearSunMoon's Moon moves on a straight line — the osculating
        // "orbit" it implies is unbound/degenerate. The engine must return a
        // typed error, not NaNs or a panic.
        let engine = EventEngine::new(LinearSunMoon::new_moon_at(2_451_545.0));
        let res = engine.nod_aps(
            CelestialBody::Moon, tdb(2_451_545.0),
            NodApsMethod::Osculating, ApsisConvention::Aphelion,
        );
        assert!(matches!(
            res,
            Err(EventError::DegenerateNodAps { .. })
                | Err(EventError::Backend(_))
                | Err(EventError::MissingCoordinates { .. })
        ));
    }
```

- [ ] **Step 2: Run to verify the current placeholder fails it** — `cargo test -p pleiades-events osculating_moon`; Expected: FAIL (placeholder returns `UnsupportedNodAps`, not one of the accepted variants).

- [ ] **Step 3: Implement.** Replace the placeholder with:

```rust
    /// Body geocentric position in the TRUE ecliptic of date (precession + Δψ),
    /// as a Cartesian vector.
    fn body_geo_true_of_date(
        &self,
        body: &CelestialBody,
        label: &'static str,
        jd: f64,
    ) -> Result<[f64; 3], EventError> {
        let (lon, lat, dist) = read_mean_ecliptic(&self.backend, body.clone(), label, jd)?;
        let p = precess_ecliptic_j2000_to_date(lon, lat, jd)
            .map_err(|e| EventError::Backend(format!("precession failed: {e}")))?;
        let dpsi_deg = nutation(jd)
            .map_err(|e| EventError::Backend(format!("nutation failed: {e}")))?
            .delta_psi_arcsec
            / 3600.0;
        Ok(spherical_to_cartesian(
            (p.longitude_deg + dpsi_deg).rem_euclid(360.0),
            p.latitude_deg,
            dist,
        ))
    }

    /// Sun-to-SSB offset R in the true ecliptic of date, from the giant-planet
    /// heliocentric vectors weighted by their SE mass ratios (Jupiter–Pluto).
    /// `r_bary = r_helio − R`.
    fn sun_to_ssb_offset(&self, jd: f64, sun_geo: [f64; 3]) -> Result<[f64; 3], EventError> {
        use crate::mean_elements::SUN_MASS_RATIO;
        const GIANTS: [(CelestialBody, &str, usize); 5] = [
            (CelestialBody::Jupiter, "Jupiter", 4),
            (CelestialBody::Saturn, "Saturn", 5),
            (CelestialBody::Uranus, "Uranus", 6),
            (CelestialBody::Neptune, "Neptune", 7),
            (CelestialBody::Pluto, "Pluto", 8),
        ];
        let mut r = [0.0_f64; 3];
        for (body, label, mi) in GIANTS {
            let geo = self.body_geo_true_of_date(&body, label, jd)?;
            let helio = sub3(geo, sun_geo);
            r = add3(r, scale3(helio, 1.0 / SUN_MASS_RATIO[mi]));
        }
        Ok(r)
    }

    /// The body's sampling state, centered for element formation, in the true
    /// ecliptic of date at `jd`. Returns (center_pos, velocity, recentering
    /// vector to go point→geocentric, negate_output).
    #[allow(clippy::type_complexity)]
    fn osculating_state(
        &self,
        body: &CelestialBody,
        jd: f64,
        method: NodApsMethod,
    ) -> Result<([f64; 3], [f64; 3], [f64; 3], bool), EventError> {
        // Per-body sample step (SE: NODE_CALC_INTV for the Moon, 0.001·r for
        // the rest — swecl.c:5256/5264).
        const NODE_CALC_INTV_DAYS: f64 = 1.0e-4;
        let label = "nod-aps body";
        if *body == CelestialBody::Moon {
            let dt = NODE_CALC_INTV_DAYS;
            let s = |jd: f64| self.body_geo_true_of_date(&CelestialBody::Moon, "Moon", jd);
            let (p0, pm, pp) = (s(jd)?, s(jd - dt)?, s(jd + dt)?);
            let vel = scale3(sub3(pp, pm), 1.0 / (2.0 * dt));
            // Geocentric orbit: points come out geocentric already.
            return Ok((p0, vel, [0.0; 3], false));
        }
        // Everything else forms a Sun-centered (or SSB-centered) ellipse.
        let sun = |jd: f64| self.body_geo_true_of_date(&CelestialBody::Sun, "Sun", jd);
        let helio_at = |jd: f64| -> Result<[f64; 3], EventError> {
            if *body == CelestialBody::Sun {
                // SE remaps the Sun to the Earth-Moon barycenter
                // (swecl.c:5116-5117, 5281-5286): EMB = Earth + Moon/(μ_ratio+1),
                // with Earth_helio = −Sun_geo.
                let sun_geo = sun(jd)?;
                let moon_geo = self.body_geo_true_of_date(&CelestialBody::Moon, "Moon", jd)?;
                Ok(add3(
                    scale3(sun_geo, -1.0),
                    scale3(moon_geo, 1.0 / (EARTH_MOON_MASS_RATIO + 1.0)),
                ))
            } else {
                let geo = self.body_geo_true_of_date(body, label, jd)?;
                Ok(sub3(geo, sun(jd)?))
            }
        };
        let h0 = helio_at(jd)?;
        let r_helio = norm3(h0);
        let dt = NODE_CALC_INTV_DAYS * 10.0 * r_helio;
        let use_bary = method == NodApsMethod::OsculatingBarycentric && r_helio > 6.0;
        let recenter_at = |jd: f64| -> Result<[f64; 3], EventError> {
            if use_bary {
                // point_geo = point_bary + Sun_geo + R.
                Ok(add3(sun(jd)?, self.sun_to_ssb_offset(jd, sun(jd)?)?))
            } else {
                sun(jd)
            }
        };
        let centered_at = |jd: f64| -> Result<[f64; 3], EventError> {
            let h = helio_at(jd)?;
            if use_bary {
                let r = self.sun_to_ssb_offset(jd, sun(jd)?)?;
                Ok(sub3(h, r))
            } else {
                Ok(h)
            }
        };
        let (p0, pm, pp) = (centered_at(jd)?, centered_at(jd - dt)?, centered_at(jd + dt)?);
        let vel = scale3(sub3(pp, pm), 1.0 / (2.0 * dt));
        let recenter = if *body == CelestialBody::Sun { [0.0; 3] } else { recenter_at(jd)? };
        Ok((p0, vel, recenter, *body == CelestialBody::Sun))
    }

    /// Osculating points: form the instantaneous ellipse in the true ecliptic
    /// of date, take its four points, recenter geocentric, aberrate
    /// (except the Moon).
    fn osculating_points_at(
        &self,
        body: &CelestialBody,
        jd: f64,
        method: NodApsMethod,
        convention: ApsisConvention,
    ) -> Result<[RawPoint; 4], EventError> {
        let second_focus = convention == ApsisConvention::SecondFocus;
        let (pos, vel, recenter, negate) = self.osculating_state(body, jd, method)?;
        let mu = mu_au3_day2(body);
        let elements = pleiades_apsides::elements_from_state(pos, vel, mu).map_err(|e| {
            EventError::DegenerateNodAps { detail: format!("{body} osculating state: {e:?}") }
        })?;
        let pts = points_from_elements(&elements, second_focus).map_err(|e| {
            EventError::DegenerateNodAps { detail: format!("{body} osculating ellipse: {e:?}") }
        })?;
        let in_frame = [pts.ascending, pts.descending, pts.perihelion, pts.aphelion];
        let mut out = [RawPoint { lon_deg: 0.0, lat_deg: 0.0, dist_au: 0.0 }; 4];
        if *body == CelestialBody::Moon {
            for (o, p) in out.iter_mut().zip(in_frame.iter()) {
                *o = apsis_to_raw(p);
            }
            return Ok(out);
        }
        let (_, v_obs) = self.sun_geo_mean_of_date(jd)?;
        for (o, p) in out.iter_mut().zip(in_frame.iter()) {
            let v = spherical_to_cartesian(p.longitude_deg, p.latitude_deg, p.distance_au);
            let geo = if negate { scale3(v, -1.0) } else { add3(v, recenter) };
            *o = cartesian_to_raw(aberrate(geo, v_obs));
        }
        Ok(out)
    }
```

Note: no `λ += Δψ` here — the osculating frame already carries Δψ (samples were rotated to the TRUE ecliptic of date). The observer velocity from `sun_geo_mean_of_date` is a mean-frame vector; the ≤17″ frame mismatch on a v/c≈20″ correction is < 2 mas — irrelevant, don't add machinery.

- [ ] **Step 4: Run** — `cargo test -p pleiades-events`; Expected: PASS.
- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/src/nod_aps.rs
git commit -m "feat(events): SP-4 osculating nod_aps path (helio/geo/EMB/SSB centers, per-body dt)"
```

---

### Task 6: Integration tests over the real backend chain

**Files:**
- Create: `crates/pleiades-events/tests/nod_aps.rs`

**Interfaces:**
- Consumes: the full public API; the chart-style routing chain (`pleiades-cli/src/commands/chart.rs:560-568` is the model).

- [ ] **Step 1: Write the tests**

```rust
//! nod_aps integration over the production-style backend chain.

use pleiades_backend::{CompositeBackend, RoutingBackend};
use pleiades_data::PackagedDataBackend;
use pleiades_elp::ElpBackend;
use pleiades_events::{ApsisConvention, EventEngine, NodApsMethod};
use pleiades_fict::FictitiousBackend;
use pleiades_jpl::JplSnapshotBackend;
use pleiades_types::{CelestialBody, CustomBodyId, Instant, JulianDay, TimeScale};
use pleiades_vsop87::Vsop87Backend;

fn engine() -> EventEngine<RoutingBackend> {
    EventEngine::new(RoutingBackend::new(vec![
        Box::new(PackagedDataBackend::new()),
        Box::new(CompositeBackend::new(Vsop87Backend::new(), ElpBackend::new())),
        Box::new(JplSnapshotBackend::new()),
        Box::new(FictitiousBackend::new(PackagedDataBackend::new())),
    ]))
}

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

const JD: f64 = 2_451_545.0; // J2000

#[test]
fn planet_nodes_lie_near_the_ecliptic_plane() {
    let engine = engine();
    for body in [CelestialBody::Mercury, CelestialBody::Mars, CelestialBody::Saturn] {
        for method in [NodApsMethod::Mean, NodApsMethod::Osculating] {
            let r = engine
                .nod_aps(body.clone(), tdb(JD), method, ApsisConvention::Aphelion)
                .unwrap();
            // Node points sit in the ecliptic plane; the geocentric direction
            // picks up only Earth's tiny ecliptic latitude.
            assert!(r.ascending.latitude_deg.abs() < 0.1, "{body:?} {method:?}");
            assert!(r.descending.latitude_deg.abs() < 0.1, "{body:?} {method:?}");
            assert!(r.perihelion.distance_au.is_finite() && r.perihelion.distance_au > 0.0);
        }
    }
}

#[test]
fn moon_mean_node_matches_the_backend_mean_node_longitude() {
    let engine = engine();
    let r = engine
        .nod_aps(CelestialBody::Moon, tdb(JD), NodApsMethod::Mean, ApsisConvention::Aphelion)
        .unwrap();
    // Mean lunar node at J2000 ≈ 125.04° (Meeus); allow frame/nutation slack.
    assert!((r.ascending.longitude_deg - 125.04).abs() < 1.0, "{}", r.ascending.longitude_deg);
    // Mean node regresses ≈ −0.0529 deg/day.
    assert!((r.ascending.longitude_speed_deg_per_day + 0.0529).abs() < 0.01);
    // Perigee distance ≈ a(1−e) with SE's mean scalars.
    assert!((r.perihelion.distance_au - 0.00256955 * (1.0 - 0.054900489)).abs() < 1e-5);
}

#[test]
fn moon_osculating_node_is_near_the_true_node() {
    let engine = engine();
    let r = engine
        .nod_aps(CelestialBody::Moon, tdb(JD), NodApsMethod::Osculating, ApsisConvention::Aphelion)
        .unwrap();
    // The osculating node oscillates around the mean node within ~±2°.
    assert!((pleiades_events_test_wrap(r.ascending.longitude_deg, 125.0)).abs() < 3.0);
}

fn pleiades_events_test_wrap(a: f64, b: f64) -> f64 {
    let mut d = (a - b).rem_euclid(360.0);
    if d > 180.0 {
        d -= 360.0;
    }
    d
}

#[test]
fn second_focus_scales_the_moon_apogee_distance() {
    let engine = engine();
    let apo = engine
        .nod_aps(CelestialBody::Moon, tdb(JD), NodApsMethod::Mean, ApsisConvention::Aphelion)
        .unwrap()
        .aphelion;
    let foc = engine
        .nod_aps(CelestialBody::Moon, tdb(JD), NodApsMethod::Mean, ApsisConvention::SecondFocus)
        .unwrap()
        .aphelion;
    let e = 0.054900489_f64;
    assert!((foc.distance_au / apo.distance_au - 2.0 * e / (1.0 + e)).abs() < 1e-9);
    assert!((foc.longitude_deg - apo.longitude_deg).abs() < 1e-9);
}

#[test]
fn barycentric_falls_back_heliocentric_inside_six_au() {
    let engine = engine();
    let oscu = engine
        .nod_aps(CelestialBody::Mars, tdb(JD), NodApsMethod::Osculating, ApsisConvention::Aphelion)
        .unwrap();
    let bar = engine
        .nod_aps(CelestialBody::Mars, tdb(JD), NodApsMethod::OsculatingBarycentric,
                 ApsisConvention::Aphelion)
        .unwrap();
    assert!((oscu.ascending.longitude_deg - bar.ascending.longitude_deg).abs() < 1e-9);
    // …and diverges beyond it.
    let n_oscu = engine
        .nod_aps(CelestialBody::Neptune, tdb(JD), NodApsMethod::Osculating,
                 ApsisConvention::Aphelion)
        .unwrap();
    let n_bar = engine
        .nod_aps(CelestialBody::Neptune, tdb(JD), NodApsMethod::OsculatingBarycentric,
                 ApsisConvention::Aphelion)
        .unwrap();
    assert!((n_oscu.ascending.longitude_deg - n_bar.ascending.longitude_deg).abs() > 1e-6);
}

#[test]
fn default_method_matches_se_semantics() {
    let engine = engine();
    let venus = engine
        .nod_aps_default(CelestialBody::Venus, tdb(JD), ApsisConvention::Aphelion)
        .unwrap();
    assert_eq!(venus.method, NodApsMethod::Mean);
    let pluto = engine
        .nod_aps_default(CelestialBody::Pluto, tdb(JD), ApsisConvention::Aphelion)
        .unwrap();
    assert_eq!(pluto.method, NodApsMethod::Osculating);
}

#[test]
fn asteroid_and_fictitious_bodies_compose_through_the_chain() {
    let engine = engine();
    for body in [
        CelestialBody::Ceres,
        CelestialBody::Custom(CustomBodyId::new("asteroid", "2060-Chiron")),
        CelestialBody::Cupido,
    ] {
        let r = engine
            .nod_aps(body.clone(), tdb(JD), NodApsMethod::Osculating, ApsisConvention::Aphelion)
            .unwrap_or_else(|e| panic!("{body:?}: {e}"));
        assert!(r.perihelion.distance_au.is_finite() && r.perihelion.distance_au > 0.0);
        assert!(r.ascending.latitude_deg.abs() < 0.5, "{body:?}");
    }
}
```

- [ ] **Step 2: Run to verify current state** — `cargo test -p pleiades-events --test nod_aps`; Expected: compile then PASS if Tasks 4–5 are correct; investigate any failure as a real defect (these are behavioral invariants, not tuned numbers — if one fails, debug the pipeline; do not loosen the assert without understanding why).
- [ ] **Step 3: Adjust exports if the test surfaced a missing item** (e.g. `RoutingBackend`/`CompositeBackend` paths — check `pleiades_backend`'s re-exports with `cargo doc -p pleiades-backend --no-deps` or the `chart.rs` imports and fix the test's `use` lines accordingly).
- [ ] **Step 4: Run the full crate + workspace check**

Run: `cargo test -p pleiades-events && cargo clippy -p pleiades-events -p pleiades-apsides -- -D warnings`
Expected: PASS, no warnings.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-events/tests/nod_aps.rs
git commit -m "test(events): SP-4 nod_aps integration invariants over the routing chain"
```

---

### Task 7: `tools/se-nodaps-reference` + committed corpus

**Files:**
- Create: `tools/se-nodaps-reference/Cargo.toml`, `tools/se-nodaps-reference/src/main.rs`
- Create: `tools/se-nodaps-reference/data/seorbel.txt` (copy of `tools/se-fictitious-reference/data/seorbel.txt`)
- Create (generated): `crates/pleiades-validate/data/nod-aps-corpus/nod-aps.csv`, `manifest.txt`
- Modify: root `Cargo.toml` — add `tools/se-nodaps-reference` to the workspace `exclude` list.

**Interfaces:**
- Produces: committed corpus CSV with header
  `label,se_body,method,fopoint,jd_tt,asc_lon,asc_lat,asc_dist,asc_dlon,asc_dlat,asc_ddist,dsc_lon,dsc_lat,dsc_dist,dsc_dlon,dsc_dlat,dsc_ddist,peri_lon,peri_lat,peri_dist,peri_dlon,peri_dlat,peri_ddist,apo_lon,apo_lat,apo_dist,apo_dlon,apo_dlat,apo_ddist`
  and `manifest.txt` line `file: nod-aps.csv rows=<n> checksum=<fnv1a64>`.

- [ ] **Step 1: Write `Cargo.toml`** (clone the fictitious tool's shape)

```toml
[package]
name = "se-nodaps-reference"
version = "0.0.0"
edition = "2021"
publish = false

[workspace]

[dependencies]
libswisseph-sys = "0.1.2"
```

- [ ] **Step 2: Write `src/main.rs`.** Follow `tools/se-fictitious-reference/src/main.rs` exactly for: provenance doc header, the local `fnv1a64` copy, fail-closed panics on SE errors/non-finite values, `--dry-run`, and manifest emission. The tool-specific parts:

```rust
use libswisseph_sys::raw::{swe_nod_aps, swe_set_ephe_path, swe_version};
use std::os::raw::{c_char, c_double, c_int};

// SE default output minus gravitational deflection (see plan §R3):
// SEFLG_MOSEPH=4, SEFLG_SPEED=256, SEFLG_NOGDEFL=512.
const IFLAG: c_int = 4 | 256 | 512;
const SE_NODBIT_FOPOINT: c_int = 256;

// Epochs spanning 1900–2100, ≥ 2 days inside the packaged window.
const EPOCHS: [f64; 8] = [
    2_415_100.5, 2_433_282.5, 2_441_683.5, 2_451_545.0,
    2_459_000.5, 2_466_154.5, 2_477_476.5, 2_488_021.5,
];
const EPOCHS_SHORT: [f64; 5] =
    [2_415_100.5, 2_433_282.5, 2_451_545.0, 2_466_154.5, 2_488_021.5];

/// (label, SE body number). SE 0–9 planets; 15–20 small bodies (need
/// seas_18.se1); 40–58 fictitious (need seorbel.txt in the ephe dir).
const PLANETS: [(&str, c_int); 10] = [
    ("Sun", 0), ("Moon", 1), ("Mercury", 2), ("Venus", 3), ("Mars", 4),
    ("Jupiter", 5), ("Saturn", 6), ("Uranus", 7), ("Neptune", 8), ("Pluto", 9),
];

fn emit_row(
    csv: &mut String,
    label: &str,
    se_body: c_int,
    method: c_int,
    fopoint: bool,
    jd_tt: f64,
) {
    let mut xnasc = [0.0_c_double; 6];
    let mut xndsc = [0.0_c_double; 6];
    let mut xperi = [0.0_c_double; 6];
    let mut xaphe = [0.0_c_double; 6];
    let mut serr = [0 as c_char; 256];
    let m = method | if fopoint { SE_NODBIT_FOPOINT } else { 0 };
    let ret = unsafe {
        swe_nod_aps(
            jd_tt, se_body, IFLAG, m,
            xnasc.as_mut_ptr(), xndsc.as_mut_ptr(),
            xperi.as_mut_ptr(), xaphe.as_mut_ptr(),
            serr.as_mut_ptr(),
        )
    };
    assert!(ret >= 0, "swe_nod_aps failed for {label} method {m} at {jd_tt}");
    for v in xnasc.iter().chain(&xndsc).chain(&xperi).chain(&xaphe) {
        assert!(v.is_finite(), "non-finite output for {label} at {jd_tt}");
    }
    csv.push_str(&format!("{label},{se_body},{method},{},{jd_tt:.9}", u8::from(fopoint)));
    for pt in [&xnasc, &xndsc, &xperi, &xaphe] {
        for v in pt.iter() {
            csv.push_str(&format!(",{v:.9}"));
        }
    }
    csv.push('\n');
}
```

`build_csv` emits, in order:
1. Mean rows: `(Sun, Moon, Mercury..Neptune)` × `EPOCHS` × `method=1`, `fopoint=false` → 80 rows.
2. Mean focal-point rows: `(Moon, Mercury)` × `EPOCHS_SHORT[..3]` × `method=1, fopoint=true` → 6 rows.
3. Osculating rows: `(Sun, Moon, Mercury..Pluto)` × `EPOCHS` × `method=2` → 88 rows.
4. Barycentric rows: `(Jupiter, Saturn, Uranus, Neptune, Pluto)` × `EPOCHS_SHORT[..4]` × `method=4` → 20 rows (Jupiter pins the ≤6 AU heliocentric fallback).
5. Osculating focal-point rows: `(Mars, Neptune)` × `EPOCHS_SHORT[..3]` × `method=2, fopoint=true` → 6 rows.
6. Small bodies: `("Chiron",15), ("Pholus",16), ("Ceres",17), ("Pallas",18), ("Juno",19), ("Vesta",20)` × `EPOCHS_SHORT` × `method=2` → 30 rows. **Guarded:** before emitting, verify `<ephe_dir>/seas_18.se1` exists AND its SHA-256 equals the constant pinned in the tool source (`const SEAS_18_SHA256: &str = "<fill in from the actual file at generation time>";` — compute with `sha256sum`, paste, commit). If absent, panic with a download hint (`https://www.astro.com/ftp/swisseph/ephe/seas_18.se1`) unless `--skip-small-bodies` was passed. Implement SHA-256 locally (no new deps): copy the checksum approach used by the repo's kernel-pinning tool if one exists in `tools/`; otherwise shell out is NOT allowed in-tool — instead read the file and use a small embedded SHA-256 implementation (~60 lines, public-domain style, with a unit test hashing `b"abc"` to `ba7816bf...`).
7. Fictitious rows: SE bodies 40–58 with the same labels as the fictitious corpus × `EPOCHS_SHORT` × `method=2` → 95 rows.

Total: **325 rows** (drop the placeholder tuple in `PLANETS`; use a clean 10-entry array — the count math above is the contract).

- [ ] **Step 3: Copy `seorbel.txt`**

```bash
mkdir -p tools/se-nodaps-reference/data
cp tools/se-fictitious-reference/data/seorbel.txt tools/se-nodaps-reference/data/seorbel.txt
```

- [ ] **Step 4: Obtain `seas_18.se1`, pin, generate.** (Needs `libclang-dev` + `LIBCLANG_PATH` for the FFI build — same as every `se-*-reference` tool.)

```bash
cd tools/se-nodaps-reference
curl -fLo data/seas_18.se1 https://www.astro.com/ftp/swisseph/ephe/seas_18.se1
sha256sum data/seas_18.se1   # paste into SEAS_18_SHA256, rebuild
cargo run --release -- --out ../../crates/pleiades-validate/data/nod-aps-corpus
```

Expected: writes `nod-aps.csv` (325 data rows) + `manifest.txt`. `data/seas_18.se1` must be listed in `tools/se-nodaps-reference/.gitignore` (create it: one line `seas_18.se1`) — the SHA constant is committed, the file is not.

- [ ] **Step 5: Sanity-check and commit**

```bash
head -4 crates/pleiades-validate/data/nod-aps-corpus/nod-aps.csv
grep -c . crates/pleiades-validate/data/nod-aps-corpus/nod-aps.csv
cat crates/pleiades-validate/data/nod-aps-corpus/manifest.txt
git add tools/se-nodaps-reference crates/pleiades-validate/data/nod-aps-corpus Cargo.toml
git commit -m "tools: SP-4 Swiss-Ephemeris nod-aps reference generator + committed 325-row corpus"
```

Inspect the Sun mean rows now (relevant to §R8): if `asc_*`/`dsc_*` columns are all zero, note it for Task 9's zeroing decision.

---

### Task 8: `validate-nod-aps` gate

**Files:**
- Create: `crates/pleiades-validate/src/nod_aps_thresholds.rs`, `crates/pleiades-validate/src/nod_aps_validation.rs`
- Modify: `crates/pleiades-validate/src/lib.rs`, `src/render/cli.rs`, `src/tests/validate_gates.rs`, `Cargo.toml`

**Interfaces:**
- Consumes: the corpus (Task 7), `pleiades_events::EventEngine::nod_aps`, `pleiades_apparent::fnv1a64`.
- Produces: `pub fn validate_nod_aps_corpus() -> Result<NodApsReport, NodApsError>`, `NodApsReport { rows, skipped_categories… }` with `passed()` / `summary_line()`, `pub const EXPECTED_ROWS: usize = 325;`, CLI `validate-nod-aps` / `nod-aps-gate`.

- [ ] **Step 1: Write `nod_aps_thresholds.rs`** — provisional GENEROUS ceilings (tightened in Task 9), one `pub const` per category × metric, mirroring `fictitious_thresholds.rs`'s doc style. Categories: `MEAN_PLANET`, `MEAN_MOON`, `OSCU_PLANET`, `OSCU_MOON`, `OSCU_SMALLBODY`, `OSCU_FICTITIOUS`; metrics: `_LONGITUDE_ARCSEC`, `_LATITUDE_ARCSEC`, `_DISTANCE_REL`, `_LON_SPEED_DEG_DAY`. Provisional values: `3600.0` arcsec, `3600.0` arcsec, `1e-2` rel, `1.0` deg/day for every category.

- [ ] **Step 2: Write `nod_aps_validation.rs`** by direct imitation of `fictitious_validation.rs` (same `include_str!` + manifest + `check_checksum` via `pleiades_apparent::fnv1a64` + `EXPECTED_ROWS` + measure/validate split + error/report shape). The gate-specific parts:

```rust
const CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"), "/data/nod-aps-corpus/nod-aps.csv"));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"), "/data/nod-aps-corpus/manifest.txt"));
const CSV_FILE: &str = "nod-aps.csv";
pub const EXPECTED_ROWS: usize = 325;
```

Row parse: 29 fields per the Task-7 schema. Body mapping `body_from_se(se_body: i32) -> Option<CelestialBody>`: 0 Sun, 1 Moon, 2–9 Mercury…Pluto, 15 `Custom("asteroid","2060-Chiron")`, 16 `Custom("asteroid","5145-Pholus")`, 17–20 Ceres/Pallas/Juno/Vesta, 40–58 the fictitious variants (copy the arm list from `fictitious_validation.rs::body_from_se`). Method mapping: 1→`NodApsMethod::Mean`, 2→`Osculating`, 4→`OsculatingBarycentric`; `fopoint` 0→`Aphelion`, 1→`SecondFocus`.

Backend + engine (the production-style chain; add `pleiades-vsop87`, `pleiades-elp`, `pleiades-jpl`, `pleiades-fict` to `[dependencies]` in `crates/pleiades-validate/Cargo.toml` if not already present — `pleiades-events`/`pleiades-data` already are):

```rust
let backend = RoutingBackend::new(vec![
    Box::new(PackagedDataBackend::new()),
    Box::new(CompositeBackend::new(Vsop87Backend::new(), ElpBackend::new())),
    Box::new(JplSnapshotBackend::new()),
    Box::new(FictitiousBackend::new(PackagedDataBackend::new())),
]);
let engine = EventEngine::new(backend);
```

Per row: call `engine.nod_aps(body, instant, method, convention)`; **Tier 1** invariants — every recomputed component finite, longitudes in `[0,360)`, latitudes in `[-90,90]`, distances > 0; **Tier 2** residuals per point (all four) — `|Δλ|` wrap-aware in arcsec, `|Δβ|` arcsec, `|Δd|/d_se` relative, `|Δ(dλ/dt)|` deg/day — accumulated into per-category maxima keyed by `(se_body, method)`: Moon → `*_MOON`, Sun/planets → `*_PLANET` (mean vs oscu; method 4 counts as oscu), 15–20 → `OSCU_SMALLBODY`, 40–58 → `OSCU_FICTITIOUS`. Report carries a max per category × metric; `validate_nod_aps_corpus()` applies the ceilings fail-closed (`ToleranceExceeded { category, label, residual, ceiling }`). `summary_line()` must state the small-body coverage bound explicitly, e.g. `"…; SE small-body reference limited to Chiron/Pholus/Ceres/Pallas/Juno/Vesta (seas_18.se1); remaining Tier-A asteroids engine-covered, gate-unreferenced"`.

- [ ] **Step 3: Register.** `lib.rs`: `mod nod_aps_thresholds;` + `pub mod nod_aps_validation;` next to the fictitious pair; re-export `pub use nod_aps_validation::{validate_nod_aps_corpus, NodApsError, NodApsReport};`. `render/cli.rs`: append to `run_all_numeric_gates()` after the fictitious line:

```rust
    crate::validate_nod_aps_corpus().map_err(|e| format!("nod-aps gate failed: {e}"))?;
```

Match arm (next to the fictitious arm):

```rust
    Some("validate-nod-aps") | Some("nod-aps-gate") => {
        ensure_no_extra_args(&args[1..], "validate-nod-aps")?;
        crate::validate_nod_aps_corpus()
            .map(|r| r.summary_line())
            .map_err(|e| e.to_string())
    }
```

Help banner (two lines, next to the fictitious entries):

```
validate-nod-aps          Planetary/lunar nodes+apsides SE-parity gate (swe_nod_aps)
nod-aps-gate              Alias for validate-nod-aps
```

- [ ] **Step 4: Tests.** In `src/tests/validate_gates.rs`, mirror the fictitious alias/help tests (`nod-aps-gate` runs and succeeds; help mentions both spellings). Near `cli.rs`'s `run_all_numeric_gates_includes_fictitious_and_passes`, add `run_all_numeric_gates_includes_nod_aps_and_passes`.

- [ ] **Step 5: Run**

Run: `cargo test -p pleiades-validate validate_gates && cargo run -p pleiades-cli -- validate-nod-aps` (or however the repo invokes gate commands — copy the invocation from the fictitious gate's test).
Expected: gate PASSES under the provisional ceilings and prints a summary line with per-category maxima. **Record the printed maxima — Task 9 needs them.** If any category exceeds even the provisional ceilings, STOP and debug the pipeline (likely suspects, in order: mean-vs-true equinox mix-up (§R4), missing aberration (§R3), the Sun negation (§R1), μ constants, sample step sign).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-validate
git commit -m "feat(validate): SP-4 validate-nod-aps SE-parity gate (provisional ceilings)"
```

---

### Task 9: Measure and pin ceilings (+ Sun node semantics)

**Files:**
- Modify: `crates/pleiades-validate/src/nod_aps_thresholds.rs`
- Possibly modify: `crates/pleiades-events/src/nod_aps.rs` (§R8 zeroing)

- [ ] **Step 1: Decide the Sun-node question (§R8).** Look at the corpus Sun rows' `asc_*`/`dsc_*` columns. If SE zeroed them, add to `mean_points_at` and `osculating_points_at` (before returning) — `if *body == CelestialBody::Sun { out[0] = RawPoint { lon_deg: 0.0, lat_deg: 0.0, dist_au: 0.0 }; out[1] = out[0]; }` — with a comment citing swecl.c:5436-5439, and exclude zero-row point comparisons from Tier-2 (compare them exactly instead). If SE emitted real values, keep the mechanical output and say so in the module docs.

- [ ] **Step 2: Re-run the gate, harvest per-category maxima** from the summary line (`cargo run -p pleiades-cli -- validate-nod-aps` or the repo's gate invocation).

- [ ] **Step 3: Pin ceilings at ~1.4× measured maxima** (round up to clean values), replacing every provisional constant; document the measured max in each constant's doc comment (fictitious_thresholds style: `/// Measured max X on YYYY-MM-DD corpus; ceiling ~1.4×.`). Expected magnitudes (sanity prior, not requirements): mean-planet ≤ a few ″; oscu-planet arcsecond-class; mean-moon possibly tens-of-″ (ELP-vs-SE mean series, §R2); oscu small-body/fictitious cross-theory, up to arcminute-class. If mean-moon exceeds ~120″, keep the measured ceiling AND open the §R2 follow-up in `docs/follow-ups.md`.

- [ ] **Step 4: Full verification**

Run: `cargo test --workspace`
Expected: PASS — including `run_all_numeric_gates_includes_nod_aps_and_passes` under the pinned ceilings.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate crates/pleiades-events docs/follow-ups.md
git commit -m "feat(validate): SP-4 pin nod-aps gate ceilings from measured residuals"
```

---

### Task 10: Docs + compatibility profile closeout

**Files:**
- Modify: `crates/pleiades-core/src/compatibility/mod.rs`, `crates/pleiades-validate/src/tests/render_request.rs:333`, `crates/pleiades-cli/src/cli/tests/summary_commands.rs:433`, `README.md`, `PLAN.md`, `plan/status/02-next-slice-candidates.md`

- [ ] **Step 1: Profile bump.** In `compatibility/mod.rs`: set `CURRENT_COMPATIBILITY_PROFILE_ID` to `"pleiades-compatibility-profile/0.7.10"`; append an `SP-4 (planetary nodes and apsides) additions:` paragraph to `CURRENT_COMPATIBILITY_PROFILE_SUMMARY` (SP-3's paragraph is the template — name the API, methods, body coverage incl. the small-body reference bound, gate name/aliases, and measured accuracy per category); update the trailer to 0.7.10. Run the profile checksum test, take the new value from the failure message, update `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM` (same commit — the doc comment at mod.rs:31-36 describes this exact loop).

- [ ] **Step 2: Version-string tests.** Update the two `0.7.9` assertions (`render_request.rs:333`, `summary_commands.rs:433`) to `0.7.10`. API-stability stays `0.2.2`.

- [ ] **Step 3: Docs.** README "Current state": add a bullet for SP-4 (mirror the SP-3 bullet's structure: what shipped, gate, measured accuracy per category, the small-body reference bound, and the §R2 Moon-mean caveat if it materialized). PLAN.md status line: append the SP-4 completion sentence with date + measured numbers, and remove `swe_nod_aps` from the remaining-follow-ups list. `plan/status/02-next-slice-candidates.md`: mark SP-4 done in the event-engine track; remaining candidates become `swe_pheno`, custom fictitious elements, occultations, central-path cartography.

- [ ] **Step 4: Full workspace verification**

Run: `cargo test --workspace && cargo clippy --workspace -- -D warnings`
Expected: PASS, no warnings.

- [ ] **Step 5: Commit**

```bash
git add README.md PLAN.md plan/status crates/pleiades-core crates/pleiades-validate crates/pleiades-cli
git commit -m "docs(events): SP-4 declare planetary nodes/apsides; profile 0.7.10; mark SP-4 done"
```

---

## Self-review notes (already applied)

- **Spec coverage:** API/types → Tasks 2/4; mean path → Tasks 3/4; osculating incl. OSCU_BAR + FOPOINT → Task 5; full body coverage by composition → Tasks 5/6; gate + corpus + ceilings → Tasks 7–9; docs/profile → Task 10. Spec's "pinned empirically" items are all resolved by the Reconciliation section (§R1–§R10).
- **Type consistency:** `KeplerianElements`/`points_from_elements`/`elements_from_state` names match across Tasks 1/3/4/5; `RawPoint`/`points_at` internal seam matches Tasks 4/5; corpus schema matches Tasks 7/8; `EXPECTED_ROWS = 325` matches the Task-7 row math (80+6+88+20+6+30+95).
- **Known deliberate deviations from SE:** speeds via ±0.5-day pipeline differences (§R5); gravitational deflection omitted with corpus generated to match (§R3); Moon mean longitudes from ELP not `swi_mean_lunar_elements` (§R2); Earth not addressable (§R1). All are measured by the gate, none are silent.
