# Follow-ups / deferred items

Tracked engineering items deferred out of the scope that surfaced them. Each
entry: what, where, evidence, impact, suggested fix, and origin.

---

## FU-2: True (osculating) lunar apsides sub-project

**Status:** resolved (2026-06-30) · Implemented by `feat/true-lilith-osculating-apsides` branch (Tasks 1–8). `TrueApogee` and `TruePerigee` are now served release-grade by `PackagedDataBackend` via the `crates/pleiades-apsides` crate (osculating Kepler apse from Moon pos+vel+mu). Gated against Swiss Ephemeris `SE_OSCU_APOG` Moshier corpus (3177 samples, 1900–2100) by `validate-lilith`; gate parity as of 2026-06-30: max longitude residual ~306″ (~5.1′), latitude ~53″, distance ~1.6e-4 relative, vs ceilings 460″/80″/2.34e-4. Of-date frame = true ecliptic of date via precession + nutation-in-longitude only (no light-time, no aberration — geometric direction). · **Next queued:** equatorial/declination output for `TrueApogee`/`TruePerigee` (chart-layer apparent equatorial shipped 2026-06-30 on `feat/equatorial-declination-output` for release-grade bodies; apsides equatorial follows when their release-grade status expands). · **Build-env note:** the reference tool `tools/se-lilith-reference` (used to generate the committed SE_OSCU_APOG corpus CSV) requires `libclang` + `LIBCLANG_PATH` to build Rust bindings to the vendored Swiss Ephemeris — provided by `devenv.nix` (`devenv shell -- cargo run ...`), the sanctioned way to get native libraries per `AGENTS.md`. This is NOT required to run the `validate-lilith` gate or build the workspace — the gate reads the committed corpus CSV via `include_str!` and never rebuilds the tool. · **Severity:** feature gap (now closed) · **Opened:** 2026-06-30

---

## FU-1: Latent geocentric-Sun aberration double-count in `pleiades-core` apparent path

**Status:** resolved (2026-06-30) · Fixed by `apparent_sun_position` in pleiades-apparent (cc575c04); chart Sun path applies aberration once (a6113705); eclipse delegates to the shared routine (70a2adf2); Sun golden tolerance tightened 26″ → 5.0″, measured residual max 2.83″ (eb4339f2). · **Addendum (2026-09-30):** the "planets are unaffected" reasoning below was wrong: the geocentric light-time re-query retards the Earth too and so already carries aberration, and the generic path added it a second time for every non-Sun body. Fixed under issue #93 (`docs/superpowers/specs/2026-09-30-apparent-aberration-double-count-design.md`); the Moon's remaining residual is FU-14. · **Severity:** important (accuracy) · **Opened:** 2026-06-29

**Where:** `crates/pleiades-core/src/chart/mod.rs` (~lines 304–313, the
`apparent_position::<_, EphemerisError>(instant, sun_lon, max_iter, query)`
call whose `query` closure re-queries the **geocentric** Sun at each
light-time-retarded epoch, while `apparent_position` also adds annual
aberration internally — `crates/pleiades-apparent/src/apparent.rs:31`).

**The bug:** For the **Sun observed geocentrically**, the light-time
retardation and the annual (stellar) aberration are the *same* Earth-orbital
reflex-motion effect (~20.5″), not two independent corrections — Meeus,
*Astronomical Algorithms* §25. Re-querying the geocentric Sun at `t − τ`
(τ ≈ 499 s) already displaces it ~20.5″; adding the annual-aberration term on
top double-counts it, producing a systematic ~+20″ error in the apparent solar
ecliptic longitude. (At the time this was thought Sun-specific; see the 2026-09-30 addendum in the status line — it was not.)

**Evidence:** The `pleiades-eclipse` work (this phase) proved the *same*
packaged backend matches an independent Skyfield 1.54 + DE440 apparent solar
longitude to **~0.5″** once aberration is applied **once** (see
`crates/pleiades-eclipse/src/ephemeris.rs::apparent_sun_longitude_deg` and the
`validate-eclipses` gate's ≤1.0″ longitude tolerance passing on all 908
in-coverage rows). Meanwhile the chart apparent path is masked: its golden
fixture gives the Sun a **26″** tolerance
(`crates/pleiades-validate/data/apparent-goldens.csv`, ~lines 7, 29–33) and the
header attributes the observed ~15–25″ residuals to ephemeris-fit error. The
eclipse result strongly suggests much of that residual is the double-count, not
fit error.

**Impact:** Apparent Sun ecliptic longitude in chart placements is off by
~20″ (≈ 0.006°). Below the 26″ golden tolerance today, so no test fails, but
it is a real systematic inaccuracy in a release-grade body.

**Suggested fix:** Special-case the Sun so the aberration/light-time correction
is applied once (mirror `pleiades-eclipse`'s `apparent_sun_longitude_deg`), or
make `apparent_position` aware that for a Sun query the light-time re-query and
annual aberration are the same effect. Then **regenerate and tighten** the Sun
rows in `apparent-goldens.csv` (the 26″ tolerance can drop toward ~1″) so the
fix is locked in by the apparent gate. Verify the Moon path separately.

**Origin:** Discovered during `phase6-eclipse-subsystem` (merged as
`00166809`); flagged by the Task 10B and final whole-branch (opus) reviews as
explicitly out of scope for the eclipse branch.

---

## FU-3: Backend J2000 ecliptic frame correction sub-project

**Status:** resolved (2026-06-30) · Implemented on `feat/equatorial-declination-output` branch (Tasks B1–B7). All first-party backends (`pleiades-vsop87`, `pleiades-elp`, `pleiades-data`) now emit a **consistent J2000 ecliptic (both longitude AND latitude)** at the backend boundary. Previously, latitude was silently "of-date" (accumulated nutation rotation not reverted), creating a mixed-frame boundary that affected topocentric latitude accuracy. Changes: (1) reverted of-date latitude band-aid in SPK reduction path, (2) brought ELP lunar theory to J2000 latitude as well, (3) added keystone **`validate-frame-consistency` / `validate_frame_consistency`** gate (≥17 representative body/epoch rows spanning 1900–2100) to the release posture — this gate permanently pins the J2000-boundary invariant, (4) recalibrated topocentric latitude tolerance to match the corrected J2000 output. · **Spec/plan:** `docs/superpowers/specs/2026-06-30-backend-j2000-ecliptic-frame-correction-design.md` + `docs/superpowers/plans/2026-06-30-backend-j2000-ecliptic-frame-correction.md`. · **Severity:** accuracy correctness + frame consistency (now closed) · **Opened:** 2026-06-30

---

## FU-4: Chart-layer apparent equatorial of date (RA/Dec) sub-project

**Status:** resolved (2026-06-30) · Implemented on `feat/equatorial-declination-output` branch (Tasks 1–6). Chart-layer body positions now carry **apparent equatorial of date** (RA/Dec, true obliquity = mean obliquity + nutation-in-obliquity) for release-grade bodies. Built on the existing `pleiades-apparent` pipeline: equatorial is derived from the final tropical ecliptic position (after apparent-place corrections) via `apparent_equatorial_of_date(ecliptic, true_obliquity) -> EquatorialCoordinates`. Gated by two independent authorities: `validate-equatorial` (JPL Horizons corpus) and `validate-equatorial-se` (Swiss Ephemeris corpus parity). Backend-boundary mean-obliquity equatorial transform strings remain unchanged (backends still emit mean-obliquity equatorial for their own mean rows; the chart layer wraps with true obliquity). · **Build-env note:** the reference tool `tools/se-equatorial-reference` (used to generate the SE equatorial corpus CSV) requires `libclang-dev` + `LIBCLANG_PATH`. This is NOT required to run the `validate-equatorial-se` gate or build the workspace. · **Severity:** feature gap (now closed) · **Opened:** 2026-06-30

---

## Deferred minor findings from feat/equatorial-declination-output tasks

**Status:** mostly resolved (2026-07-01) — two items remain open by design. · Source branch: `feat/equatorial-declination-output` · Resolution: closing-follow-ups plan (`docs/superpowers/plans/2026-07-01-equatorial-branch-followups.md`)

These were cosmetic or non-blocking issues discovered during the B-series (frame correction) and equatorial tasks. Each was explicitly deferred out of scope — do not conflate with bugs.

- **B3 — frame-consistency gate assertion strength:** → **Resolved 2026-07-01:** the test now asserts `rows_validated == 17` exactly (`validate_frame_consistency`). The proposed extra `|Sun-1900 ecliptic latitude| > 40″` GREEN assertion was intentionally NOT added — the Sun@1900 latitude sentinel already runs inside the gate loop and proves the latitude component is genuinely non-trivial, so re-asserting it in the test would be redundant (documented in a comment there).

- **B5 — `PrecessedEcliptic` rustdoc drift:** → **Resolved 2026-07-01:** the module, struct, and field rustdoc were reworded to "caller-selected frame (mean equinox/ecliptic of date or J2000)", no longer unconditionally "of date".

- **Task 1 — `composes_rotation_with_true_obliquity` test tautology:** → **Resolved 2026-07-01:** the test now asserts against independently pinned RA/Dec literals (captured once, hard-coded) instead of recomputing the expected value from the function under test; rotation-direction correctness remains owned by the sibling `solstice_point_maps_to_ra90_dec_obliquity` test.

- **Task 2 — discarded `true_obliquity_degrees` smoke call:** → **Resolved 2026-07-01 (by existing coverage):** `true_obliquity_degrees` is already exercised directly by `true_obliquity_is_mean_plus_delta_eps` and transitively by the equatorial composition / solstice / roundtrip tests, so a dedicated smoke test adds no regression surface.

- **Task 4 — SE gate report epoch-range typo:** → **Resolved 2026-07-01:** the `se-equatorial-reference` end-epoch comment was corrected — `JD_END_TT = 2_488_065.5` is `2099-12-28` (was mislabelled `2099-12-26`); the JD value itself was already correct.

- **Whole-branch review — duplicate ε₀ literals (opportunistic unification):** → **Resolved 2026-07-01 (partial, by design):** the bare `23.439_291_111_111_11` in `pleiades-houses/src/systems/mod.rs` (mean-obliquity lead term) and the `OBLIQUITY_RAD` cache in `pleiades-eclipse/src/geometry.rs` now derive from `pleiades_types::OBLIQUITY_J2000_DEG`. The of-date polynomial at `pleiades-eclipse/src/geometry.rs:399` (`23.439_291 - 0.013_004_2 * t`) was **left untouched by design** — it is a distinct of-date IAU coefficient series, not the J2000 constant, so folding it into `OBLIQUITY_J2000_DEG` would be incorrect.

### Still open (by design)

- **Task 4 — SE ceiling raised for the Moon Moshier outlier:** The SE equatorial gate ceilings remain `4000″` (RA) / `1810″` (Dec) because the Moon's Moshier-vs-DE440 residual peaks ~2643″/1203″ at century edges (all other bodies stay <100″). The ceilings are **global** (apply to every body), not per-body — gross-error detection (~57× the ceiling) is preserved, and sub-arcsec per-body accuracy is the Horizons gate's job. A future ELP/Moshier accuracy improvement could let them tighten. **Remains open.**

- **Whole-branch review — ELP raw backend equatorial is intentionally of-date:** The ELP backend emits a J2000 `ecliptic` (for the Moon and, since issue #57 was fixed on 2026-09-26, for the mean/true node and mean apogee/perigee channels too) but derives its `equatorial` from the raw of-date lon/lat (preserving prior mean-mode values), so a direct ELP consumer who self-converts the J2000 ecliptic with mean obliquity will not reproduce the provided equatorial. Coherent and test-asserted (the ELP batch tests check that forward-precessing the J2000 ecliptic reproduces the equatorial), and overridden by the chart layer for apparent bodies. **Remains open** (documented for any future direct-backend consumer).

**Severity:** cosmetic / defensive hardening · **Opened:** 2026-06-30 · **Largely resolved:** 2026-07-01

---

## FU-5: SP-1 angles & sidereal-time deferred items

**Status:** resolved (2026-07-01) · GMST + equation-of-equinoxes duplicates single-sourced, a southern-hemisphere `validate-angles` gate row added, and a Porphyry high-latitude fallback `asc_mc` consistency test added.

Opened by the `feat/sp1-angles-sidereal` whole-branch review (2026-07-01). SP-1
shipped public sidereal time + the Swiss-Ephemeris `ascmc` chart points
(`AscMc`, `chart_points`/`chart_points_from_armc`, `HouseSnapshot::asc_mc`),
gated by `validate-angles` (armc/gast ~0.16″; geometry points <0.05″ vs SE,
`swehouse.c` ports verified line-by-line). All items below are non-blocking.

- **GMST/equation-of-equinoxes math is duplicated across crates (single-source seam):**
  `pleiades-apparent/src/sidereal.rs` `greenwich_mean_sidereal_time_degrees` is
  byte-identical to the pre-existing `pleiades-time/src/sidereal.rs` `gmst_degrees`,
  and the equation of equinoxes is implemented both in `sidereal.rs` and inline at
  `pleiades-core/src/chart/mod.rs:411`. The crates share no dependency and there is
  no cross-crate test asserting the formulas agree, so a future coefficient edit to
  one could silently diverge. Values are identical today and each is individually
  tested — no current numeric divergence. **Suggested fix:** have `pleiades-apparent`
  delegate to `pleiades-time::gmst_degrees` (or add a cross-crate agreement test over
  a JD sweep), and migrate the `chart/mod.rs` topocentric-LAST path onto the public
  `sidereal_time` during SP-2 (already earmarked as the consolidation point).
  **Related (not a defect):** the public `sidereal_time` consumes the `Instant` JD
  as-supplied (UT1-based, honoring the existing house-layer time policy — a Global
  Constraint), whereas the topocentric path converts TT→UT1 first; a caller passing a
  TT instant sees a ΔT≈69 s ≈ 0.29° offset. Documented in the module header and
  `docs/time-observer-policy.md`. **Correction (2026-09-26, #56):** that offset
  *was* a defect at the house layer — the documented chart path (`from_civil` →
  TT) fed the TT day to sidereal time, so every angle and cusp was ΔT late.
  `pleiades-houses` now converts through `pleiades_apparent::ut1_instant`; the
  `sidereal_time` primitive itself still consumes the JD as supplied. Remaining
  consumers that do not convert are tracked as FU-11.
  → **Resolved 2026-07-01 (`4c79c6c2`, `bd0da1bc`):**
  the GMST polynomial is now single-sourced into `pleiades-time::gmst_degrees_raw`
  (unnormalized), with `pleiades-apparent`'s `greenwich_mean_sidereal_time_degrees`
  delegating to it instead of carrying its own byte-identical copy; a cross-crate
  GMST agreement test guards against re-divergence. The equation of equinoxes is
  now a shared `equation_of_equinoxes(delta_psi_deg, true_obliquity_deg)` helper,
  called by both `pleiades-apparent`'s wrapper and `pleiades-core`'s chart
  topocentric-LAST path, replacing the hand-inlined `cos(ε)` term at
  `chart/mod.rs:411`. Scope note: this closes only the apparent/time GMST duplicate
  and the apparent/core equation-of-equinoxes duplicate that this item targeted — a
  separate, truncated (linear-only) copy of the leading GMST coefficients still
  exists in `crates/pleiades-eclipse/src/geometry.rs` (`sub_shadow_point`); that copy
  was out of this item's scope by design and is left untouched. It is intentionally
  *not* a single-source candidate: it is a deliberately truncated constant+linear
  approximation (it drops the quadratic/cubic terms) paired with a mean-obliquity
  approximation, so delegating it to `gmst_degrees_raw` would change its output —
  it must stay independent.

- **Southern-hemisphere `asc_mc_from` branch is transcribed but unexercised:** the
  `f_pole = -90 - lat` pole-height branch and the vertex western-hemisphere flip in
  `crates/pleiades-houses/src/systems/mod.rs` are exact `swehouse.c` ports but the
  committed angles corpus is northern-only (lat 0/40/55/66), so the strictly-southern
  path has no gate row. **Suggested fix:** add one southern-latitude row to the
  `validate-angles` corpus. Low risk (transcription verified). → **Resolved
  2026-07-01 (`10d71ec7`):** added a lat −33° / lon 20° fixture (`c5_lat33s`) to
  `se-house-reference` and regenerated the houses corpus (cusps/sectors/angles);
  manifest bumped to cusps=138/sectors=6/angles=6. This row exercises the
  `asc_mc_from` `f_pole = -90 - lat` branch under `validate-angles`. Corpus note:
  regeneration also canonicalized the row order of five pre-existing `Horizon` cusp
  rows (identical values, previously appended out-of-band) alongside adding the
  southern rows — no existing row value changed. Build-env note: `tools/se-house-reference`
  needs `LIBCLANG_PATH=/lib/x86_64-linux-gnu` to build and, from a nested git worktree,
  must be built from outside the worktree (cargo resolves the parent workspace root,
  which excludes the tool); the gates never rebuild it — they read the committed CSVs
  via `include_str!`.

- **`asc_mc` consistency test covers only one production site:** the
  `HouseSnapshot`-carries-`AscMc` test exercises the main construction site; the
  high-latitude Porphyry-fallback site is structurally identical and verified by
  inspection but not by an assertion. **Suggested fix:** add a high-latitude test
  hitting the fallback `HouseSnapshot` construction. Trivial. → **Resolved 2026-07-01
  (`5836ea13`):** added a characterization test that forces the Placidus-at-lat-75°
  `SwissEphemerisFallback` early-return branch and asserts the fallback snapshot's
  `asc_mc` equals an independent `asc_mc_from` recomputation.

**Severity:** maintainability / test-coverage hardening (now closed) · **Opened:** 2026-07-01

---

## FU-6: SP-4 `swe_nod_aps` fictitious/small-body coverage bound

**Status:** open (by design) · Opened 2026-07-07 during `feat/sp4-planetary-nodes-apsides` (Task 6/7).

**What:** `EventEngine::nod_aps`'s `Osculating`/`OsculatingBarycentric` methods
are engine-covered for any body the backend chain can supply a state vector
for, including the SP-3 fictitious bodies and packaged asteroids. The
`validate-nod-aps` gate, however, has no committed Swiss-Ephemeris reference
rows for those bodies, so this coverage is exercised by unit/property tests
only, not by cross-checked SE parity.

**Why:** Swiss Ephemeris's own `swe_nod_aps` does not implement fictitious
bodies — the enabling branch for that body class is commented out upstream —
so there is no authoritative SE output to diff against. Separately, offline
backend chains (the packaged artifact, JPL/SPK snapshots) cannot supply the
continuous sub-day state sampling that computing an accurate osculating node/
apsis for a fast-moving small body needs; their fixtures are sparse
regression snapshots, not a continuous ephemeris.

**Impact:** No known correctness defect — `nod_aps` for fictitious/asteroid
bodies is exercised by non-SE tests and, where the backend can't honestly
support it (see FU-7), fails closed with a typed error. This is a gate
*reference* gap, not a behavior gap.

**Suggested fix:** A future SPK-at-runtime backend (continuous ephemeris
sampling) or expanded packaged-asteroid coverage with denser source cadence
could add SE-referenced rows for at least the asteroid subset. Fictitious
bodies remain permanently gate-unreferenced unless a non-SE authoritative
source is adopted for them.

**Severity:** known gap (documented, not blocking) · **Opened:** 2026-07-07

---

## FU-7: Pre-existing asteroid ephemeris-derivative defects surfaced by SP-4

**Status:** open · Opened 2026-07-07, surfaced (not introduced) by
`feat/sp4-planetary-nodes-apsides` Task 6. Pre-existing in
`crates/pleiades-jpl`'s `JplSnapshotBackend` and the packaged asteroid
artifact.

**What:** Two independent, pre-existing defects in asteroid velocity
derivatives, both surfaced because `nod_aps`'s osculating path is the first
consumer to finite-difference these positions for a Kepler-element fit:

1. `JplSnapshotBackend`'s sparse regression fixtures produce non-physical
   finite-difference velocities when the bracketing epochs are widely spaced
   (e.g. Ceres), which can manifest as a spurious `UnboundOrbit` when fit to
   Kepler elements.
2. The packaged artifact's `asteroid:433-Eros` fit has accurate *positions*
   at its sample epochs, but a non-physical time-derivative: its ~180-day
   source sampling cadence undersamples Eros's ~643-day orbit, so any
   consumer that finite-differences the packaged position (not just
   `nod_aps`) gets a garbage velocity.

**Impact:** Both currently manifest as `nod_aps` failing closed with a typed
error for the affected bodies/epochs — correct, safe behavior, not silent
wrong output. But the underlying backends' velocity/derivative output is
dishonest for any other present or future consumer that differentiates
position.

**Suggested fix:** Make `JplSnapshotBackend` and the packaged asteroid
artifact's derivative output honest — either by densifying the source
sampling cadence (Eros) / regression fixture epoch spacing (Ceres), or by
having those backends explicitly decline to serve a derivative/motion
request when the sampling cadence can't support it, rather than returning a
numerically-derived but physically nonsensical value.

**Severity:** accuracy / API-honesty (pre-existing, not SP-4-introduced) ·
**Opened:** 2026-07-07

---

## FU-8: `nod_aps` engine emits NaN (not a typed error) on non-physical r≈0 geometry

**Status:** open · Opened 2026-07-07, surfaced by the SP-4 final whole-branch
review. In `crates/pleiades-events/src/nod_aps.rs`.

**What:** `cartesian_to_raw`'s `(z / r).asin()` is unclamped and `aberrate`'s
`1 / norm(...)` is unguarded, so a geocentric point at r≈0 would yield a NaN
longitude/latitude rather than a typed `EventError`. (This mirrors the existing
unclamped house style in `pleiades-apsides::to_ecliptic`.)

**Impact:** Not reachable for any in-scope body — geocentric distances are
always physical AU-scale — and any NaN that did occur is currently caught
fail-closed at the gate boundary by Tier-1's `is_finite` check. So there is no
wrong output today; the gap is that the *public engine API* itself is not
fail-closed for this hypothetical, relying on the gate as the backstop.

**Suggested fix:** Add a defensive finite/`r > 0` check inside `nod_aps`
(or clamp the `asin` argument and guard the `aberrate` normalization) so the
engine returns a typed `DegenerateNodAps`/`NonFinite`-class error at the source
rather than propagating a NaN.

**Severity:** robustness / API fail-closed hardening (not reachable in scope) ·
**Opened:** 2026-07-07

---

## FU-9: cargo-mutants surviving-mutant triage backlog

**Status:** open · Opened 2026-07-18 by the devkit Phase 3 cargo-mutants slice.

**What:** The first mutation-testing baseline over `pleiades-types`,
`pleiades-time`, and `pleiades-apparent` found 318 surviving mutants out of
1,451 — production logic that can be changed without any test noticing.

**Where:** Full breakdown in
`docs/superpowers/specs/notes/2026-07-18-mutants-baseline.md`; survivors
concentrate in `crates/pleiades-apparent/src/apparent.rs` (49),
`crates/pleiades-apparent/src/nutation.rs` (45),
`crates/pleiades-apparent/src/refraction.rs` (37),
`crates/pleiades-apparent/src/aberration.rs` (28), and
`crates/pleiades-apparent/src/topocentric.rs` (27).

**Evidence:** `mise run mutants` at `5eaeaaadd17d4271f65df9232e2c5ca035499f48`,
cargo-mutants 27.1.0, overall score 77.1% (1070 caught / 1388 viable).
Per-crate: `pleiades-types` 85.0%, `pleiades-time` 84.2%, `pleiades-apparent`
71.6%. Reproduce with `mise run mutants`; per-crate with
`mise run mutants-crate pleiades-apparent` (substitute the crate name).

**Impact:** No known defect. A surviving mutant is a *coverage* signal, not a
bug — it means the test suite does not constrain that line, so a future
regression there would land silently. Highest concern is any survivor in
release-grade numeric paths, where the repo's parity gates are the intended
safety net.

**Suggested fix:** Work the backlog by writing tests that express intent, NOT
assertions that pin whatever the code currently returns — the latter locks in
behavior without validating it and is the failure mode the report-only posture
exists to avoid. Triage in priority order: numeric/logic survivors first.
Caution: the baseline's own assessment found survivors concentrated in
numeric logic (arithmetic-operator swaps in polynomial series evaluation), not
in `Display`/`Debug`/accessors as originally hypothesized, so `#[mutants::skip]`
/ `--skip-calls` exclusion applies only to the small non-numeric tail (e.g. the
few `provenance.rs` survivors) — the numeric bulk must be worked through, not
suppressed, because suppressing it would hide exactly the signal this tier
exists to surface.

**Severity:** test-coverage hardening (report-only, non-blocking) ·
**Opened:** 2026-07-18

**Progress (2026-07-19) — `pleiades-apparent/src/nutation.rs`:** triaged from
`45` → `1` surviving mutant by adding intent-expressing white-box unit tests
(spec/plan:
`docs/superpowers/specs/2026-07-19-fu9-nutation-mutant-triage-design.md`). The
single residual is a documented **equivalent mutant** (`replace || with && in
nutation`): the non-finite guard `!Δψ.is_finite() || !Δε.is_finite()` cannot be
distinguished from its `&&` form by any reachable input, because a non-finite
`jd_tt` poisons the shared fundamental arguments and drives *both* Δψ and Δε
non-finite together — no input makes exactly one non-finite. A function-level
`#[mutants::skip]` would blanket-suppress the whole `nutation` fn's numeric
mutants, so it is intentionally NOT applied; the mutant is left visible and
documented instead. **Reusable method** for the remaining files: regenerate the
per-file survivor list with `cargo mutants -p <crate> --test-tool nextest
--test-workspace=false --file <crate-relative path>`; classify each survivor as
polynomial, series-accumulation, parse/validation, or guard; add a white-box
test asserting against an *independent* reference (published coefficients
evaluated outside the code, or a crafted-input branch), never the code's own
output; re-run `--file` to confirm the residual is 0 or a documented equivalent
mutant. No parity gate was touched; the tier stays report-only. **Remaining
slices** (priority order): `aberration.rs` (28),
`topocentric.rs` (27), `sidereal.rs` (17), `precession.rs`
(17), `lighttime.rs` (5), then the `pleiades-time` and `pleiades-types`
survivors.

**Progress (2026-07-19) — `pleiades-apparent/src/apparent.rs`:** triaged to
`0` surviving mutants (spec/plan:
`docs/superpowers/specs/2026-07-19-fu9-apparent-mutant-triage-design.md`). A
count note: the baseline above lists `apparent.rs` at `49`, but that figure came
from the whole-workspace `mise run mutants` run (default test-tool, and measured
*before* this slice's refactor). Running the reusable method's authoritative
per-file command for the first time — `cargo mutants -p pleiades-apparent
--test-tool nextest --test-workspace=false --file
crates/pleiades-apparent/src/apparent.rs`, against the *post-refactor* file —
measured `10` survivors, which were then driven to `0`. The two numbers reflect
two different invocations, not a regression; the `10 → 0` figure is the
authoritative per-file result. Because `apparent.rs` is an **orchestrator** (it
composes already-tested light-time, precession, nutation Δψ, and annual
aberration into an apparent place with provenance) rather than a polynomial
evaluator, the method adapted in two ways. First, a minimal
**behavior-preserving refactor** (its own separate commit, no runtime-result
change) extracted the two combine primitives `combine_apparent` and
`precession_shift_arcsec` from the three near-identical public functions, so the
combine/scaling/`rem_euclid`/wrap/guard mutant surface is defined and tested
once. Second, the reference strategy is **independent recomposition**: every
expected value comes from crafted inputs or from independently-invoked
sub-correction functions, never from the orchestrator's own output — non-circular
by construction. The relocated, expanded white-box suite covers the combine
primitives directly, both `precession_shift_arcsec` wrap branches (including the
exact `±180°` comparison boundaries), full per-function `ApparentProvenance`
assertions, an end-to-end recomposition-equality check, and fail-closed
non-finite propagation (which surfaces the `"precession"` stage, since precession
rejects a non-finite input longitude before the combine guard is reached).
**No documented residual this slice** — unlike `nutation.rs`'s one equivalent
mutant, `apparent.rs` reached a genuine `0`. The design's residual candidate,
`DEFAULT_MAX_ITERATIONS`, produces no mutant at all (cargo-mutants does not mutate
a bare `pub const`), so there was nothing to suppress or document. No parity gate
was touched; the tier stays report-only; `mise run ci` is green. **Remaining
slices** (priority order): `aberration.rs` (28),
`topocentric.rs` (27), `sidereal.rs` (17), `precession.rs` (17), `lighttime.rs`
(5), then the `pleiades-time` and `pleiades-types` survivors.

**Progress (2026-07-20) — `pleiades-apparent/src/refraction.rs`:** triaged from
`37` → `3` documented equivalent mutants (spec/plan:
`docs/superpowers/specs/2026-07-20-fu9-refraction-mutant-triage-design.md`).
Baseline confirmed by the authoritative per-file command (`101 mutants tested,
37 missed, 64 caught`). This slice was **tests-only** — no refactor was needed,
unlike `apparent.rs`: the file was already decomposed into small pure functions
at exactly the right seams, so the only source edit was relocating the inline
test module to `src/refraction/tests.rs` per AGENTS.md. The dominant finding was
a plain **coverage hole** rather than tolerance masking: `true_from_apparent_below_horizon`
had no test at all (the committed SE corpus exercises only the
`apparent_from_true` direction), accounting for 21 of the 37 survivors including
all three whole-function replacements. The remainder split into blend-region gaps
(the corpus reaches only `h <= -9.96`, where the fade contributes ~9″ under a 15″
tolerance, leaving the `h ∈ [-1, 0)` branch and the fade slope unconstrained) and
loose-tolerance formula survivors (`scale -> 1.0` hides because the default
atmosphere's scale is 0.9858 ≈ 1.0). Reference strategy: **crafted-exact
atmospheres** — `(1010 mbar, 10 °C)` makes `scale` exactly `1.0` and
`(2020 mbar, 25 °C)` makes both factors non-unit and distinct — combined with
Bennett/Saemundsson literals evaluated outside the code from the published
formulas, and fade midpoints chosen so `fade` is an exact binary fraction
(`h = -5.5` → `fade = 0.5`). The blend model is repo-invented (SE's own
below-horizon model is discontinuous and deliberately not reproduced), so its
authority is its own documented spec: anchor = R(-1), linear fade to zero at -10.
**Documented residual — 3 equivalent mutants**, left visible rather than
`#[mutants::skip]`-suppressed: `saemundsson`'s `scale * 1.0` → `/ 1.0`
(bit-identical), and `< → <=` in both public dispatchers, which differ only at
exactly `h == 0.0` where both branches evaluate the identical expression. No
parity gate was touched; the tier stays report-only; `mise run ci` is green.
**Remaining slices** (priority order): `aberration.rs` (28), `topocentric.rs`
(27), `sidereal.rs` (17), `precession.rs` (17), `lighttime.rs` (5), then the
`pleiades-time` and `pleiades-types` survivors.

**Progress (2026-07-20) — `pleiades-apparent/src/aberration.rs`:** triaged from
`28` → `0` surviving mutants (spec/plan:
`docs/superpowers/specs/2026-07-20-fu9-aberration-mutant-triage-design.md`).
Baseline confirmed by the authoritative per-file command (`56 mutants tested,
28 missed, 27 caught, 1 unviable`). The distinguishing finding of this slice is
that **11 of the 28 survivors were arithmetically unreachable through the
public API**: the Earth-orbit elements `e` and `ϖ` enter the output only via the
~0.34″ `e κ cos(ϖ - λ)` term, so mutating their polynomial coefficients moves
Δλ by only ~0.001″ (`e`) to ~0.006″ (`ϖ`) — below any tolerance the model's own
accuracy justifies. Killing them without pinning the function's own output
therefore required a testability seam, so a minimal **behavior-preserving
refactor** (its own commit, no runtime-result change) extracted
`earth_orbit_elements(t) -> (e, pi_deg)`; the polynomials are now asserted
directly against Meeus 25.4 coefficients evaluated outside the code at
`t = 0, +1, -1` (the `±1` pair is what separates the linear term, which flips
sign, from the quadratic, which does not). `julian_centuries` needed no refactor
— it was already a seam with no test, and every prior test passed the J2000
epoch, the one input where `t = 0` is indistinguishable from the `-> 0.0`
whole-function mutant; a single half-century epoch (`2469807.5` → `t = 0.5`)
kills all five. The remaining 12 formula-line survivors were genuine coverage
holes — notably every prior Δβ assertion used `.abs()` against a bound, leaving
the sign free — and fall to one **crafted discriminating geometry**
(`λ = 30°, β = 60°, ⊙ = 120°`, `t = 0`) which makes `cos β = 0.5` (so
`/cos_beta` and `*cos_beta` differ 4×) while avoiding two degeneracies that a
more obvious choice would hit: `λ = ϖ` zeroes `sin(ϖ - λ)` and lets the
bracket-minus mutant survive bit-identically, and `λ = 0` makes `ϖ + λ ≡ ϖ - λ`.
Both rejected geometries are recorded in the design so they are not
re-proposed. **No documented residual this slice** — like `apparent.rs`, and
unlike `nutation.rs` (1) and `refraction.rs` (3), `aberration.rs` reached a
genuine `0`, so nothing was suppressed or excused. No parity gate was touched;
the tier stays report-only; `mise run ci` is green. **Remaining slices**
(priority order): `topocentric.rs` (27), `sidereal.rs` (17), `precession.rs`
(17), `lighttime.rs` (5), then the `pleiades-time` and `pleiades-types`
survivors.

**Progress (2026-07-20) — `pleiades-apparent/src/topocentric.rs`:** triaged from
`27` → `3` documented equivalent mutants (spec/plan:
`docs/superpowers/specs/2026-07-20-fu9-topocentric-mutant-triage-design.md`).
Baseline confirmed by the authoritative per-file command (`82 mutants tested,
27 missed, 54 caught, 1 unviable`) — the first slice where the per-file and
whole-workspace figures agree exactly. **Tests-only** like `refraction.rs`: the
only source edit was relocating the inline test module to
`src/topocentric/tests.rs` per AGENTS.md. The dominant root cause was
**sign-free and degenerate assertions**: every parallax assertion used `hypot`
(no sign), the diurnal-aberration bound (`< 0.36″`) constrained no term, and —
decisively — the existing test observer (equator, sea level) makes
`ρcosφ′ = 1.0` exactly, so the `* rho_cos_phi_prime → /` mutants were
**bit-identical** and unkillable from those tests. Reference strategy:
**independent recomposition** — a Python reimplementation of the published
Meeus ch. 11/40 pipeline (script reproduced in the plan doc), cross-validated
against the crate at ~1e-11″, pins exact literals at one discriminating
geometry (Palomar `ρcosφ′ = 0.836`, `dec_topo ≈ 27.9°`, `H ≈ 328.2°` — 17
kills including all four provenance fields) plus two wrap-crossing geometries
(body at λ = 0.02°/359.98°, Moon-scale parallax carrying the topocentric
longitude across the 0°/360° seam — 6 kills). Rejected geometries recorded in
the spec so they are not re-proposed: equator/sea-level observer
(`ρcosφ′ = 1`), and `β ≈ 0` for the primary geometry (`cos δ = 1`,
`sin δ = 0` degeneracies). **Documented residual — 3 equivalent mutants**,
left visible rather than `#[mutants::skip]`-suppressed: `||`→`&&` in the
output non-finite guard (the `nutation.rs` shape — the guard returns the
byte-identical error regardless of which operand triggers it, since
`to_ecliptic` mixes RA and Dec into both outputs and any non-finite value
poisons both together, so no reachable input distinguishes the operators),
and `>`→`>=` / `<`→`<=` in the Δlon wrap comparisons (they differ only at a
raw Δlon of exactly ±180.0°, unreachable for physical inputs since the
topocentric shift is bounded ≪ 2° beyond the observer's geocentric radius).
A fourth candidate — `||`→`&&` in the *input* non-finite guard
(`!topo_distance.is_finite() || topo_distance <= 0.0`) — was originally
classified equivalent alongside the output guard, but the final whole-branch
review found it killable: a finite `distance_au` as large as `1e301`
overflows the squared-norm sum to `+inf`, and under the `&&` mutant
`inf <= 0.0` is false, so the guard fails to fire and every downstream value
stays finite (`tz / inf == 0.0`), producing `Ok(Some(inf))` instead of the
expected `Err`. Per the spec's own killed-instead-of-documented rule, it is
killed by an added overflow fail-closed test rather than documented as
equivalent. No parity gate was touched; the tier stays report-only;
`mise run ci` is green. **Remaining slices** (priority order): `sidereal.rs`
(17), `precession.rs` (17), `lighttime.rs` (5), then the `pleiades-time` and
`pleiades-types` survivors.

**Residual audit gap (future slice):** the independence discipline behind
this slice's reference script covers *formulas*, not *constants* — the
script inherits the crate's own `DIURNAL_ABERRATION_ARCSEC = 0.3192` (whose
doc comment, "0.0213 s × 15", actually works out to 0.3195; a first-principles
derivation gives ≈0.3200) and `AU_IN_EARTH_RADII = 23454.779` (the IAU-1976
value, not the WGS84 value named beside it; the true WGS84-consistent ratio
is ≈23454.791) rather than re-deriving them independently. Both discrepancies
are ≲0.001″, negligible for this slice's purpose, and production code is
untouched by this note — flagged here only so a future slice can decide
whether to re-derive the constants independently.

**Progress (2026-07-20) — sidereal (`pleiades-apparent/src/sidereal.rs` +
`pleiades-time/src/sidereal.rs`):** triaged from `17 + 5` → `0` surviving
mutants (spec/plan:
`docs/superpowers/specs/2026-07-20-fu9-sidereal-mutant-triage-design.md`).
The first **two-file slice**: after FU-5's single-sourcing, the apparent-side
file is a thin composition layer delegating the GMST polynomial to
`pleiades-time`, so the `pleiades-time` sidereal survivors (queued in the
backlog tail) were folded in rather than re-deriving the same references
later. Both baselines confirmed by the authoritative per-file commands
(apparent: `46 tested, 17 missed, 28 caught, 1 unviable`; time: `30 tested,
5 missed, 25 caught`). **Tests-only** like `refraction.rs`/`topocentric.rs`:
the only source edits were relocating both inline test modules to
`src/sidereal/tests.rs`. Root causes: on the time side, all 5 survivors were
the small quadratic/cubic Meeus 12.4 terms invisible to a single-epoch 1e-4°
test — killed by literals evaluated outside the code at **t = ±4** Julian
centuries (JD 2597645.0 / 2305445.0, ≈ years 2400/1600, inside the project's
coverage target) at 2e-7° tolerance, with design-stage verified margins
(smallest mutant displacement 111 ulp of the raw value vs a ~27 ulp
tolerance); the ± pair separates the even quadratic term from the odd cubic
term (the aberration slice's ±1 trick at larger |t|). On the apparent side,
15 survivors were three entirely untested hours accessors and 2 were the
`gmst + lon` composition in `local_mean_deg` (the one field no test
constrained by value) — all 17 killed by a single recomposition-pinning test
at `(jd = 2446895.5, lon = +52.5°)` asserting every `_deg` field and every
hours accessor against expectations rebuilt from independently-invoked
sub-functions, plus a Meeus example 12.b GAST anchor (197.692230°, 1e-4°)
tying the composed output to a published value. **No documented residual
this slice** — like `apparent.rs` and `aberration.rs`, a genuine `0 + 0`;
no equivalent-mutant candidates surfaced. No parity gate was touched; the
tier stays report-only; `mise run ci` is green. **Remaining slices**
(priority order): `precession.rs` (17), `lighttime.rs` (5), then the
remaining `pleiades-time` (non-sidereal) and `pleiades-types` survivors.

**Progress (2026-07-21) — precession + lighttime
(`pleiades-apparent/src/precession.rs` + `lighttime.rs`):** triaged from
`17` → `2` documented equivalent mutants and `5` → `0` (spec/plan:
`docs/superpowers/specs/2026-07-21-fu9-precession-lighttime-mutant-triage-design.md`).
The second two-file slice, closing out `pleiades-apparent` entirely. Both
baselines confirmed by the authoritative per-file commands (precession:
`238 tested, 17 missed, 219 caught, 2 unviable`; lighttime: `14 tested,
5 missed, 8 caught, 1 unviable`), both matching the whole-workspace figures
exactly. **Tests-only** like refraction/topocentric/sidereal: the only
source edits were relocating both inline test modules to
`src/precession/tests.rs` and `src/lighttime/tests.rs`. Root causes: all 15
precession polynomial survivors (`*`→`/` on the quadratic/cubic ζ/z/θ
terms) sit in the **inverse** function, whose only test was the 1900
round-trip at t ≈ −1 — where `t*t ≈ t/t` displaces the output by ~1e-8°,
under the 1e-6° tolerance — while the forward twin's identical mutants die
at t = 0 (`/t` → NaN) in the forward-only identity test; lighttime's 5
survived because every query closure ignored the instant it was given, so
no test could observe the retarded epoch, plus two never-hit exact
comparison boundaries. Kills: pinned literals for the inverse at t = ±4
(independent Python implementation of the published Meeus 20.3/21.4/13.x
pipeline, cross-validated against the genuinely different Meeus 21.5/21.7
direct-ecliptic formulation to ~3e-3″; smallest mutant displacement 0.961″
vs a 1e-9° tolerance, ~2.7e5× margin), an inverse identity test (closing a
real intent gap), an instant-dependent 1000 °/day query pinning the
retarded epoch (mutant margins 28.9°/57.8°/112.8°), and crafted-exact f64
boundary distances landing the light-time exactly on the 10-day cap and
the 5e-7-day convergence threshold (both representability-checked, with
in-test precondition asserts). A fail-closed overflow test at
jd_tt = 7.0e107 (the window where θ's cubic term alone overflows) records
why the residual exists. **Documented residual — 2 equivalent mutants**,
left visible rather than `#[mutants::skip]`-suppressed: `||`→`&&` in both
output non-finite guards — the `nutation.rs`/`topocentric.rs` shape,
checked against the overflow lens rather than by analogy: every non-finite
route (NaN inputs, or finite-huge `jd_tt` overflowing θ first) flows
through shared variables (t, ζ/z/θ, α/δ, ε) that poison both outputs
together, the outputs themselves cannot overflow (bounded `atan2`, clamped
`asin`), so no reachable input makes exactly one output non-finite. No
parity gate was touched; the tier stays report-only; `mise run ci` is
green. **Remaining slices** (priority order): `pleiades-time` non-sidereal
(`convert.rs` 16, `deltat.rs` 10, `tdb.rs` 9), then `pleiades-types`
(`zodiac.rs` 12, `time.rs` 10, and the small tail).

**Progress (2026-07-21) — `pleiades-time` non-sidereal (`calendar.rs` +
`deltat.rs` + `tdb.rs` + `convert.rs`):** triaged from `9 + 10 + 9 + 16`
→ `0` surviving mutants (spec/plan:
`docs/superpowers/specs/2026-07-21-fu9-time-mutant-triage-design.md`),
closing out `pleiades-time` entirely. Scope note: the previous entries'
remaining-slices line listed only `convert.rs`/`deltat.rs`/`tdb.rs` —
`calendar.rs` (9 survivors in the baseline notes) was omitted by
transcription oversight; this slice covers it. The first four-file
slice, and **tests-only** like refraction/topocentric/sidereal/
precession: the only source edits were relocating the four inline test
modules to `src/<module>/tests.rs`. All four baselines confirmed by the
authoritative per-file commands (calendar: `130 tested, 9 missed, 120
caught, 1 unviable`; deltat: `65 tested, 10 missed, 52 caught, 3
unviable`; tdb: `17 tested, 9 missed, 8 caught`; convert: `47 tested,
16 missed, 23 caught, 8 unviable`), all matching the whole-workspace
figures exactly. Root causes: diagnostics never string-asserted and JD
values never pinned on either ΔT path (`convert.rs`); a `dt > 69.0`
bound as the only extrapolation test (`deltat.rs`); a magnitude-bound
test that can never kill a phase mutant because the USNO term is <2 ms
by construction for any g (`tdb.rs`); and coincidence-degenerate test
dates — at 1987, `floor(alpha/4) == alpha % 4` — plus no January
(e=14), February (e=15), or negative/≥61-second coverage
(`calendar.rs`, where the surviving `||`→`&&` re-associates by
precedence to `A || (B && C)`, verified by hand-applying the mutant:
it silently accepts `second = -1.0` and `61.5`). Kills: pinned literals
from a Python mirror of the published formulas (Espenak–Meeus at
exactly t = 80 via representable JD 2480765.0, margin ~2.6e10×; USNO
two-term at two epochs, min displacement 2.04e-6 s vs 1e-9 s
tolerance), hand-interpolated ΔT table references, a signed TDB−TT
assertion near the annual peak (the topocentric slice's sign-free
`.abs()` lesson), exact leap-epoch boundary acceptance, full six-field
`from_julian_day` literals at 2100-01-01 (alpha=16, e=14) and
2000-02-29 12:00 (e=15, month==2), and direct white-box fail-closed
tests of the `finite` guard (unreachable via the bounded public API —
overflow lens checked — so tested at the seam, per the `apparent.rs`
private-primitive precedent). **No documented residual this slice** —
a genuine `0` across all four files; no equivalent-mutant candidates
surfaced at design time (like sidereal). No parity gate was touched;
the tier stays report-only; `mise run ci` is green. **Remaining
slices:** `pleiades-types` only (`zodiac.rs` 12, `time.rs` 10,
`time_range.rs` 4, and the small tail) — the final slice of the FU-9
baseline.

**Progress (2026-07-21) — `pleiades-types` (10 files) + `pleiades-apparent/src/provenance.rs`
— FU-9 measured baseline COMPLETE:** triaged from `41` → `0`
(`pleiades-types`) and `3` → `0` (`provenance.rs`) surviving mutants
(spec/plan:
`docs/superpowers/specs/2026-07-21-fu9-types-mutant-triage-design.md`), the
ninth and **final** slice. Scope note: `provenance.rs` (3 survivors) was
recorded in the 2026-07-18 baseline notes but omitted from every prior
remaining-slices line — the prior `pleiades-apparent` slices (through
precession+lighttime, whose entry above calls the crate "closed out entirely")
closed every *numeric* survivor file but not this non-numeric diagnostic tail;
that "entirely" referred to the crate's numeric survivor count, and
`provenance.rs` is its last piece, folded in here the way the sidereal slice
folded in `pleiades-time/src/sidereal.rs`, so the crate's "entirely" is now
literally true and the entire measured baseline reaches a terminal state in one
pass. The first slice
dominated by **enum plumbing** rather than numeric logic (`Display` impls,
match-arm dispatch, validation guards, one conversion) plus a single polynomial
(`Instant::mean_obliquity`). **Tests-only** like refraction/topocentric/
sidereal/precession/time: the only source edits were relocating the monolithic
`pleiades-types/src/tests.rs` (1,464 lines, 69 tests) into a per-source-module
`src/tests/` directory per AGENTS.md, and relocating `provenance.rs`'s inline
test module to `src/provenance/tests.rs`. All eleven baselines confirmed by the
authoritative per-file commands, all matching the whole-workspace figures;
whole-crate re-check `cargo mutants -p pleiades-types --test-tool nextest
--test-workspace=false` reports `0 missed` (was `311 tested, 41 missed`), and
`provenance.rs` `0 missed` (was `6 tested, 3 missed`). Per-file: `zodiac.rs`
12→0, `time.rs` 10→0, `time_range.rs` 4→0, `coordinates.rs` 3→0, `ayanamsa.rs`
3→0, `angles.rs` 3→0, `house_systems.rs` 2→0, `motion.rs` 2→0, `observer.rs`
1→0, `frames.rs` 1→0, `provenance.rs` 3→0. Root causes: `Display`/`name`/
`summary_line` renderings never string-asserted (11 mutants — release-facing
diagnostics that could silently empty or drift); nine `from_longitude` match
arms unreached because the only test checked the 0°/30° boundary, killed by a
mid-band longitude per sign plus a wraparound case (`780°`→Gemini pins the
`floor(deg/30) % 12` reduction); the ten `mean_obliquity` cubic
operator-swaps invisible at the sole J2000 (t = 0) test where every t/t²/t³
term vanishes, killed by two off-epochs at t = ±1 (JD 2488070.0 / 2415020.0,
`jd − 2451545` exactly ±36525.0 so t is exactly ±1.0) pinned to the published
IAU-1976 cubic evaluated outside the code at 1e-12° (true-minimum mutant
displacement ~1.64e-7°, a ~1.6e5× margin); and reachable-boundary/inverted
validation guards plus enum-vs-struct dispatch gaps (the existing
`validate_against_reserved_labels` tests called the *struct* method, never the
`Ayanamsa`/`HouseSystem` enum's `Self::Custom` arm). The `coordinates.rs:216`
`validate_finite_coordinate_value → Ok(())` mutant is reachable through the
public constructor (a NaN longitude survives `rem_euclid` normalization), so it
is **killed, not documented** — the overflow-lens exception of prior slices
does not apply (the input itself is non-finite). **No documented residual this
slice** — a genuine `0` across all eleven files; no equivalent-mutant candidate
surfaced (like `apparent.rs`/`aberration.rs`/sidereal/time). No parity gate was
touched; the tier stays report-only; `mise run ci` is green.

**FU-9 measured baseline CLOSED.** Every file in the 2026-07-18 three-crate
measurement (`pleiades-types`, `pleiades-time`, `pleiades-apparent`) now reaches
`0` surviving mutants or a documented equivalent. Nine slices; **total
documented-equivalent tally = 9**: `nutation.rs` 1, `refraction.rs` 3,
`topocentric.rs` 3, `precession.rs` 2 — every one a guard `||`↔`&&` on a shared
poisoned variable or an unreachable exact comparison boundary, each left visible
with a reachability argument rather than `#[mutants::skip]`-suppressed;
`apparent.rs`, `aberration.rs`, sidereal (both files), `lighttime.rs`,
`pleiades-time` (all files), `pleiades-types` (all files), and `provenance.rs`
all reached a genuine `0`. FU-9 stays **open only as a standing posture entry**: there are no
remaining slices for the original three-crate baseline, but the report-only
mutants tier remains, so any future `mise run mutants` expansion to `pleiades-*`
domain/backend crates outside the original three would open new slices under
this follow-up (new work, not part of the closed baseline).

**Progress (2026-07-22) — houses Foundation
(`pleiades-houses/src/systems/mod.rs`, shared primitives):** first PR of the
post-baseline `pleiades-houses` expansion campaign (spec:
`docs/superpowers/specs/2026-07-22-fu9-houses-mutant-triage-design.md`). The
whole-crate baseline measured `1,231 mutants, 569 missed` — `systems/mod.rs`
alone has `554`, ~15× the previous largest slice, so the crate is worked as a
~6-PR family-grouped campaign. This Foundation PR triaged the shared geometry
primitives + chart-point set + trivial/Porphyry family from `113` surviving
mutants to **13 documented equivalents** (measured; an intermediate revision of
this note said `19`, before the final review found 6 of those were actually
killable — see the correction below): `spherical_cotrans` (34),
`asc1`/`asc2` (28), `asc_mc_from` (22), `porphyry_houses` (16),
`interpolate_longitude` (6), `signed_longitude_difference` (3), and one each in
`right_ascension_from_ecliptic_longitude`, `whole_sign_houses`,
`longitude_in_arc`. **Tests-only** — every expected value comes from an
independent from-scratch port of the published swehouse.c Asc1/Asc2 +
`swe_houses_armc` point set (`docs/superpowers/specs/notes/2026-07-22-houses-reference.py`),
cross-validated against the crate to 1e-12 before its literals were trusted.
Killing the shared primitives once removes their survivors from every composing
system, which the later family PRs build on.

The plan predicted a `113 → 1` residual, but mutation verification measured
`26` survivors: `7` were real coverage gaps the crafted normal-path geometries
never reached, and `19` were *initially* classified as genuine equivalents. The
`7` were killed by two degenerate-axis `asc2` pins (x on the `sinx ≈ 0` axis,
reaching the `sinx.abs() < 1e-12` branch) and one `asc_mc_from` geometry where
the vertex flip actually fires (`vemc > 0`), which the plan's three geometries
never triggered.

**Correction (final whole-branch review, 2026-07-23):** 6 of those 19
"equivalents" were misclassified — the equivalence sweep never sampled
`lat = 0`, where the pole height is exactly `±90°` and `tan` is **not**
180-periodic in f64 (`tan(90°) = +1.633123935319537e16` vs
`tan(-90°) = -1.633123935319537e16`), and `asc2`'s `1e-12` guard at
`value.abs() < 1e-12` **assigns** `value = 0.0`, making the downstream
`value < 0.0` comparison reachable at equality for a real observer geometry
(pole `= 90 - obliquity`). All 6 are now killed by two new tests
(`asc_mc_from_equator_pole_asymmetry_kills_tan_periodicity_mutants` and
`asc2_value_zero_guard_reachable_at_exact_equality`): the `lat >= 0 → lat < 0`
branch swap and the `delete -` turning `-90 - lat` into `90 - lat`
(mod.rs 192, 195 — the "180-periodicity of tan" claim), the `vemc == 180` /
`vemc == 0` exact-equality boundaries (mod.rs 204, 207), and `asc2`'s
`value < 0.0 → <= 0.0` / `value.abs() < 1e-12 → == 1e-12` guard mutants
(mod.rs 1812, 1807). Some of the surviving prose also overstated bit-exactness
where the true behavior is only *below the 1e-9 parity tolerance*
(e.g. `armc ± 180` and `(x ± 180) mod 360` differ by up to `~6.25e-13` /
`~5.68e-14` respectively, not exactly 0 — the `armc ± 180` figure was later
re-measured at `~1.31e-12` on a finer sweep; see the corrected per-bucket
breakdown below); that prose was rewritten to state the measured magnitude
instead of claiming exactness. **Documented residual —
13 equivalent mutants** (down from 19), all left visible (no
`#[mutants::skip]`), enumerated in `asc_geometry_equivalent_mutants_are_documented`
with a per-mutant reachability argument grouped by structural reason:

- **Structurally unreachable / bit-identical** (no floating-point
  approximation involved — no representable input can distinguish the
  operators, independent of tolerance): `asc2` 1818's `< → ==`, `< → >` and
  `< → <=` variants paired with 1819 `delete -` — this `else if value == 0.0`
  arm is reached only when `sinx.abs() >= 1e-12` (the 1811 guard consumed the
  small-`sinx` case), so `sinx` is never `0` here; the arm is reached because
  the 1807 guard *assigned* `value = 0.0`. However these four steer the
  `sinx < 0.0` test, the result is `±90.0`, and the 1826 fold maps
  `-90.0 + 180.0` to exactly `90.0`, so all four return a bit-identical
  `90.0`. (An earlier revision of this note stated the inverted premise "the
  1811 guard already forced `sinx == 0`" — the conclusion held, the reason did
  not.) Plus `asc2` 1826 `< → <=` (`longitude == 0.0` is unreachable from all
  three producing branches). [5]
- **Sub-tolerance, but measurably NOT bit-identical (I2 correction).** Every
  magnitude below is a **sweep maximum, not a proven bound**: `asc1`
  `delete match arm 3` — arm 3 and the `_` arm are *algebraically* identical
  but not f64-identical, since `(180-u)·π/180` and `π - u·π/180` differ in the
  last bits (measured max diff `~5.68e-14` at `x1 ≈ 180.315`, pole `-52`; this
  was previously mis-filed as bit-identical); `asc1` arm-3
  `x1 - 180 → x1 + 180` (measured max diff `~2.56e-13`); `asc_mc_from`
  `armc - 180 → + 180` at both call sites 201 and 215 (measured max circular
  diff `~1.31e-12`); the vertex flip `vertex + 180 → vertex - 180` at 208 and
  `longitude_opposite`'s `+ → -` at 1833 (measured max circular diff
  `~5.68e-14`). None of these differences are exactly 0 — each is simply far
  below the crate's 1e-9 parity tolerance, which is why no 1e-9 white-box pin
  can distinguish the mutated operators. [6]
- **`asc2`'s remaining `1e-12` guard thresholds, below tolerance under
  generic inputs**: `value.abs() < 1e-12 → <=` (1807, the `<=` variant —
  distinct from the `== 1e-12` variant killed above) and
  `sinx.abs() < 1e-12 → <=` (1811) — for non-adversarial inputs the reachable
  boundary difference is `~1.4e-10`, below the 1e-9 tolerance. This is a
  below-tolerance claim, not the "no representable input hits equality"
  claim the earlier writeup made — that stronger claim is exactly what was
  wrong for the two guard mutants killed above. **These two are therefore
  best read as "not proven equivalent, not currently killable"**: an
  adversarial input sitting exactly on the threshold could exceed the
  tolerance, so a later campaign PR may yet kill them. [2]

These bring the **running documented-equivalent tally to `9 + 13 = 22`**,
superseding the earlier `9 + 19 = 28` figure, which counted 6 killable mutants
as equivalent. The `13` is **measured, not predicted**: the authoritative
scoped run (`cargo mutants -p pleiades-houses --test-tool nextest
--test-workspace=false --file crates/pleiades-houses/src/systems/mod.rs -F
'in (…Foundation functions…)$'`, 164 mutants) reports **`13 missed / 151
caught / 0 unviable`**, and `mutants.out/missed.txt` matches the three buckets
above line-for-line. No parity gate was touched;
the tier stays report-only; `mise run ci` is green. **Remaining houses PRs:**
great-circle (`apc_sector`/`krusinski`/`horizon`), sector
(`pullen_sr`/`pullen_sd`/`albategnius`/`gauquelin`), sunshine/solar-arc,
quadrant/projection, then catalog + thresholds (which adds `-p pleiades-houses`
to `[tasks.mutants]`).

**Progress (2026-07-23) — houses Great-circle
(`pleiades-houses/src/systems/mod.rs`, `apc_sector`/`apc_houses`/`horizon_houses`/
`krusinski_pisa_goelzer_houses`):** second PR of the post-baseline
`pleiades-houses` expansion campaign (spec:
`docs/superpowers/specs/2026-07-22-fu9-houses-mutant-triage-design.md`; plan:
`docs/superpowers/plans/2026-07-23-fu9-houses-greatcircle-mutant-triage.md`).
Triaged the Great-circle family from `90` surviving mutants (`apc_sector` 58,
`krusinski` 19, `horizon_houses` 12, `apc_houses` 1 — matching the design's
whole-crate prediction exactly) to **8 documented equivalents**, all in
`horizon_houses`'s pole-singularity clamp. **Tests-only.** Two reference
strategies keyed to survivor structure: `apc_sector` is a pure function, so its
58 arith survivors fell to a single **independent-port** pin of all 12 sector
outputs at one non-degenerate geometry (lat=52°, obl=23.4366°, sidereal=45° —
measured 59/59 caught); the other three take `Instant`/`ObserverLocation` and
call `local_sidereal_time` (GMST+nutation, not reproduced), so their
**structural** survivors (sector index, hemisphere sign, rotation-angle signs,
offset arithmetic — the inner trig was already gate-killed) fell to
**independent recomposition** (the `apparent.rs` precedent): the test threads
`st` from the un-mutated `local_sidereal_time` and recomposes expected cusps
from the published SE formula plus Foundation-pinned primitives (`ascendant_for`,
`longitude_opposite`, `spherical_cotrans`, `ecliptic_longitude_from_ra`,
`signed_longitude_difference`), then asserts equality with the crate. The
`apc_sector` port extends the shared `houses-reference.py`. Per the Foundation
lesson (probe extremes before documenting equivalence), four horizon survivors
first classified as pole-clamp equivalents were **killed** by extreme
geometries: the `-90 - lat` hemisphere sign by a southern observer, the `/90`
clamp-guard variant at the pole (`lat=90`, where it clamps and HEAD does not),
and both N-side clamp-target mutants by a near-equator northern observer
(`lat=1e-11`, where `cosfi` flips sign); krusinski's `< -> <=` flip-boundary
mutant was killed by an `asc == mc` (signed_diff == 0) geometry. **Documented
residual — 8 equivalent mutants**, all `horizon_houses`, left visible (no
`#[mutants::skip]`) and enumerated with per-mutant reachability arguments in
`horizon_pole_singularity_equivalent_mutants_are_documented`, grouped:
(A) `1082:69` `+180 -> -180` — `(LST±180).rem_euclid(360)` differ by at most
`~5.68e-14°` (measured over a fine sweep; the Foundation `armc±180` shape),
far below the 1e-9° tolerance; (B) `1094:36` `-` -> `+` (never-clamp),
`1094:50` `<` -> `==`/`<=` (measure-zero clamp-fire boundary), and `1095:56`
`<` -> `<=` (unreachable `tl==0` inside the clamp branch) — all four reachable
only where the clamp effect is itself sub-tolerance or at a measure-zero
equality; (C) `1108:33` `>` -> `==`/`<`/`>=` (×3) — the `if cosfi == 0.0` arm is
structurally dead (`cos(tl_rad)` is never exactly `0.0`; min `|cos|` over the
reachable range is `6.123e-17`). This brings the **running documented-equivalent
tally to `22 + 8 = 30`**. The `8` is **measured, not predicted**: the
authoritative scoped run (`-F 'in (apc_sector|apc_houses|
krusinski_pisa_goelzer_houses|horizon_houses)$'`, 131 mutants) reports
`8 missed / 123 caught / 0 unviable`. No parity gate was touched; the tier stays
report-only; `mise run ci` is green. **Remaining houses PRs:** sector
(`pullen_sr`/`pullen_sd`/`albategnius`/`gauquelin`), sunshine/solar-arc,
quadrant/projection, then catalog + thresholds (which adds `-p pleiades-houses`
to `[tasks.mutants]`).

**Progress (2026-07-24) — houses Sector
(`pleiades-houses/src/systems/mod.rs`, `pullen_sr_houses`/`pullen_sd_houses`/
`albategnius_houses`/`solve_gauquelin_sector`/`gauquelin_houses`):** third PR of
the post-baseline `pleiades-houses` expansion campaign (spec:
`docs/superpowers/specs/2026-07-22-fu9-houses-mutant-triage-design.md`; plan:
`docs/superpowers/plans/2026-07-24-fu9-houses-sector-mutant-triage.md`). Triaged
the Sector family from `161` surviving mutants (`pullen_sr` 73, `pullen_sd` 42,
`albategnius` 42, `solve_gauquelin_sector` 4, `gauquelin_houses` 0 — matching the
design's whole-crate prediction 73/42/42/4 exactly, and confirming
`gauquelin_houses` is already fully caught by the parity gates) to **6 documented
equivalents**. **Tests-only.** The three pure systems (`pullen_sd`/`albategnius`
byte-identical, `pullen_sr`) are pure functions of `(asc, mc)` — no
`local_sidereal_time`, unlike the Great-circle family — so their 157 arith/
comparison survivors fell to **independent-port** pins of all 12 cusps at a small
set of discriminating geometries (extending the shared `houses-reference.py` with
`pullen_sd` + `pullen_sr`, cross-validated to ~1e-12). `pullen_sr`'s ratio `r` was
re-derived non-circularly as the positive root of `r^4 + 2r^3 - 2c*r - c = 0`
(`c=(180-q)/q`, from the published SR symmetric-arc property) solved by bisection+
Newton — a different method than the crate's Ferrari closed form. The 4
`solve_gauquelin_sector` survivors are all guard/convergence boundaries: the
`|| -> &&` fail-closed guard was **killed** by a crafted non-convergence-but-finite
geometry (lat=80, fraction=1/9 — HEAD returns `Err`, the `&&` mutant `Ok`), the
lighttime solver-boundary precedent. **Documented residual — 6 equivalent mutants**,
left visible (no `#[mutants::skip]`) and enumerated with per-mutant reachability
arguments in `sector_equivalent_mutants_are_documented`: `pullen_sr` `1437 > -> >=`
(q=90 maps to itself under both operators), `1458 > -> >=` (at acmc=90, r=1 exactly
so the two placement branches are bit-identical), `1441 < -> <=` (q=1e-30
unreachable); `solve_gauquelin_sector` `1327 < -> ==` (the gp<1e-12 interval is
reachable — min |gp| ~1.9e-13 — but both HEAD and mutant return
`Err(NumericalFailure)` there, differing only in message, which the campaign does
not pin), `1327 < -> <=` (gp==1e-12 measure-zero), `1335 < -> <=` (delta==1e-9
measure-zero). This brings the **running documented-equivalent tally to
`30 + 6 = 36`**. The `6` is **measured, not predicted**: the authoritative scoped
run (`-F 'in (pullen_sr_houses|pullen_sd_houses|albategnius_houses|
solve_gauquelin_sector|gauquelin_houses)$'`, 233 mutants) reports
`6 missed / 227 caught / 0 unviable`. No parity gate was touched; the tier stays
report-only; `mise run ci` is green. **Remaining houses PRs:** sunshine/solar-arc
(`sunshine_houses`/`sunshine_offsets`/`apparent_solar_declination`/…),
quadrant/projection (`solve_placidian_cusp`/`topocentric_latitude`/
`regiomontanus`/`koch`/campanus/alcabitius/morinus/carter), then catalog +
thresholds (which adds `-p pleiades-houses` to `[tasks.mutants]`).

**Progress (2026-07-24) — houses Sunshine/solar-arc
(`pleiades-houses/src/systems/mod.rs`,
`sunshine_houses`/`sunshine_offsets`/`apparent_solar_declination`/
`apparent_midheaven_declination`/`nutation_for`):** fourth PR of the
post-baseline `pleiades-houses` expansion campaign (spec:
`docs/superpowers/specs/2026-07-22-fu9-houses-mutant-triage-design.md`; plan:
`docs/superpowers/plans/2026-07-24-fu9-houses-sunshine-mutant-triage.md`).
Triaged the Sunshine/solar-arc family from `126` surviving mutants
(`sunshine_houses` 68, `sunshine_offsets` 34, `apparent_solar_declination` 20,
`apparent_midheaven_declination` 2, `nutation_for` 2 — matching the design's
whole-crate prediction 68/34/20/2/2 exactly) to **5 documented equivalents**.
**Tests-only.** The pure numeric helpers were pinned by **independent
recomputation**: `apparent_solar_declination` (20→0) from the published
NOAA/USNO low-precision Sun series and `sunshine_offsets` (34→0) from the
Albategnian semi-arc trisection (both re-derived, not transcribed, in the shared
`houses-reference.py` as `solar_declination`/`sunshine_offsets`, cross-validated
to ~1e-9), and `apparent_midheaven_declination` (2→0) by a crafted product-form
geometry (`sin*tan != sin+tan != sin/tan`). `sunshine_houses` (68→3) was pinned
by `recompose_sunshine` — an **independent recomposition** threading the
un-mutated `local_sidereal_time`/`asc1`/`longitude_opposite`/
`signed_longitude_difference` and the just-pinned
`apparent_solar_declination`/`sunshine_offsets` — asserted equal to the crate
over a geometry table that straddles the `acmc < 0` axis flip and the
`mc_under_horizon` under-horizon flip (both hemispheres, a `lat == 0` row, an
`acmc == 0` axis boundary, an exact `|lat − mc_dec| == 90.0` under-horizon
boundary, and a degenerate `c == 0` semi-arc). **Documented residual — 5
equivalent mutants**, left visible (no `#[mutants::skip]`) and enumerated with
per-mutant reachability arguments in
`sunshine_family_equivalent_mutants_are_documented`: `nutation_for` `600:30
/ -> *` and `/ -> %` (the `delta_psi_arcsec / 3600.0` term is the FIRST tuple
element, discarded at both call sites — `asc_mc` `mod.rs:268`,
`validated_obliquity` `mod.rs:610` — so it reaches no public output);
`sunshine_houses` `1576:36 > -> >=` (`house > 7` vs `>= 7`, but the loop's fixed
house set `[2,3,5,6,8,9,11,12]` never contains 7, so both agree on every
reachable value), `1585:32 < -> <=` (the `c.abs() < f64::EPSILON` div-guard
differs only at `c.abs() == EPSILON`, but the reachable `c.abs()` set is
`{0.0} ∪ [~8.5e-7°, …]` with `EPSILON ≈ 2.22e-16°` in the unreachable gap), and
`1600:27 + -> -` (`sidereal_time + 180.0` vs `- 180.0` differ by exactly 360°;
`asc1` normalizes its first argument mod 360, so the physical cusp is identical
— the only f64 divergence is a measure-zero wraparound seam, `~1e-12°` vs
`359.999999999999°`, one modular Longitude differing by `~2e-12°`, well within
the 1e-9 recomposition tolerance). The `5` is **measured, not predicted**: the
plan forecast `2` (assuming `sunshine_houses → 0`), but the scoped re-runs
established 3 additional genuine `sunshine_houses` equivalents — and an
adversarial review of the residual **refuted two initially-suspected
equivalents** (`1585:32 < -> ==`, killable because `c.abs() == 0.0` is reachable
at a house-8 degenerate semi-arc → NaN divergence; and `1546:92 > -> >=`,
killable because `latitude` is a free input so `|lat − mc_dec| == 90.0` is
f64-exact via `lat = mc_dec + 90.0`), both of which were then **killed** by
crafted exact-boundary tests. This brings the **running documented-equivalent
tally to `36 + 5 = 41`**. The authoritative scoped run
(`-F 'in (sunshine_houses|sunshine_offsets|apparent_solar_declination|
apparent_midheaven_declination|nutation_for)$'`, 137 mutants)
reports `5 missed / 132 caught / 0 unviable`. No parity gate was
touched; the tier stays report-only; `mise run ci` is green. **Remaining houses
PRs:** quadrant/projection (`solve_placidian_cusp`/`topocentric_latitude`/
`regiomontanus`/`koch`/campanus/alcabitius/morinus/carter), then catalog +
thresholds (which adds `-p pleiades-houses` to `[tasks.mutants]`).

**Progress (2026-07-24) — houses Quadrant/projection
(`pleiades-houses/src/systems/mod.rs`, `topocentric_latitude`/
`solve_placidian_cusp`/`regiomontanus_houses`/`koch_houses`/
`validate_topocentric_observer`/`midpoint_longitude`):** fifth PR of the
post-baseline `pleiades-houses` expansion campaign (spec:
`docs/superpowers/specs/2026-07-22-fu9-houses-mutant-triage-design.md`; plan:
`docs/superpowers/plans/2026-07-24-fu9-houses-quadrant-mutant-triage.md`).
This slice opened by splitting the 3,209-line
`crates/pleiades-houses/src/systems/tests.rs` into a per-family
`systems/tests/` directory (`support`/`request`/`dispatch`/`trivial`/
`quadrant`/`greatcircle`/`sector`/`sunshine`/`primitives`) — a verified no-op
move (identical `cargo nextest list` inventories before/after), per AGENTS.md's
"split it before adding more, not after". **Tests-only.**

The re-measured whole-file baseline at `a8917919f` (Sunshine's landed SHA) is
`1,128 mutants tested / 83 missed / 1,038 caught / 7 unviable`, decomposing
three ways with no remainder: `28` `catalog_name` survivors (untouched,
deferred to PR 6), `32` prior-slice documented equivalents (untouched —
Foundation 13 / Great-circle 8 / Sector 6 / Sunshine 5), and this PR's `23`
Quadrant/projection targets. Triaged the 23 to **3 documented equivalents**,
per function:

| Function | Survivors at baseline | Survivors now | Killed by |
|----------|----------------------|---------------|-----------|
| `topocentric_latitude` | 9 | **0** | WGS-84 elevation pins + closed-form cross-check |
| `solve_placidian_cusp` | 6 | **2** (equivalent) | Bisection root pin + both fail-closed guards |
| `regiomontanus_houses` | 5 | **0** | SE corpus rows c1_lat40, c2_lat55 |
| `koch_houses` | 1 | **0** | Private-seam polar-circle guard |
| `validate_topocentric_observer` | 1 | **1** (equivalent, proven unreachable) | — |
| `midpoint_longitude` | 1 | **0** | SE corpus Sripati row c1_lat40 |
| **Total** | **23** | **3** | |

A SE-corpus / independent-Python-reference hybrid was used, keyed to survivor
shape: the two whole-cusp-array pins (`regiomontanus_houses`,
`midpoint_longitude`) need an external authority, since
`midpoint_longitude`'s only prior test compared the crate's Sripati output
against `midpoint_longitude` itself — the same function on both sides, so a
`-> Default::default()` mutant produces `0 == 0` and still passes; only a
Swiss-Ephemeris corpus row breaks that circularity. The pure numeric helpers
(`topocentric_latitude`, `solve_placidian_cusp`) instead need an
**independently-formulated** reference, not merely an independent transcript:
`topocentric_latitude` was cross-checked against the published WGS-84 datum
constants (`WGS84_A`, `WGS84_INV_F`) evaluated by a second, genuinely
different closed-form identity (`tan(phi') = (1-f)^2 tan(phi)`, exact at sea
level); `solve_placidian_cusp`'s Newton iteration was cross-checked against a
bisection root-finder over the same published residual — a different
numerical method that cannot inherit a Newton-specific convergence error.
**Correction (final whole-branch review, 2026-07-24):** the
"independently-formulated, not merely an independent transcript" claim was
only true for `topocentric_latitude`'s `h = 0` closed-form pin above. The
`h != 0` elevation pins
(`topocentric_latitude_pins_the_wgs84_reduction_with_elevation`) were in fact
constrained only by `houses-reference.py::topocentric_latitude`, a
line-for-line transcript of the crate's own prime-vertical formula (same `a`,
`1/f`, `e2`, `N`, and `atan2((N(1-e2)+h)s, (N+h)c)`); at `h = 0` the `N` factor
cancels out of the `atan2`, so the closed form never exercises it, and the
`1680:*` prime-vertical mutants plus `1681:29` were pinned only by the
transcript. Closed by
`topocentric_latitude_matches_the_parametric_latitude_formulation`
(`quadrant.rs`), which cross-checks the same `h != 0` pins against a third,
genuinely independent formulation — the parametric (reduced) latitude `beta`,
which never forms `N` — agreeing to 1.421e-14 / 7.105e-15, both far inside the
1e-12 tolerance.

**Per-mutant margin tables** (never aggregated, per the campaign discipline —
reproduced verbatim from the task briefs):

`topocentric_latitude`, displacement at `lat = 40°, h = 1000 m` versus the
`1e-12` tolerance:

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

`solve_placidian_cusp`, displacement at `RAMC = 90°, lat = 61°` versus the
`1e-11` tolerance:

| Mutant | House 11 | House 12 | House 2 | House 3 |
|--------|----------|----------|---------|---------|
| 1739:49 `+`→`-` | 3.41e-10 | 3.83e+01 | 3.83e+01 | 3.41e-10 |
| 1739:37 `*`→`/` | 5.73e-10 | 1.04e-09 | 1.04e-09 | 5.73e-10 |

True minimum displacement **3.41e-10**, a ~34× margin; `HEAD` agrees with the
bisection root to **≤ 2.84e-14**, a ~352× margin on the passing side.

**Documented residual — 3 equivalent mutants**, all left visible (no
`#[mutants::skip]`) and enumerated with per-mutant reachability arguments in
`quadrant_family_equivalent_mutants_are_documented`:

- **(VT-1)** `validate_topocentric_observer` `618:5` → `Ok(())`:
  `validated_obliquity` calls `validate_observer` *before*
  `validate_topocentric_observer`, and `validate_observer` already maps a
  non-finite elevation to the identical `InvalidElevation` kind and message
  for every system — no input can reach this function in a state where it
  would return `Err`, so the whole-function replacement is byte-identical on
  every path.
- **(PL-1)** `solve_placidian_cusp` `1741:21` `<` → `<=`: the zero-derivative
  guard differs only at `gp.abs() == 1e-12` exactly. `gp` is a function of three
  free test inputs — latitude, `st_deg`, and `obliquity_deg` — and near the
  vanishing-derivative latitude, all three perturb it comparably (~5e-15 per ulp).
  Even so, landing bit-exactly on the `1e-12` boundary from any combination of
  the three free parameters is a lattice-search coincidence. Measure-zero and
  unreachable.
- **(PL-2)** `solve_placidian_cusp` `1750:24` `<` → `<=`: unreachable because it
  differs only at the exact-equality coincidence `|delta| == 1e-9` — every escape
  route needs delta's Newton iterate to land on that boundary bit-for-bit, which a
  quadratically-shrinking sequence does not do.

This brings the **running documented-equivalent tally to `41 + 3 = 44`**,
continuing the campaign-wide series the prior entries maintain
(`9 → 22 → 30 → 36 → 41`). Within the `pleiades-houses` crate alone the
sub-total is `32 + 3 = 35` (Foundation 13 + Great-circle 8 + Sector 6 +
Sunshine 5 + this PR's 3); the PR 5 plan quoted that sub-total as if it were
the running tally, which would have restarted the campaign-wide series.
The `3` is **measured, not predicted**: the plan
forecast `3 missed` including `618:5` from the scoped run itself, but the
scoped run (`-F 'in (topocentric_latitude|solve_placidian_cusp|
regiomontanus_houses|koch_houses|validate_topocentric_observer|
midpoint_longitude)$'`, 145 mutants) reported only **2 missed** — `1741:21
<=` and `1750:24 <=` — because cargo-mutants matches `-F` against the
mutant's full description, and whole-function replacement mutants are named
`replace validate_topocentric_observer -> Result<(), HouseError> with Ok(())`;
they never end in `in <function>`, so the `in (...)$` anchor structurally
excludes them. `618:5` appears nowhere among the 145 scoped mutants (verified:
the run's per-mutant logs cover lines 843–1763 only, and
`midpoint_longitude`'s `1790:5` is likewise absent). **Guidance for the
remaining campaign slices: a scoped `-F` run verifies operator mutants only; a
whole-file run is required to confirm function-replacement survivors.** The
whole-file confirmation re-run (1,128 mutants tested in 21m) reports `63
missed / 1,058 caught / 7 unviable / 0 timeout` — 83 → 63 exactly as
predicted, decomposing with no remainder into `28` `catalog_name` + `32`
prior-slice equivalents + this PR's `3` (`618:5`, `1741:21 <=`, `1750:24
<=`), confirming no prior slice regressed and that VT-1/PL-1/PL-2 all
survived as predicted. No parity gate was touched; the tier stays
report-only; `mise run ci` is green.

**Record-keeping for this slice:**

- **Deferred to PR 6, as an explicit decision, not an omission:**
  `assert_corpus_cusps` was added this PR, but the six pre-existing
  open-coded corpus closures in `quadrant.rs` (the
  `*_match_swiss_ephemeris_corpus_*` tests — Morinus, Placidus+Topocentric,
  Koch, Campanus, Alcabitius c1_lat40, Alcabitius c2_lat55 — ~270 lines of
  near-identical arrange blocks) were **not** migrated onto it. The PR 5 plan
  deliberately excluded that migration as a maintainability change, not a
  triage-slice change. PR 6 scope. While there, rename the five of those six
  tests named `..._within_120_arcsec` that actually assert a `1.0` arcsec
  tolerance (only `alcabitius_cusps_c2_lat55_match_swiss_ephemeris_corpus_within_1_arcsec`
  is named correctly today).
- **A third plan defect** (alongside the two already recorded above): the
  plan's Task 2 Step 1 expected every `h = 0` closed-form cross-check in
  `houses-reference.py` to print `diff=0.000e+00`; `lat = -33.0` actually
  prints `7.105e-15`. Harmless — the Rust pin uses a `1e-12` tolerance — but
  the plan text was wrong.
- **Benign spec deviation:** the PR 5 addendum said new tests would land in
  `quadrant.rs` and `dispatch.rs`; the Sripati anchor landed in `trivial.rs`
  instead, and the `systems/tests/` split (this PR's opening move) produced
  `request.rs` and `trivial.rs` beyond the six family files the addendum
  named. Both are improvements over the addendum's plan, recorded here so
  they are not silent.
- **No per-mutant margin table for the corpus-anchored kills.** Unlike
  `topocentric_latitude` (9 rows) and `solve_placidian_cusp` (2 rows) above,
  no per-mutant displacement table exists for the 5 `regiomontanus_houses` +
  1 `midpoint_longitude` mutants — do not fabricate or aggregate one; campaign
  discipline is per-mutant rows, and none were measured for this group. These
  six are killed by whole-cusp-array comparison against the Swiss Ephemeris
  corpus (`assert_corpus_cusps`/`assert_eq!` on all 12 cusps), not by a scalar
  displacement pin, so the measured HEAD residuals against the 1″ tolerance
  are recorded instead, as tolerance headroom rather than a mutant margin:
  Regiomontanus `c1_lat40` 0.0424″, `c2_lat55` 0.0941″, Sripati `c1_lat40`
  0.0092″.

> **Correction to the Sector slice (PR 3).** That slice documented
> `solve_gauquelin_sector`'s `1327:21 <` → `==` as an equivalent mutant on the
> grounds that "the campaign does not pin error-message text". That premise is
> incorrect: `systems/tests.rs` already pinned error-message text in several
> places before this PR. PR 5 kills the structurally identical mutant in
> `solve_placidian_cusp` (1741 `<` → `==`) by asserting the message, so
> `solve_gauquelin_sector`'s GQ-1 is **killable the same way** and its
> equivalent classification should be withdrawn. Not done here — it is PR 3's
> territory and would change a merged slice's tally. **Follow-up:** add the
> message assertion to `solve_gauquelin_sector_fails_closed_on_nonconvergence`
> and drop GQ-1, taking the Sector residual from 6 to 5.

**Progress (2026-07-25) — houses Catalog + thresholds
(`pleiades-houses/src/catalog/mod.rs`, `catalog_name` in `systems/mod.rs`,
`src/thresholds.rs`):** sixth and **final** PR of the post-baseline
`pleiades-houses` expansion campaign (spec:
`docs/superpowers/specs/2026-07-22-fu9-houses-mutant-triage-design.md`; plan:
`docs/superpowers/plans/2026-07-25-fu9-houses-catalog-mutant-triage.md`).
Triaged `catalog/mod.rs` from `15` surviving mutants to **2 documented
equivalents** and `catalog_name` from `28` to **0**. **Tests-only** apart from
one deliberate single-source refactor (below). This slice opened by splitting
`catalog/tests.rs` into a per-concern `catalog/tests/` directory
(`descriptor`/`families`/`validation`/`aliases`) and relocating
`thresholds.rs`'s inline test module to `thresholds/tests.rs`, per AGENTS.md.
The move is test-identity-preserving, verified by comparing *leaf* test names
before and after: the plan asked for an empty `cargo nextest list` diff, which
is impossible, because splitting into submodules necessarily re-parents every
qualified id (`catalog::tests::X → catalog::tests::<concern>::X`) — the same
shape as the `systems/tests/` split at `348c00ac6`.

**Measured baseline**, by the authoritative per-file command (`cargo mutants -p
pleiades-houses --test-tool nextest --test-workspace=false --baseline run
--file <crate-relative path>`): `catalog/mod.rs` `100 mutants tested, 15
missed, 69 caught, 16 unviable`; `thresholds.rs` `1 tested, 0 missed, 0
caught, 1 unviable`; `catalog_name`'s `28` carried in from PR 5's whole-file
`systems/mod.rs` run. **Correction to the plan's baseline prose:** the plan
stated that `thresholds.rs` "contributes **no** survivors — its single mutant
is caught". That is verified **false**. The mutant is `replace
house_family_ceiling -> HouseFamilyCeiling with Default::default()`, and
`HouseFamilyCeiling` derives `Clone, Copy, Debug, PartialEq` but **not**
`Default`, so it cannot compile: it contributes no survivors because it is
**unviable**, not because it is caught. The distinction is not cosmetic — an
unviable mutant measures nothing at all about the test suite, whereas a caught
one is evidence. The plan's combined `101 tested / 15 missed / 69 caught / 17
unviable` row therefore decomposes as `catalog/mod.rs` `100/15/69/16` plus
`thresholds.rs` `1/0/0/1`.

**`catalog_name` — two mechanisms, kept in separate commits and separately
attributed.** (a) A cross-table characterization test killed all `28`
(measured `28 tested / 0 missed / 28 caught`) — a **test-only** kill, no
source change. `catalog_name` was dead through the public API: its only
dispatch site is reached through a `_` arm and `HouseSystem` is
`#[non_exhaustive]`, so no downstream caller can observe it, and it duplicated
`HouseSystemDescriptor::canonical_name` for all 25 built-ins with **nothing in
the workspace asserting the two agree**. The new test asserts exactly that
agreement across the whole table; the kills are real rather than tautological
because `catalog_name`'s match table is independent of
`BUILT_IN_HOUSE_SYSTEMS` (independently re-verified at review). (b) A separate
single-source **refactor** then made `catalog_name` delegate to the descriptor
table, taking the mutant *surface* from `28` to `2` (measured `2 tested / 0
missed / 2 caught`, both whole-function `-> &'static str` replacements at
`1887:5`). That is a **surface reduction, not a kill** — the kill was (a), one
commit earlier. Attribution detail, corrected against the plan: of the 28
pre-refactor mutants, 26 were arm-deletes — 25 built-in arms plus the
`Custom(_)` arm — and the refactor deleted **25** arms, not 26. The
`Custom(_) => "Custom"` arm **still exists**; its arm-delete mutant is simply
no longer generated, for a cargo-mutants *genre* reason (plausibly, stated as
a plausible cause and not a verified mechanism, that the arm-deletion genre
stops applying once the fallback is a computed `other => …` binding rather
than a literal `_` wildcard). The net `28 → 2` (−26) arithmetic is unaffected;
only the causal attribution of one mutant changes. An unfiltered `cargo
mutants --list` confirmed the `-F 'catalog_name'` filter excluded nothing
structurally.

**`catalog/mod.rs` — 15 → 2, bucket by bucket** (all test-only; the file
itself was never modified):

| Bucket | Survivors | Now | Killed by |
|--------|-----------|-----|-----------|
| Renderings and vectors (`678:5` ×3, `245:9` ×2, `428:9`) | 6 | **0** | exact string pins on the eight `latitude_sensitive_house_failure_modes()` notes, on `HouseSystemDescriptor::failure_mode_summary_line()`, and on all four `HouseSystemCodeAliasValidationError` renderings |
| Counters (`462:24`, `594:24`, `610:28`, all `+=` → `*=`) | 3 | **0** | pinning `house_catalog_validation_summary()` (`entry_count` 25, `baseline_entry_count` 12, `release_entry_count` 13, `label_count` 181) and both private validators' own return values (22 alias entries, 181 labels) |
| Validation guards and the `Custom` family arm (`118:13`, `160:50`, `219:13`, `645:49`) | 4 | **0** | crafted single-operand fixtures: a padded `"  Equal  "` canonical name; two case-variant aliases `["EQUAL", "equal"]` against a one-variant control that must stay legal; a `HouseSystem::Custom` descriptor asserting the `Custom` formula family rather than `Unknown`; all-Custom and mixed slices for the family collector's or-guard |
| No-argument public validator wrappers (`475:5`, `637:5`) | 2 | **2** (equivalent) | — |
| **Total** | **15** | **2** | |

**Documented residual — 2 equivalent mutants**, both left visible (no
`#[mutants::skip]`) and enumerated with per-mutant reachability arguments in
`catalog_equivalent_mutants_are_documented`:

- **(CAT-1)** `475:5` `replace validate_house_system_code_aliases -> Result<(),
  HouseSystemCodeAliasValidationError> with Ok(())`.
- **(CAT-2)** `637:5` `replace validate_house_catalog -> Result<(),
  HouseCatalogValidationError> with Ok(())`.

Both are whole-function replacements on a **no-argument** public validator that
wraps a private slice-taking entry point over one immutable built-in table:
CAT-1 validates exactly `house_system_code_aliases()` (the private `const
SWISS_EPHEMERIS_HOUSE_SYSTEM_CODE_ALIASES`), CAT-2 exactly
`built_in_house_systems()` (the private, immutable `static
BUILT_IN_HOUSE_SYSTEMS: [HouseSystemDescriptor; 25]` — a **`static`, not a
`const`**, and stated that way deliberately, since the argument turns on
immutability and so the storage class matters; the alias table *is* a `const`).
The argument closes three ways for each: no caller can substitute entries (both
slice-taking entry points are module-private and unexported, and `lib.rs`
declares `mod catalog;` privately, so the module is unreachable downstream
regardless of item visibility); neither table can be reached in an invalid
state (payload types hold only `&'static str`, `bool`, `Option<f64>`,
`HouseSystem`, `CompatibilityClaimTier` and `&'static [&'static str]` — no
interior mutability — and the crate is `#![forbid(unsafe_code)]`); and no build
configuration swaps them (`Cargo.toml` declares no `[features]` at all, the
crate carries no non-`cfg(test)` `#[cfg]`, and there is no
`build.rs`/`include!`). Both tables validate, asserted in the same test, so
HEAD returns `Ok(())` on every reachable input — byte-identical to the mutant
on every path. This is PR 5's VT-1 shape. The guards are **not** dead code:
the private entry points that do the real work have reachable failure branches
and their mutants **are** killed above by crafted invalid slices; killing the
two no-argument wrappers would require adding a test-only injection seam to
production code purely to observe a guard over a compile-time constant.

The reachability argument **survived a nine-line adversarial refutation
attempt** at review, every line closed: a workspace-wide caller census; an
`Err`-observability grep (the only two strings HEAD's `Err` branch can produce
appear at two `pleiades-validate` format sites and in **zero** test
assertions); the one workspace call site that *does* hold a descriptor slice,
`pleiades_validate::compatibility::verify_house_system_aliases`, which invokes
both wrappers with no arguments and then checks its own `entries` separately;
of its five tests exercising this call site, four craft invalid descriptors and
assert errors from validate's own loop, and one
(`..._allows_case_insensitive_duplicate_house_aliases_within_entry`) is a
happy-path `expect(Ok)` — neither shape observes the wrapper's `Err`; module
and item visibility; table storage class and interior
mutability across both crates; `Cargo.toml`/`cfg`/`build.rs`; doctests; the
generic/trait angle; and side-effect observability.

**Sector GQ-1 withdrawn**, discharging the follow-up recorded in the
correction block above. `solve_gauquelin_sector`'s `1327:21 <` → `==` was
classified equivalent by PR 3 on the premise that "the campaign does not pin
error-message text", which was false. Killed by
`solve_gauquelin_sector_fails_closed_on_a_zero_derivative`: at `(ramc_deg 280,
latitude_deg 68.926_784_442_096_97, obliquity_deg 23.4366, fraction 8/9, sign
1.0)` the derivative `gp` lands at `|gp| = 3.875e-18`, inside the
`gp.abs() < 1e-12` interval, so HEAD
takes the zero-derivative branch and the `==` mutant does not; the test asserts
both `HouseErrorKind::NumericalFailure` and the exact message `"gauquelin
sector iteration encountered a zero derivative"`, which is what distinguishes
them (the geometry was independently re-derived in Python at review and both
operators simulated). Measured by a scoped `solve_gauquelin_sector` run: `46
tested / 2 missed / 44 caught`, the 2 missed being only the `<=` variants at
`1327:21` and `1335:24`. **Provenance note:** the Sector residual is now
stated as `5`, which is PR 3's measured `6` minus this one confirmed kill —
arithmetic over a merged slice's measurement, not a re-measurement of PR 3's
233-mutant scope, which this PR did not re-run. Resulting tally arithmetic:
Sector `6 → 5`, houses sub-total `35 → 34`, campaign-wide `44 → 43`, then
**`+2`** for CAT-1/CAT-2 = **45**.

**Whole-crate confirmation.** Run whole-crate — *not* `-F`-scoped — per PR 5's
closing guidance that a scoped filter structurally excludes whole-function
replacement mutants, which is exactly the genre of both residuals here.

The command actually run, three times — the whole-crate invocation split into
three disjoint shards, because the un-sharded run takes ~40 min and the
environment it was run in caps a single foreground command at **10 minutes**
(background execution is unavailable there, so waiting it out was not an
option):

```bash
# k = 0, 1, 2 — cargo-mutants shards are 0-indexed
cargo mutants -p pleiades-houses --test-tool nextest --test-workspace=false \
  --baseline run --shard <k>/3 -j 4
```

Sharding partitions the mutant list, so it changes no verdict, and each shard
ran its own unmutated baseline (all three `Success`). The `-j 4` parallelism
changes only wall clock — and the risk it carries, that concurrent load inflates
test durations past cargo-mutants' auto-set timeout and converts real verdicts
into timeouts, is excluded by the measured **`0 timeout`** across all three
shards. cargo-mutants 27.1.0. Verbatim summary lines:

```
402 mutants tested in 4m: 8 missed, 371 caught, 23 unviable
402 mutants tested in 4m: 13 missed, 389 caught
401 mutants tested in 5m: 15 missed, 385 caught, 1 unviable
```

Union: **`1,205 mutants tested / 36 missed / 1,145 caught / 24 unviable / 0
timeout`** — the campaign's closing whole-crate measurement, and the first
whole-crate run since it opened (the Foundation entry's `1,231 mutants, 569
missed` was the opening whole-crate baseline; PR 5's `1,128` was
`systems/mod.rs` alone). The three shards' own
totals sum to it: `402 + 402 + 401 = 1,205`. Shard coverage is verified, not
assumed: the shards' verdict files hold 1,205 lines carrying 1,205 *distinct*
mutants — distinct on the **full mutant description** (`file:line:col:
replacement`), which is the only key that works here, since the coarser
`file:line` collapses to 454 and even `file:line:col` to 687 (three mutants
share `678:5`, `1094:50`, `1108:33` and `1818:17`; a pair shares `600:30`) — and
an independent `cargo mutants -p pleiades-houses --list` **on the same checkout**
reports exactly 1,205. Equal line count, equal distinct count and equal
enumeration total together exclude both overlap and omission. Three independent
cross-checks reconcile it:

- `1,231 − 26 = 1,205`. The `catalog_name` refactor is the only
  production-*logic* change in the entire campaign — the test-module
  relocations move `#[cfg(test)]` code, which cargo-mutants does not mutate —
  and it removed exactly 26 mutants. The identity holding exactly is also what
  establishes that the Foundation entry's `1,231` and this run are comparable
  measurements.
- Per file: `systems/mod.rs` 1,102 + `catalog/mod.rs` 100 + `error.rs` 2 +
  `thresholds.rs` 1 = 1,205. This split is **tallied from the shard verdict
  files themselves** (every `missed`/`caught`/`unviable`/`timeout` line, keyed on
  its file path), not derived from any earlier figure — which is what keeps it
  independent of the first cross-check. `1,128 − 26 = 1,102` is then a
  *prediction* from PR 5's whole-file figure that the tally agrees with, not its
  source. (`error.rs` contributes 2 mutants and 0 survivors; it appears in no
  bucket because it needs none. No prior entry enumerated it.)
- Unviable: `systems/mod.rs` 7 (unchanged from PR 5) + `catalog/mod.rs` 16 +
  `thresholds.rs` 1 = 24 — an independent confirmation of the corrected
  `thresholds.rs` decomposition above.

Verbatim union of the three shards' `missed.txt`, sorted by file/line/column, so
the no-remainder claim below is auditable from this entry alone rather than only
by cross-referencing five earlier ones:

```
catalog/mod.rs:475:5:  replace validate_house_system_code_aliases -> Result<(), HouseSystemCodeAliasValidationError> with Ok(())
catalog/mod.rs:637:5:  replace validate_house_catalog -> Result<(), HouseCatalogValidationError> with Ok(())
systems/mod.rs:201:49: replace - with + in asc_mc_from
systems/mod.rs:208:38: replace + with - in asc_mc_from
systems/mod.rs:215:70: replace - with + in asc_mc_from
systems/mod.rs:600:30: replace / with % in nutation_for
systems/mod.rs:600:30: replace / with * in nutation_for
systems/mod.rs:618:5:  replace validate_topocentric_observer -> Result<(), HouseError> with Ok(())
systems/mod.rs:1082:69: replace + with - in horizon_houses
systems/mod.rs:1094:36: replace - with + in horizon_houses
systems/mod.rs:1094:50: replace < with <= in horizon_houses
systems/mod.rs:1094:50: replace < with == in horizon_houses
systems/mod.rs:1095:56: replace < with <= in horizon_houses
systems/mod.rs:1108:33: replace > with < in horizon_houses
systems/mod.rs:1108:33: replace > with == in horizon_houses
systems/mod.rs:1108:33: replace > with >= in horizon_houses
systems/mod.rs:1327:21: replace < with <= in solve_gauquelin_sector
systems/mod.rs:1335:24: replace < with <= in solve_gauquelin_sector
systems/mod.rs:1437:10: replace > with >= in pullen_sr_houses
systems/mod.rs:1441:34: replace < with <= in pullen_sr_houses
systems/mod.rs:1458:13: replace > with >= in pullen_sr_houses
systems/mod.rs:1576:36: replace > with >= in sunshine_houses
systems/mod.rs:1585:32: replace < with <= in sunshine_houses
systems/mod.rs:1600:27: replace + with - in sunshine_houses
systems/mod.rs:1741:21: replace < with <= in solve_placidian_cusp
systems/mod.rs:1750:24: replace < with <= in solve_placidian_cusp
systems/mod.rs:1799:9:  delete match arm 3 in asc1
systems/mod.rs:1799:30: replace - with + in asc1
systems/mod.rs:1807:20: replace < with <= in asc2
systems/mod.rs:1811:39: replace < with <= in asc2
systems/mod.rs:1818:17: replace < with <= in asc2
systems/mod.rs:1818:17: replace < with == in asc2
systems/mod.rs:1818:17: replace < with > in asc2
systems/mod.rs:1819:13: delete - in asc2
systems/mod.rs:1826:18: replace < with <= in asc2
systems/mod.rs:1833:49: replace + with - in longitude_opposite
```

(Paths abbreviated from `crates/pleiades-houses/src/…`; otherwise verbatim.)
These 36 decompose with **no remainder**, every line matching a
previously-enumerated mutant's file, line, column and operator:

| Bucket | Missed |
|--------|--------|
| Foundation — `asc_mc_from` 3, `asc1` 2, `asc2` 7, `longitude_opposite` 1 | 13 |
| Great-circle — `horizon_houses` pole-singularity clamp | 8 |
| Sector — `pullen_sr_houses` 3, `solve_gauquelin_sector` 2 | 5 |
| Sunshine/solar-arc — `nutation_for` 2, `sunshine_houses` 3 | 5 |
| Quadrant/projection — `618:5`, `1741:21`, `1750:24` | 3 |
| **This PR** — `catalog/mod.rs` `475:5`, `637:5` | 2 |
| **This PR** — `catalog_name` | 0 |
| **Total** | **36** |

No prior slice regressed and no new survivor appeared. Two positive controls in
the same run: `1327:21 <` → `==` (GQ-1) is now in `caught.txt`, confirming the
withdrawal, and `catalog_name`'s two residual mutants at `1887:5` are both
caught. The plan projected `~1,205` from `1,231 − 26`; the measurement is
**exactly** 1,205, so the *figure* in `.github/workflows/mutants.yml`'s
calibration comment needed no revision — but its "not yet measured" hedge is
discharged by this entry, and the comment was updated in this commit to cite
the measured count and point here instead of at an ephemeral plan task. The
`~1.85×` growth factor and the `~24-25m` projection it feeds are unchanged, as
is `timeout-minutes: 90`.

**Weekly-tier expansion.** `-p pleiades-houses` added to `mise.toml`'s
`[tasks.mutants]`, so the report-only weekly tier now regression-checks the
crate. The `mutants.yml` `timeout-minutes` rationale, previously a guess, was
replaced with a measured calibration from the first scheduled run
(`29733596109`): `16m05s` **job wall-clock**, of which cargo-mutants itself
self-reported ~10m (9m54s) testing **1,415** mutants (`191 missed, 1,160
caught, 64 unviable`) across the three baseline crates, the remaining ~6m being
fixed checkout/install/cache/upload overhead that does not scale with mutant
count. A pre-existing stale `1451` in `mise.toml`'s `--test-workspace=false`
rationale comment was corrected to `1415` at the same time. **Projection, not a
measurement:** `1,415 + 1,205 = 2,620` (~1.85× baseline) → ~18-19m testing +
the same ~6m fixed → **~24-25m** job wall-clock; `timeout-minutes: 90` retained
as headroom for further crate additions. No parity gate was touched; the tier
stays **report-only** (no mutation-score gate — surviving mutants are an
expected result, and exit code 2 passes the job); `mise run ci` is green.

**Record-keeping for this slice:**

- **PR 5's deferral discharged.** The six open-coded corpus closures in
  `systems/tests/quadrant.rs` were migrated onto `assert_corpus_cusps` (net
  −158 lines, `+103/−261`); all six `[f64; 12]` arrays were verified
  element-by-element byte-identical and the strict `< 1.0` arcsec tolerance was
  **not** loosened. Five of the six were mis-named `..._within_120_arcsec`
  while asserting `1.0` arcsec and were renamed to `..._within_1_arcsec`; the
  sixth, `alcabitius_cusps_c2_lat55_match_swiss_ephemeris_corpus_within_1_arcsec`,
  already carried the correct suffix. A seventh, `trivial.rs`'s
  `equal_house_angles_…`, was *measured* at `1.0`
  arcsec and renamed too, but deliberately **not** migrated: it asserts angles,
  not cusps.
- **No per-mutant margin table for this slice, by construction.** Every
  `catalog/mod.rs` kill is an exact string, count, or enum-equality pin, not a
  scalar displacement against a tolerance, so no displacement margin exists to
  report. Per campaign discipline, none is fabricated or aggregated.
- **Plan defects found and corrected — six, recorded so none is silent.** Three
  are wrong figures: `thresholds.rs`'s mutant called caught when it is unviable;
  "26 arms deleted" when 25 were; and the `mutants.yml` calibration's "~1451
  mutants" for the first scheduled run, measured at **1,415** (all three
  corrected above). One is a wrong forecast: the plan expected `catalog_name`'s
  post-refactor residual to be "roughly 3"; it measured **2**. Two are
  unsatisfiable acceptance checks: an empty `cargo nextest list` diff across a
  module split (repaired to a leaf-name comparison, preserving its stated
  purpose), and confirming a 233-mutant Sector total from a function-scoped run
  (repaired by disclosing provenance rather than by guessing PR 3's filter).
  Separately, the plan's roadmap estimate of "~13,900" was recomputed from
  verified figures as **13,910**.
- **One doc-comment overclaim corrected during the refactor:** the replacement
  test's doc initially said the `Unspecified` fallback was pinned. It is
  unreachable — the 25 built-ins plus `Custom` cover every input — and the doc
  now says "unreachable, defensive for a future `#[non_exhaustive]` variant".

**`pleiades-houses` campaign COMPLETE.** Six PRs — Foundation, Great-circle,
Sector, Sunshine/solar-arc, Quadrant/projection, Catalog + thresholds — and
every file in the crate now reaches `0` surviving mutants or a documented
equivalent, confirmed by the whole-crate run above. In-crate
documented-equivalent sub-total **36**: `34` from the first five PRs
(Foundation 13 + Great-circle 8 + Sector 5 + Sunshine 5 + Quadrant/projection
3, after GQ-1's withdrawal took Sector `6 → 5`) plus this PR's `2`.
Campaign-wide running tally **45** (`9` from the closed three-crate baseline
plus the houses `36`), continuing the series
`9 → 22 → 30 → 36 → 41 → 44 → 43 → 45` — the dip is GQ-1's withdrawal, a
documented equivalent reclassified as killable and then killed. That is the
second such reclassification in this campaign: the Foundation slice's own
correction took its residual from `19` to `13` the same way, before it was
merged (its entry preserves the superseded `9 + 19 = 28` figure).

**FU-9 stays open as a standing posture entry** — the same disposition the
2026-07-18 three-crate baseline took when it closed. There is **no remaining
slice** for either the measured baseline or the houses campaign. The
report-only mutants tier remains, so any future expansion to further
`pleiades-*` crates opens new slices under this follow-up: new work, not part
of either closed body.

**Next-campaign roadmap** (measured, so the next slice's ordering rests on data
rather than crate size):
`docs/superpowers/specs/notes/2026-07-25-mutants-roadmap-baseline.md`. Four
candidate crates were measured at the close of this campaign —
`pleiades-apsides` `223 tested / 33 missed` (84.9%), `pleiades-backend` `263 /
70` (67.4%), `pleiades-ayanamsa` `305 / 85` (69.8%), and `pleiades-fict` `308 /
148`, whose run exited 3 with 2 mutants timing out, so its 49.5% covers only
306 of the 308 verdicts and is **not directly comparable** to the other three;
the note keeps that row but visibly demotes it, and the remedy (a re-run with a
raised test timeout) is named there rather than done. Eight further crates are
**sized only** — `cargo mutants --list` counts, no survivor data, so no
ordering may be inferred from them: `pleiades-compression` 607,
`pleiades-eclipse` 913, `pleiades-core` 962, `pleiades-vsop87` 1,493,
`pleiades-elp` 1,521, `pleiades-data` 1,752, `pleiades-events` 1,901,
`pleiades-jpl` 3,662 — subtotal 12,811, which with the four measured crates'
1,099 tested mutants gives a twelve-crate total of **13,910**. For scale: this
campaign covered **one crate**. Thirteen `pleiades-*` logic crates lay outside
the closed three-crate baseline — the twelve in that note plus
`pleiades-houses` — so twelve remain. (`pleiades-cli` and `pleiades-validate`
are outside the tier, which enumerates its crates explicitly. Relatedly but not
identically: `[tasks.mutants]`'s comment cites those two crates' 300+ second
individual tests as the rationale for `--test-workspace=false`, since without it
every mutant would pay their cost.)


**Progress (2026-09-08) — `pleiades-apsides`, a NEW post-baseline expansion
slice:** triaged from `33` → **`1`** documented equivalent, whole crate.

This is **not** part of the closed 2026-07-18 three-crate baseline, nor of the
closed `pleiades-houses` campaign. Both of those are finished. This is the first
slice of the expansion those closures explicitly anticipated ("any future
expansion to further `pleiades-*` crates opens new slices under this follow-up:
new work, not part of either closed body"), taking the first crate off the
`2026-07-25` roadmap queue.

**Measured, not estimated.** Baseline re-measured on this branch at
`7aefa946c`: `223 mutants tested in 2m: 33 missed, 186 caught, 4 unviable`
(exit 2). Final, after six kill tasks plus **three** review-driven kills, each
reclaiming a mutant this slice had already written off as an equivalent:

```
223 mutants tested in 3m: 1 missed, 218 caught, 4 unviable
```

Exit 2 both times — the report-only tier's expected outcome. Command:
`cargo mutants -p pleiades-apsides --test-tool nextest --test-workspace=false
--baseline run`. `32` mutants killed by `13` new tests and `1` new independent
reference helper (`state_from_elements`, a published perifocal-basis forward
construction deliberately *not* the crate's own inverse formulation, so a sign
or operator error in `apsides` cannot be masked by a shared expression). Crate
suite `19 → 23` tests, all passing.

**Per-task kills, each verified by its own whole-crate run:** forward-reference
geometry `10` (`33 → 23`), southern-perihelion branch `5` (`→ 18`),
`points_from_elements` guards `7` (`→ 11`), overflow-lens guards + negative μ
`3` (`→ 8`), `elements_from_state` node guards `3` (`→ 5`), derived-eccentricity
floor `1` (`→ 4`), then review-driven underflow lens `1` (`→ 3`) and
review-driven near-pole aphelion pin `1` (`→ 2`), and review-driven
perihelion-at-the-node identity `1` (`→ 1`).

**Mutant IDs in this entry are the CURRENT ones**, as printed by a run against
the merged branch — not the triage-time line numbers. The difference matters
because this slice adds no `#[mutants::skip]`: the whole point is that a
maintainer can take a survivor out of a weekly report and find its written
argument, which fails if the identifiers disagree. The one surviving equivalent
is **`120:43`**. Sites at or below the original line 104 are shifted `+16` by
the equivalence comment block that remains in `lib.rs`; sites above it
(`76:23`, `81:35`) are unchanged. Triage-time IDs, should the intermediate
reports need reconciling, are the current ones minus that shift.

**The one remaining documented equivalent**, carrying a written reachability
argument as a comment at its site in `crates/pleiades-apsides/src/lib.rs`. **No
`#[mutants::skip]` was added** — the arguments are the record, and the mutants
stay visible in every future run:

- `120:43` `||` → `&&` in `apsides`' input guard. **Rust precedence plus a
  poisoned downstream value.** `&&` binds tighter than `||`, so mutating *this*
  operator yields `a || (b && c) || d`, not `((a || b) && c) || d`. That has
  **two** distinguishing regions, and both land on `Err(NonFinite)` anyway.
  **A:** `{r_mag == 0.0, mu finite, mu > 0}`, where `mu / r_mag = +inf` forces
  `c1 = -inf` and poisons `e` to `inf` or `NaN` in every sub-case (all-zero
  position → `NaN`; all-subnormal position whose squared norm underflows →
  `inf`; mixed → `NaN`). **B:** `{r_mag finite and non-zero, mu ∈ {NaN, +inf}}`
  — `mu <= 0.0` is false for both, so the mutant proceeds; `mu = NaN` gives
  `c1 = NaN` directly and `mu = +inf` gives `c1 = (v2 - inf)/inf = NaN`, both
  measured to yield `e = NaN`. Every case in A and B fails the
  `!e.is_finite()` check below, so the arm is redundant with that downstream
  check. (Region B was **missed in the first draft of this argument**, which
  claimed A was the *only* distinguishing region. The conclusion survived the
  correction; the enumeration did not.)
Campaign-wide running tally **45 → 46** (`9` from the closed three-crate
baseline, `36` from the closed houses campaign, `1` from this slice), extending
the series `9 → 22 → 30 → 36 → 41 → 44 → 43 → 45 → 46`. In-crate
documented-equivalent sub-total for `pleiades-apsides`: **1**. (Successive
drafts of this entry claimed `4` (tally `49`), `3` (`48`) and `2` (`47`);
review proved **three of the four** killable — see predictions 4, 5 and 6. The
one that survived, `120:43`, is also the one with the strongest verification
behind it: analytic on both distinguishing regions plus 448 crafted cases with
zero divergences. That contrast is the honest summary of this slice — an
equivalence claim is worth exactly as much as the search that failed to break
it.)

**Six predictions that measurement overturned.** Recorded so none is silent —
in each case the claim was rewritten from the measurement, not the other way
round. The first three were design-phase predictions overturned during
implementation; the **fourth, fifth and sixth were overturned by review**,
after this slice had already written the equivalence argument down and
committed it:

1. **`138:10` (`<` → `<=` on the eccentricity floor) was predicted an
   unreachable boundary; it is killable, and was killed.** Unlike
   `points_from_elements`, `e` is *derived* here through a cancellation that
   makes the reachable grid ~1e6× coarser than the target's precision, and the
   design's searches failed. The reason they failed is instructive: they swept
   `r_mag` over **powers of two**, which makes the final `c1 * r_mag` product
   *exact* and so never lands on the boundary. Sweeping over consecutive
   doubles gives that product an independent rounding, and a state whose
   osculating eccentricity is bit-identical to `MIN_ECCENTRICITY` falls out.
2. **`120:43` was predicted killable; it is an equivalent.** The prediction
   assumed the mutant was `((a || b) && c) || d`. Rust's precedence makes it
   `a || (b && c) || d`, which has a far smaller distinguishing region — and
   that region is entirely absorbed by the downstream `!e.is_finite()` check,
   as argued above.
3. **`220:27` was predicted killable "by any radial state"; plain radial motion
   does not reach it.** A plain radial state has `e == 1.0` *exactly*, which
   makes `r_peri = a(1 - e) = 0` and fails inside `apsides()` before the
   `h_mag` guard is ever reached. The kill needs a *crafted* velocity leaving
   `e` one ulp below `1.0`, and the test now asserts that precondition
   (`apsides()` succeeds, `r_peri != 0`) rather than assuming it.
4. **`81:35` (`||` → `&&` on `to_ecliptic`'s *output* guard) was documented as
   an equivalent; review proved it killable, and it is now killed.** The
   argument claimed `p[2] / r` always lies in `[-1, 1]`, making `asin` total.
   That fails in the **subnormal** regime: once `fl(z*z)` underflows,
   `sqrt(fl(z²)) < |z|`, so the ratio exceeds `1`. At
   `z = f64::from_bits(0x1e60000000000001)` (`2.222758749485078e-162`, whose
   square underflows to the smallest subnormal `5e-324`), `p[2] / r =
   1.0000000000000002` and `asin` returns `NaN`, while `atan2(0.0, 0.0)` stays
   a finite `0.0`. Exactly one of the two angles is non-finite — precisely the
   region where `||` and `&&` differ — so the original returns
   `Err(NonFinite)` and the mutant returns `Ok` carrying a `NaN` latitude.
   Killed by `to_ecliptic_rejects_underflowing_norm`.

   **The process lesson, stated plainly so the next slice inherits it.** This
   repo already carries a memory note on guard equivalence: *test the
   finite-overflow-to-`inf` direction before calling a non-finite-guard mutant
   equivalent.* This slice did exactly that — `to_ecliptic_rejects_overflowing_norm`
   is in the suite — and then declared a *sibling* guard equivalent without
   testing the **mirror** direction. Underflow is the other half of the same
   lens. **For any future non-finite guard, test both directions: components
   large enough that the squared norm overflows to `+inf`, and components small
   enough that the squared norm underflows to a subnormal.** The two new tests
   now sit adjacent in `tests.rs` so the pair reads as one idea. A
   documented-equivalent claim is a permanent, load-bearing assertion about
   unreachability; it earns strictly more adversarial search than a kill does,
   because a wrong kill fails loudly and a wrong equivalence sits silent.
5. **`303:46` (`+` → `-` on the aphelion argument of latitude) was documented
   as an equivalent through three successive rewrites; review proved it
   killable, and it is now killed.** In exact reals `ω+π` and `ω−π` are the
   same point, so it can only be separated where the two arguments round
   differently *and* the result is ill-conditioned — near the pole, where
   `atan2`'s arguments both collapse toward zero. At `node 0, incl 89.999997,
   peri_lon 90.00003` the correct code lands `2.59e-13` deg from an independent
   60-digit reference while `ω−π` lands `7.47e-9` deg away: the suite's
   **ordinary `1e-9` deg tolerance** separates them with `~3,800×` headroom on
   one side and `7.5×` on the other. Killed by
   `aphelion_argument_of_latitude_is_omega_plus_pi`.

   **Both reclamations in this slice have the same root cause, and that is the
   transferable lesson.** Each equivalence argument was validated against a
   reference *derived from the code* rather than independent of it — for
   `81:35`, reasoning about `asin`'s domain from the code's own guarantees
   instead of searching the input space; for `303:46`, measuring the branch
   separation against the exact-in-real value of the mutated expression, which
   subtracts out exactly the error term that breaks the symmetry. **A
   documented equivalent is only as good as the independence of the reference
   that failed to kill it.** Before writing one down, state explicitly what
   reference the non-separation was measured against and whether a test could
   actually use it; if it could not, the argument has not been tested.

   The reference question bit this slice a third time during review itself. A
   proposed kill geometry (`node 0, incl 90.00001, peri_lon 90.0001`) reported
   the correct code `5.03e-11` from truth and the mutant `1.47e-9`. Recomputing
   at 60 digits **against the exact `f64` element values the function actually
   receives** — rather than the decimal literals as written — gives `1.56e-9`
   and `3.08e-9`: *both* outside `1e-9`, so that geometry yields no kill. At
   this conditioning the `3e-15` deg gap between `90.00001_f64` and the decimal
   `90.00001` swings the reference by `1.6e-9` deg, which is larger than the
   entire effect. The geometry finally used was found by searching against the
   `f64`-faithful reference, and there the distinction is decisive in the other
   direction: under the decimal reference the *mutant* would look closer to
   truth (`5.97e-9`) than the correct code (`1.34e-8`).
6. **`242:20` (`<` → `<=` on the southern-perihelion branch) was documented as
   an equivalent through four rewrites; review proved it killable, and it is
   now killed.** The final argument rested on a step that is false: that
   `cos_omega == 1.0` exactly makes both branches agree, because `acos` returns
   `0` and `0` vs `2π` are the same after `rem_euclid(360)`. They are not.
   `(2π).to_degrees()` is exactly `360.0`, and
   `(node_deg + 360.0).rem_euclid(360.0) != node_deg` whenever `node_deg`
   carries bits below `ulp(node_deg + 360)` — true for most `node_deg < 256`.
   So `ε` has no floor at `2^-26`, and the `ω = 0` case is live rather than
   excluded. Killed by `perihelion_at_the_node_gives_peri_lon_equal_to_node`,
   which asserts the exact identity `ϖ = Ω` at a state simultaneously at a node
   (`r_z == 0`) and at an apsis (`r·v == 0`) — an invariant between two outputs,
   the same shape as Task 2's bifocal sum. The mutant returns
   `255.49849089543818` against `255.49849089543824`.

   **The robustness question was settled before shipping, not after.** The kill
   needs `cos_omega` to be exactly `1.0`, which reaches through libm
   `sin`/`cos`/`atan2`/`asin`. The state was chosen so the RAW dot is
   `1.0000000000000002` — one ulp **above** `1.0` — so `clamp(-1.0, 1.0)`
   forces the exact value and no libm difference can perturb it. That is
   structural, not lucky: `n_hat` and `peri_vec` are independently computed
   unit vectors along the same direction, differing in angle by `O(ulp)`, so
   their exact cosine deficit is `O(ulp²) ≈ 5e-33` — sixteen orders below
   `half-ulp(1.0)`. Across a 90,621-state search of node-and-apsis geometries
   the raw dot never fell below `1.0` (84,158 exactly `1.0`, 6,463 one ulp
   above, **none** below). A state landing on `1.0` by rounding alone would
   have been fragile and was rejected in favour of one the clamp protects.

**Per-mutant margin tables** (never aggregated, per the campaign discipline —
inlined below in full, one row per mutant, so the rows outlive any working
notes). No margin is fabricated for a kill that has none: `16` of the `32`
kills carry a genuine scalar displacement against an assertion tolerance, and
the remaining `16` are **exact error-variant or exact-equality pins** with no
displacement to report, disclosed as such rather than given an invented number.

Across the 16 displacement rows the **true minimum** kill margin is **`7.5 ×`**
its assertion tolerance — the near-pole aphelion pin (`303:46`), whose mutant
lands `7.47e-9` deg against a `1e-9` deg tolerance. That is stated as the
minimum, not smoothed into an aggregate. The largest is `8.08e15 ×` (`131:24`
`*` → `/`). Second smallest is `3.65e10 ×` (`243:21` `*` → `/`).

**One kill in this slice IS marginal, by design, and that is worth saying
plainly.** The near-pole aphelion pin is a last-few-ulps distinction: it
separates two expressions identical in exact arithmetic, so its `7.5 ×` margin
is the nature of what it pins, not a defect. It is defensible because the
*correct* code sits `2.59e-13` deg from the independent reference — `~3,800 ×`
inside the same tolerance — so the assertion has large headroom on the passing
side even though the failing side clears it by only `7.5 ×`. The other fifteen
displacement rows carry margins of `10^9` or more and depend on no ulp-scale
behaviour. The perihelion-at-the-node identity carries **no margin at all, by
construction**: it is an exact equality (`ϖ == Ω`, bit-for-bit) with no
tolerance to have a margin against; the mutant misses by `5.68e-14` deg, which
any exact comparison catches, and its robustness comes from `clamp` forcing
`cos_omega` to exactly `1.0` — see prediction 6.

**Rows with a genuine scalar displacement — 16.**

`apsides`, eccentricity vector and apsis positions, against the independent
forward construction at `a = 2, e = 0.2, i = 10°, Ω = 40°, ω = 30°, ν = 50°,
μ = 2.959e-4` (test `apsides_match_forward_construction_at_non_degenerate_geometry`).
Each row gives the mutant's strongest-firing assertion:

| Mutant | Firing assertion | Residual | Tolerance | Margin |
|--------|------------------|----------|-----------|--------|
| 128:17 `/`→`%` | apogee longitude | 50.2258 deg | 1e-10 deg | 5.02e11× |
| 128:17 `/`→`*` | apogee longitude | 50.2273 deg | 1e-10 deg | 5.02e11× |
| 130:19 `-`→`+` | apogee longitude | 69.5107 deg | 1e-10 deg | 6.95e11× |
| 131:19 `-`→`+` | perigee longitude | 24.9897 deg | 1e-10 deg | 2.50e11× |
| 131:24 `*`→`/` | bifocal sum `r_apo+r_peri−2a` | 8.0848e3 AU | 1e-12 AU | 8.08e15× |
| 132:19 `-`→`+` | apogee latitude | 4.6242 deg | 1e-10 deg | 4.62e10× |
| 155:39 delete `-` | apogee longitude | 139.2433 deg | 1e-10 deg | 1.39e12× |
| 155:49 `*`→`/` | bifocal sum | 1.4577 AU | 1e-12 AU | 1.46e12× |
| 155:58 delete `-` | apogee latitude | 9.9619 deg | 1e-10 deg | 9.96e10× |
| 155:68 `*`→`/` | apogee latitude | 4.1141 deg | 1e-10 deg | 4.11e10× |

True minimum in this group **4.11e10×**.

`elements_from_state`, southern-perihelion branch, at `a = 2, e = 0.2, i = 10°,
Ω = 40°, ω = 210°, ν = 50°, μ = 2.959e-4` (test
`elements_recover_southern_perihelion_argument`). Correct `peri_lon_deg` is
`250°`, the unmutated residual is `0.0` exactly, tolerance `1e-9` deg:

| Mutant | Mutated `peri_lon_deg` | Displacement | Margin |
|--------|------------------------|--------------|--------|
| 242:20 `<`→`==` | `190.000000` | 60.0000 deg | 6.00e10× |
| 243:21 `*`→`+` | `184.591559` | 65.4084 deg | 6.54e10× |
| 243:21 `*`→`/` | `286.475626` | 36.4756 deg | 3.65e10× |
| 243:45 `-`→`+` | `190.000000` | 60.0000 deg | 6.00e10× |
| 243:45 `-`→`/` | `177.509871` | 72.4901 deg | 7.25e10× |

True minimum in this group **3.65e10×**.

`points_from_elements`, near-pole aphelion argument, at `node 0,
incl 89.999997, peri_lon 90.00003, e 0.2, a 2.0`, against a 60-digit
independent reference (test `aphelion_argument_of_latitude_is_omega_plus_pi`),
tolerance `1e-9` deg:

| Mutant | Firing assertion | Residual | Correct code | Margin |
|--------|------------------|----------|--------------|--------|
| 303:46 `+`→`-` | aphelion longitude | 7.46528e-9 deg | 2.58783e-13 deg (~3,800× inside) | **7.5×** |

**Slice-wide true minimum: `7.5×`, this row.**

**Rows with NO displacement — 16 exact error-variant or `Ok`/`Err` pins.** The
assertion is a discrete `assert_eq!` on an error variant, or an
`is_ok()`/`unwrap()` that flips outright. "Displacement" is not a defined
quantity for these and none is invented:

| Mutant | Killing test | Kill mechanism | Margin |
|--------|--------------|----------------|--------|
| 264:9 `&&`→`\|\|` | `points_from_elements_rejects_non_finite_semi_major` | mutant returns `UnboundOrbit`, original `NonFinite` | exact variant pin — none |
| 265:9 `&&`→`\|\|` | same | same | exact variant pin — none |
| 266:9 `&&`→`\|\|` | same | same | exact variant pin — none |
| 267:9 `&&`→`\|\|` | same | same | exact variant pin — none |
| 271:10 `<`→`<=` | `points_from_elements_eccentricity_floor_is_exclusive` | at `e == MIN_ECCENTRICITY` mutant gives `DegenerateOrbit`, original `Ok` | exact `Ok`/`Err` pin — none |
| 271:10 `<`→`==` | same | at `e = 1e-7` mutant gives `Ok`, original `DegenerateOrbit` | exact `Ok`/`Err` pin — none |
| 274:17 `\|\|`→`&&` | `points_from_elements_rejects_unbound_conics` | neither `e=1.5,a=2` nor `e=0.5,a=-2` trips the mutated conjunction | exact variant pin — none |
| 76:23 `\|\|`→`&&` | `to_ecliptic_rejects_overflowing_norm` | overflow lens: `r = inf` but both angles finite, mutant returns `Ok` | exact `Ok`/`Err` pin — none |
| 81:35 `\|\|`→`&&` | `to_ecliptic_rejects_underflowing_norm` | underflow lens: `fl(z*z)` subnormal ⇒ `asin` `NaN` while `atan2` stays finite; mutant returns `Ok` with `NaN` latitude | exact `Ok`/`Err` pin — none |
| 120:27 `\|\|`→`&&` | `apsides_rejects_overflowing_position_norm` | mutant proceeds to `inv_a = 2.0/inf − 1e-120 ≤ 0` and returns `UnboundOrbit` | exact variant pin — none |
| 120:62 `\|\|`→`&&` | `apsides_rejects_non_positive_mu` | at `μ = −1` mutant returns `Ok` (`e = 2.0`), original `NonFinite` | exact `Ok`/`Err` pin — none |
| 220:27 `\|\|`→`&&` | `radial_motion_has_no_orbital_plane` | `h_mag == 0` yet finite; mutant returns `Ok` carrying `NaN` elements | exact `Ok`/`Err` pin — none |
| 226:14 `<`→`<=` | `node_threshold_is_exclusive_at_the_exact_boundary` | state sits bit-exactly on `n_mag == 1e-12·h_mag`; mutant returns `DegenerateNode` | exact `Ok`/`Err` pin — none |
| 226:22 `*`→`/` | `node_threshold_scales_multiplicatively_with_angular_momentum` | `1e-12·h_mag < n_mag < 1e-12/h_mag`; mutant returns `DegenerateNode` | exact `Ok`/`Err` pin — none |
| 138:10 `<`→`<=` | `apsides_eccentricity_floor_is_exclusive` | derived `e` bit-identical to `MIN_ECCENTRICITY`; mutant returns `DegenerateOrbit` | exact `Ok`/`Err` pin — none |
| 242:20 `<`→`<=` | `perihelion_at_the_node_gives_peri_lon_equal_to_node` | exact identity `ϖ == Ω`; mutant returns `255.49849089543818` where `255.49849089543824` is correct | exact equality pin, no tolerance — none |

`242:20` appears in both tables under two different mutations: `<`→`==` carries
a `60°` displacement, `<`→`<=` is an exact-identity pin. Two mutants at one
site, two unrelated kill mechanisms.

**The one surviving mutant — a documented equivalent, not a kill.**

| Mutant | Argument | Margin |
|--------|----------|--------|
| 120:43 `\|\|`→`&&` | Rust precedence makes the mutant `a \|\| (b && c) \|\| d`. Both distinguishing regions — **A** `{r_mag == 0, μ finite, μ > 0}` and **B** `{r_mag finite non-zero, μ ∈ {NaN, +inf}}` — drive `e` to `NaN` and return `Err(NonFinite)`, matching the original. Verified analytically and over 448 crafted region-A/region-B cases with zero divergences. | not applicable — no distinguishing input exists |

`16 + 16 = 32` kills, plus `1` documented equivalent, accounting for all `33`
baseline survivors.

**One test's honest-naming gap closed.** `apsides_eccentricity_floor_is_exclusive`
pins only the *on*-boundary side; on its own it would also pass if the floor
check were deleted entirely. Its doc comment now cross-references the sibling
`near_circular_orbit_is_degenerate`, which holds the below-boundary side — the
two together give the exclusivity. Neither test was restructured.

**Two plan defects found and corrected, so neither is silent.** One wrong
figure: the southern-perihelion rationale reported the `*` → `+` mutant landing
at `144.6°`, which is the raw `omega` *before* the `+40°` node offset; the
correct final `peri_lon_deg` is **`184.6°`** (measured). One wrong mechanism:
the overflow-lens rationale said `120:27`'s mutant "proceeds and returns `Ok`".
Traced and measured, it proceeds past the guard and reaches
`inv_a = 2.0/inf - 1e-120`, which is `<= 0`, so it returns `Err(UnboundOrbit)`.
It is still a kill — `UnboundOrbit != NonFinite` — but by a different exit than
described. Neither correction changes a conclusion: all five southern-perihelion
landings remain far outside the `1e-9` tolerance either way.

**Reference-lever narrowing, recorded rather than left silent.** The design
named three independent reference levers and four conic invariants. Lever 1
(forward construction, elements → state) carries the whole numeric class. Of
the conic invariants only the **bifocal sum** (`r_apo + r_peri = 2a`) is
asserted; vis-viva, the conic radius law and `h ⊥ r, v` are **deliberately not
added** — with the forward construction already pinning every apsis coordinate
they kill no additional mutant, and adding assertions that constrain nothing new
would be noise. They remain available if a future change reopens survivors here.
Lever 3 (published-constant recomputation) is used as
`mu_matches_its_published_derivation`, which recomputes
`MU_EARTH_MOON_AU3_PER_DAY2` from the GM⊕/GM☾/AU/day constants its rustdoc
cites. **That test kills no mutant and does not claim to** — cargo-mutants does
not mutate `const` items, so nothing is attached to it. Its `2e-5` tolerance is
the *measured* `1.43e-5` gap between the pure derivation
(`8.997011530622141e-10`) and the shipped, gate-tuned `8.99714e-10`, not a
tolerance picked to make the assertion pass.

**Weekly tier expanded.** `-p pleiades-apsides` joins `[tasks.mutants]` in
`mise.toml` alongside `pleiades-types`, `pleiades-time`, `pleiades-apparent`
and `pleiades-houses`. The set grows `2,620 → 2,843` mutants (~1.09× the
previous projection) on a measured 223-mutant crate. The `.github/workflows/
mutants.yml` calibration comment records the cost as roughly **+2 minutes** on
the measured ~24-25m job wall-clock, and marks it explicitly as a **projection,
not a measurement** — no scheduled run has yet executed the five-crate set, and
the next one supersedes the estimate. `timeout-minutes: 90` is retained
unchanged.

**Posture unchanged.** The mutants tier stays **report-only**: surviving mutants
(exit 2) pass the job and **no mutation-score gate was introduced**. No parity
gate was touched — the `validate-lilith` corpus, its tolerances and its gate
code are untouched, and `crates/pleiades-apsides/src/lib.rs` changed by
**comments only** (verified by diff; not one executable line differs).

**Queue after this slice: three crates.** `pleiades-backend` `263 / 70`,
`pleiades-ayanamsa` `305 / 85`, and `pleiades-fict` `308 / 148` (provisional —
its run exited 3 with 2 timeouts). The roadmap note
`docs/superpowers/specs/notes/2026-07-25-mutants-roadmap-baseline.md` now marks
`pleiades-apsides` triaged and closed, so its `33` is no longer restated as an
open survivor count.
---

## FU-10: `mise.toml` Tera `{{arg()}}` templating is deprecated repo-wide

**Status:** open · Opened 2026-07-18 during the devkit Phase 3 cargo-mutants
slice final review.

**What:** `mise.toml`'s `[tasks.mutants-crate]` (`{{arg(name="crate")}}`) and
the pre-existing `[tasks.fuzz-target]` (`{{arg(name="target")}}`,
`{{arg(name="seconds")}}`) both use mise's Tera-based `{{arg(...)}}`
templating to accept positional task arguments. mise warns this form is
deprecated and will be **removed in mise 2027.5.0**, directing callers to use
the `usage` field instead.

**Where:** `mise.toml` lines ~122 (`[tasks.fuzz-target]`) and ~146
(`[tasks.mutants-crate]`).

**Evidence:** mise's own deprecation warning, surfaced when running either
task, names the removal version and the `usage`-field replacement.

**Impact:** No current defect — both tasks work as written today. But the
removal date is known and fixed, so both tasks will break with no warning
period once the repo's pinned mise version crosses 2027.5.0, unless migrated
first.

**Suggested fix:** Migrate both tasks' argument declarations from
`{{arg(...)}}` to the `usage` field, in one pass covering every task using the
deprecated form (not just the mutants one) so the repo doesn't end up with a
mix of old- and new-style argument declarations. Deferred here rather than
fixed opportunistically in this slice because migrating one task in isolation
while leaving `fuzz-target` on the old form would create exactly that
inconsistency.

**Severity:** low — maintenance (known removal date, no current breakage) ·
**Opened:** 2026-07-18

## FU-11: Sidereal-time consumers outside the house layer, and the ΔT extrapolation gap

**Status:** resolved — item 1 2026-09-27 (issue #74), item 2 2026-09-27 ·
Opened 2026-09-26 while fixing #56 (houses evaluated sidereal time at the TT
day).

**What:** #56 fixed `pleiades-houses` by routing every Earth-rotation quantity
through `pleiades_apparent::ut1_instant`. Two related items were deliberately
left out of that change:

1. **`pleiades-events` rise/set/transit and horizontal coordinates** label
   their working instants `TimeScale::Tdb` and call `sidereal_time` on the raw
   JD, so the returned event instants are UT1-scale under a `Tdb` label (the
   `validate-rise-trans` gate compares against `se_jd_ut` for exactly this
   reason, and the compatibility summary carries an honesty caveat). The
   occultation and local-eclipse paths already subtract ΔT but then tag the
   converted day `Tdb`, so a scale-aware `sidereal_time` would double-convert
   them. Making the events crate honour the instant tag means: converting via
   `ut1_instant` (or tagging the converted instants `Ut1`), switching the
   rise-trans gate to `se_jd_tdb`, and rewording the SP-2b caveat in
   `pleiades-core::compatibility`. That is a semantic change to returned event
   instants and belongs in its own reviewed change.
   → **Resolved 2026-09-27 (issue #74):** `pleiades-events` now reads the
   `TimeScale` tag on every rise/set/transit and horizontal query instant
   (`time_scale::tdb_jd`: TDB/TT as TDB, UT1/UTC + ΔT, other scales fail
   closed with `EventError::UnsupportedTimeScale`), samples bodies in TDB,
   evaluates sidereal time at the UT1 re-expression of each sampled day
   (`time_scale::local_apparent_sidereal_deg`), and returns genuine TDB
   instants tagged `Tdb`. The `validate-rise-trans` gate tags corpus UT days
   `Ut1` and compares against `se_jd_tdb` (all ceilings retained; the moon
   transit residual dropped from 2.89 s to 1.00 s), the SP-2b caveat is
   reworded, and `docs/time-observer-policy.md` documents the convention.
   The occultation and local-eclipse paths were left as they were (they
   already rotate with UT1); their `Tdb`-tagged converted instants are
   internal and never passed to a scale-aware adapter.

2. **ΔT model beyond 2020.** `crates/pleiades-time/data/delta-t-observed.csv`
   ends at 2020 (69.4 s) and `deltat::extrapolate` (Espenak–Meeus 2005–2050
   form) gives ≈75 s for 2026 against an observed ≈69 s. For post-1972 dates
   the leap-second table already pins TT − UTC exactly (32.184 s + TAI − UTC),
   so ΔT = TT − UTC − DUT1 is known to within |DUT1| < 0.9 s wherever the leap
   table applies — a far tighter bound than the polynomial. Until that is
   used, house angles for 2021+ carry a ≈6 s ≈ 0.025° residual against Swiss
   Ephemeris (documented in `docs/time-observer-policy.md`; the #56 regression
   test asserts a 0.05° ceiling on the 2026 Chennai fixture for this reason).
   → **Resolved 2026-09-27:** `pleiades_time::deltat::delta_t` now has three
   tiers. From the 2020 node while the leap-second table is authoritative it
   returns `32.184 s + (TAI − UTC)` tagged with the new
   `DeltaTQuality::LeapSecondBound` (DUT1 taken as zero, so within 0.9 s of
   truth; `ConversionQuality` reports it as `Observed` on the `Ut1DeltaT`
   path, leaving the three-tier report vocabulary unchanged). Beyond the leap
   horizon the Espenak–Meeus polynomial is anchored to the bound at the
   horizon so ΔT is continuous there, still tagged `Predicted`. The leap
   horizon moved from 2025-12-31 to 2026-06-30 on IERS Bulletin C 71; a
   test pins it, and each future Bulletin C should move it again
   (**2026-10-03, issue #105:** moved to 2027-07-01 00:00, exclusive, on
   Bulletin C 72; UTC past the horizon now holds the last `TAI − UTC`
   rather than following this polynomial, which remains the UT1 model). The
   observed table stays authoritative through 2020 (0.2 s step at the node,
   same order as its interpolation error). The 2026 Chennai ascendant
   residual against SE dropped from 0.026° to 0.0014° and the #56 regression
   ceiling tightened from 0.05° to 0.005°. A subtlety kept as a comment, not
   fixed: the ΔT lookup receives a TT-scale day on the `ut1_jd_from_tt`
   path while the leap lookup expects UTC; the 69 s difference only matters
   within 69 s of a leap epoch, where ΔT steps by a full second anyway.

**Severity:** resolved (both items) · **Opened:** 2026-09-26

## FU-12: Osculating true lunar node for `TrueNode` (issue #58)

**Status:** resolved (2026-09-26) · Implemented on `feat/true-node-osculating`,
merged as PR #62 (spec `docs/superpowers/specs/2026-09-26-true-node-osculating-design.md`).
`TrueNode` is now served release-grade by `PackagedDataBackend` as the
osculating ascending node of the geocentric lunar orbit (formed in the mean
ecliptic of date from the packaged Moon state, emitted in J2000; chart layer
applies precession + Δψ only, like the true apsides). Gated against the Swiss
Ephemeris 2.10.03 Moshier `SE_TRUE_NODE` corpus (3177 rows, 1900–2100, same
23-day grid as the Lilith corpus) by `validate-true-node`; gate parity as of
2026-09-26: max longitude residual 52.851″, latitude 6.3e-11″ (floating-point
noise; latitude is 0 by construction), distance 1.58e-4 relative, vs ceilings
80″/1″/2.38e-4. A blocking-tier regression test pins the channel to the 8 Moon
`SE_NODBIT_OSCU` rows of the nod-aps corpus at ≤40″. Topocentric charts leave
`TrueNode`/`TrueApogee`/`TruePerigee` geocentric: no diurnal parallax or
diurnal aberration is applied to a geometric orbit direction (matches Swiss
Ephemeris), fixing a pre-existing ~1° parallax shift of True Lilith in
topocentric charts.
· **Residual, documented not gated:** `ElpBackend`'s own `TrueNode` stays
Meeus's periodic-term-corrected mean node (±0.14° vs the osculating node) for
direct ELP consumers; its evidence rows cannot detect the gap because the 1913
sample is Meeus's own worked value. Routed charts requesting `TrueNode`
outside 1900–2100 now fail with `OutOfRangeInstant` instead of receiving the
ELP approximation (the router does not fall back on that kind; default
charts already fail there because the Moon does). · **Build-env note:** the
reference tool `tools/se-true-node-reference` builds inside `devenv shell`
(clang/libclang from `devenv.nix`); the gate reads the committed CSV and
never rebuilds the tool. · **Open question:** the gate comment attributes the
52.851″ maximum to Moshier-vs-DE440 amplified by 1/sin i; the zero-mean,
trend-free residual distribution supports that, but interpolation error in
the packaged Moon velocity has not been separated out (tracked as #65).
· **Follow-up issues:** #63 (comment documenting the topocentric exemption in
`chart/mod.rs`), #64 (enforce the validated-row floor inside the true-node and
Lilith gate functions, not only in the nightly test), #65 (residual
attribution above). · **Severity:** accuracy (now closed) · **Opened:**
2026-09-26 · **Merged:** PR #62 (2026-09-27)

## FU-13: Event surfaces outside rise/set/transit and chained searches (issues #80, #81)

**Status:** open · Opened 2026-09-29 while fixing #80 and #81.

**What:** #80/#81 made every instant returned by `pleiades-events`'
rise/set/transit searches "settled" (`root::bisect` returns the later end of
its final bracket, so the instant trails its event by less than the 0.5 s
tolerance and never precedes it) and anchored every such search at the query
instant, so that `next_rise_set` and `previous_rise_set` partition the event
sequence at any instant, including the ones the engine returns. Two
neighbouring surfaces were deliberately left out of that change:

1. **Longitude crossings, backward direction.** The shared `bisect` change
   fixes the forward chain (`next_longitude_crossing(after = c.instant)` no
   longer returns `c` again; pinned by
   `next_after_a_returned_crossing_is_the_following_one`). The backward
   searches still walk `root::last_crossing_before`'s window-anchored grid and
   decide "before" by comparing a refined root against the query instant, so
   `previous_longitude_crossing(before = c.instant)` returns either `c` or the
   crossing before it, as it did before the change. Its documented contract
   is "strictly before", which a sign-at-the-anchor rule would have to be
   reconciled with (at a settled instant the crossing has already happened).
   Documented on `previous_longitude_crossing`; callers step `before` back by
   a second. The same applies to `longitude_crossings_in_range`'s two ends.
2. **Occultations and `pleiades-eclipse` contacts.** `occult.rs` brackets
   each Moon-target conjunction with the shared `root` scanners (so that
   intermediate instant is now settled too) but refines contacts and maxima
   with its own midpoint bisection and golden-section searches;
   `pleiades-eclipse` (`syzygy.rs`, `local.rs`) carries its own
   midpoint-returning copies throughout. Whether a search chained from a
   returned contact or maximum can re-find or skip an event has NOT been
   audited; these events are weeks apart and the searches step from one
   conjunction to the next, so the exposure is likely smaller, but that is an
   expectation, not a measurement.

**Severity:** correctness at the tolerance boundary (consumer-visible only
when chaining searches) · **Opened:** 2026-09-29

---

## FU-14: Moon apparent-place residual against Horizons after the #93 aberration fix

**Status:** resolved (2026-09-30) — the goldens' epoch tag was the cause; both
regen scripts now pass `TIME_TYPE=TT`, both goldens files are regenerated, and the
tolerances are re-tightened (see **Resolution** below). The topocentric Moon at
2100 was probed and closed 2026-10-01 (see **Topocentric Moon at 2100** below).
· Opened 2026-09-30 while fixing #93.

**What:** #93 removed the separate annual-aberration term from the generic
light-time path; the planets' Horizons residuals collapsed to the ephemeris-fit
floor (Jupiter–Pluto under 0.5″). The Moon did not. Measured on the #93 branch
(`validate-apparent` diagnostic, geocentric, Horizons ObsEcLon Q31):

| JD (TT) | Moon residual vs Horizons |
|---|---|
| 2415025.5 | +1.095″ |
| 2433282.5 | −14.781″ |
| 2451545.0 | −32.100″ |
| 2469807.5 | −38.781″ |
| 2488065.5 | −38.704″ |

Against Swiss Ephemeris (`validate-crossings` Tier-2, `geo/Moon` group, engine
longitude at the SE crossing instant) the same code measures a maximum of
2.606″ (21.701″ before the fix). The Moon's equatorial-goldens Dec maximum rose
from 11.70″ to 12.37″ (jd 2469807.5) with the fix while every other body's
residual fell; the `validate-equatorial` Moon Dec tolerance was tightened to
14.4″ under #93. The tightened Moon tolerances are: apparent 40.8″, equatorial
RA 40.3″ / Dec 14.4″, topocentric longitude 16.7″ (each = measured max + 2″);
`validate-crossings` `GEO_MOON_ARCSEC` is 4″.

**Cause:** the reference epoch tag, not the engine or the ephemeris fit. The
Horizons goldens in `apparent-goldens.csv` and `equatorial-goldens.csv` were
fetched by `regen-apparent-goldens.sh` / `regen-equatorial-goldens.sh` without
`TIME_TYPE=TT`, so Horizons read `TLIST` as UT (its output header says
`Date_________JDUT`) while the engine evaluates the same rows as TT. The offset
is ΔT (about 64 s at J2000) times the Moon's rate of about 0.5″/s. Re-derived
2026-09-30 for the Moon (COMMAND 301, CENTER 500@399, QUANTITIES 31,
EXTRA_PREC=YES) by re-querying each epoch with `TIME_TYPE=TT`; engine = committed
golden + the signed residual above:

| JD | Committed golden (deg) | Horizons, TIME_TYPE=TT (deg) | UT−TT delta (golden − TT) | Engine − TT value |
|---|---|---|---|---|
| 2415025.5 | 345.7526277 | 345.7529518 | −1.167″ | −0.072″ |
| 2433282.5 | 61.4154091 | 61.4113240 | +14.706″ | −0.075″ |
| 2451545.0 | 223.3237860 | 223.3148557 | +32.149″ | +0.049″ |
| 2469807.5 | 18.6755948 | 18.6647918 | +38.891″ | +0.110″ |
| 2488065.5 | 102.2038630 | 102.1930835 | +38.806″ | +0.102″ |

Against TT-tagged Horizons the engine's Moon agrees to about 0.1″ at every
epoch. The same tag explains the floors of the other bodies: the UT−TT deltas
at J2000 for the Sun, Mercury, Venus and Mars are 2.93″ / 4.63″ / 3.62″ / 2.07″
against measured maxima of 2.83″ / 4.54″ / 3.67″ / 2.03″, and the Moon's
equatorial Dec rise from 11.70″ to 12.37″ is the Dec rate times ΔT. The
topocentric goldens DO pass `TIME_TYPE=TT` (Sun maximum 0.080″).

**Impact:** the engine matches TT-tagged Horizons to about 0.1″ for the Moon, so
there is no known Moon error in charts or `pleiades-events` surfaces from this
item. The cost is slack in the gates: the three Moon goldens tolerances (apparent
40.8″, equatorial RA 40.3″ / Dec 14.4″) and the Sun/Mercury/Venus/Mars ones
certify ΔT-sized slack until the goldens are regenerated, so a real Moon
regression of up to about 35″ would currently pass those three gates. The
`validate-crossings` `GEO_MOON_ARCSEC` gate (4″ against Swiss Ephemeris) still
bounds it. Separately recorded, not diagnosed: in `validate-occultations` the
metrics not gated by this change rose after #93 (`planet_mag_rel` 0.0489 to
0.0502, `sublunar` 20.2′ to 21.1′), both within their ceilings (0.07 / 30′).

**Resolution (2026-09-30):** `regen-apparent-goldens.sh` and
`regen-equatorial-goldens.sh` now pass `TIME_TYPE=TT` and refuse to write a
response whose header is not `Date_________JDTT`; both goldens files were
regenerated from Horizons (the Moon J2000 row became 223.3148557°, the value
re-derived above) and the pinned checksums updated. Measured against the
TT-tagged goldens, every body's maximum residual is at the packaged-ephemeris
fit floor, and every tolerance is the measured maximum + 2″ rounded up to 0.1″:

| Gate | Body | Before (max / tolerance) | After (max / tolerance) |
|---|---|---|---|
| `validate-apparent` | Moon | 38.781″ / 40.8″ | 0.109″ / 2.2″ |
| `validate-apparent` | Sun | 2.83″ / 5.0″ | 0.111″ / 2.2″ |
| `validate-apparent` | Mercury / Venus / Mars | 4.541″ / 3.670″ / 2.028″ (6.6″ / 5.7″ / 4.1″) | 0.114″ / 0.107″ / 0.166″ (2.2″ each) |
| `validate-apparent` | Jupiter–Pluto | 0.18–0.46″ / 2.2–2.5″ | 0.11–0.25″ / 2.2″ (Uranus 2.3″) |
| `validate-equatorial` RA | Moon | 38.283″ / 40.3″ | 0.102″ / 2.2″ |
| `validate-equatorial` RA | Sun | 2.825″ / 6.0″ floor | 0.111″ / 2.2″ |
| `validate-equatorial` RA | others | 0.17–4.53″ / 2.2–6.6″ | 0.10–0.25″ / 2.2″ (Uranus 2.3″) |
| `validate-equatorial` Dec | Moon | 12.370″ / 14.4″ | 0.046″ / 2.1″ |
| `validate-equatorial` Dec | others | ≤ 2.8″ measured / 2.1–6.0″ | 0.005–0.18″ / 2.1″ (Venus 2.2″) |

Gate output after the change: `validate-apparent` 50 rows, max residual 0.25″;
`validate-equatorial` 50 rows, max RA 0.25″ (cos δ-weighted), max Dec 0.18″. No
engine code changed, so `validate-crossings` and `validate-topocentric` are
unaffected. The README "Current state" row for apparent place now reads
sub-arcsecond.

**Topocentric Moon at 2100 (resolved 2026-10-01):** the topocentric Moon's
14.68″ at JD 2488065.5 in `topocentric-goldens.csv` (a TT-tagged file, so not
the cause above) was a disagreement between two ΔT extrapolations, not an engine
defect. The engine does convert TT to UT1 before sidereal time
(`chart/mod.rs`, `ut1_jd_from_tt`); at 2099-12-26 its predicted ΔT is 144.8 s,
while Horizons reports TDB−UT = 69.18 s there (its current value held forward).
The 75.6 s gap is 0.32° of Earth rotation. Probe: re-running each Moon row with
the observer's longitude shifted by the rotation equivalent to the gap, i.e.
feeding the engine Horizons' Earth orientation:

| JD (TT) | Engine ΔT | Horizons TDB−UT | Moon Δlon as-is | Moon Δlon at Horizons' value |
|---|---|---|---|---|
| 2415025.5 | −2.780 s | −1.929 s | +0.004″ | −0.069″ |
| 2433282.5 | 29.100 s | 28.931 s | −0.057″ | −0.083″ |
| 2451545.0 | 63.800 s | 64.184 s | +0.043″ | +0.086″ |
| 2488065.5 | 144.815 s | 69.184 s | +14.635″ | +0.076″ |

(Horizons' column is TDB−UT as printed; at the measured epochs the engine's ΔT
is the better UT1 value, which is why substituting does not help there.) Neither
extrapolation is knowable truth for 2100, so the row gated nothing about the
engine. The three 2099 rows were replaced with 2458849.5 (2020-01-01 TT, inside
measured ΔT) and the tolerances re-derived as measured maximum + 2″: Moon
0.124″ / 2.2″ (was 14.677″ / 16.7″), Sun 0.120″ / 2.2″, Mars 0.165″ / 2.2″.
Standing caveat: a topocentric Moon past the leap-second horizon carries the ΔT
prediction's uncertainty at about 0.2″ of longitude per second of ΔT (see FU-11
item 2); geocentric places do not.

**Origin:** issue #93, `docs/superpowers/specs/2026-09-30-apparent-aberration-double-count-design.md` section 4.

---

## FU-15: Mean lunar points on the packaged backend (issue #90)

**Status:** resolved (2026-10-01) · Spec
`docs/superpowers/specs/2026-10-01-mean-lunar-points-packaged-design.md`, plan
`docs/superpowers/plans/2026-10-01-mean-lunar-points-packaged.md`.

**What:** `PackagedDataBackend` did not serve `MeanNode`/`MeanPerigee`, so
`EventEngine::nod_aps(Moon, NodApsMethod::Mean)` failed on
`packaged_backend()`. Resolved by single-sourcing the mean lunar elements in
`pleiades-apsides`, having `nod_aps` use them directly, and serving
`MeanNode`/`MeanApogee`/`MeanPerigee` release-grade from the packaged backend
behind `validate-mean-lunar-points` (3177-row `SE_MEAN_NODE` / `SE_MEAN_APOG`
corpus, 1900–2100; measured maxima: 3177 rows validated (0 oor-skipped) vs Swiss Ephemeris SE_MEAN_NODE/SE_MEAN_APOG, node max lon 0.1337" lat 0.0000" dist 7.78e-11 rel; apogee max lon 0.5705" lat 0.0395" dist 1.02e-10 rel; perigee max lon 0.5705" lat 0.0395" dist 1.02e-10 rel).

**Behaviour change:** routed charts' `MeanApogee`/`MeanPerigee` moved from the
ELP element (latitude 0) to the Swiss Ephemeris point (up to about 7′ in
longitude, latitude up to 5.15°). The chart now treats all six lunar points as
geometric directions, which also closes #63. Because the mean lunar points are
now release-grade, an Apparent chart reports them in the true ecliptic of date
(precession plus nutation in longitude), as Swiss Ephemeris does; previously
`ElpBackend` served them under a `constrained` claim and an Apparent chart
returned them un-rotated (J2000 mean ecliptic, no apparent provenance).
`MeanNode` at JD 2461041.5 moved from 341.806020° to 342.170759° (about +0.36°,
1313″, in 2026, growing with distance from J2000). Separately, removing the
light-time path moved the packaged points by at most 0.0063″ (`MeanNode`
−0.0028″). In a routed chain with `PackagedDataBackend` first, the mean lunar points are
now served only inside the packaged window (1900–2100); a chart requesting them
outside it returns an out-of-range error instead of the `ElpBackend` element it
previously fell back to (the same fail-closed behaviour as `TrueNode` and the
packaged planets); direct `ElpBackend` consumers are unaffected.

**Build-env note:** `tools/se-mean-lunar-reference` builds inside
`devenv shell` (libclang), like the other `se-*-reference` tools; it is not
needed to run the gate.

**Severity:** feature gap (closed) · **Opened:** 2026-10-01

## FU-16: A public ecliptic position with latitude and speed (issue #89)

**Status:** resolved (2026-10-01) · Spec
`docs/superpowers/specs/2026-10-01-heliocentric-position-design.md`, plan
`docs/superpowers/plans/2026-10-01-heliocentric-position.md`.

**What:** the only public heliocentric read was `EventEngine::longitude_at`,
which returned a longitude alone. `EventEngine::position_at` now returns
longitude, latitude, distance and their speeds in either `CrossingFrame`, with
a longitude bit-identical to `longitude_at`. The speed is the backend's speed
plus the rate of the frame or apparent-place correction, differenced over
±0.5 day, using the helper shared with the chart layer
(`pleiades_apparent::motion`).

**Gate:** `validate-helio-position` (25424-row Swiss Ephemeris
`SEFLG_MOSEPH | SEFLG_HELCTR | SEFLG_TRUEPOS | SEFLG_SPEED` corpus,
Mercury–Pluto, 1900–2100). Both sides are the geometric heliocentric place, so
the residual is the Moshier-vs-DE440 ephemeris difference. Measured
(2026-10-01, final review):

```text
Helio-position gate: 25424 rows validated (0 oor-skipped) vs Swiss Ephemeris SEFLG_HELCTR|SEFLG_TRUEPOS|SEFLG_SPEED, Mercury-Neptune max lon 2.294" lat 0.237" dist 3.10e-6 rel, speed lon 0.0795"/d lat 0.0898"/d dist 1.86e-7 AU/d; Pluto max lon 1.194" lat 0.606" dist 4.27e-6 rel, speed lon 0.0300"/d lat 0.0869"/d dist 1.96e-7 AU/d; outer-planet mean signed lon speed +0.0002"/d
```

The gate also fails closed when the mean signed Jupiter–Neptune longitude-speed
residual reaches ±0.02″/day (≈ ±0.137″/day would mean the two sides disagree
on whether the speed includes the precession rate). The geocentric frame is
pinned to the apparent chart placement by
`crates/pleiades-events/tests/position.rs`.

**Corpus history:** the first corpus used `SEFLG_HELCTR | SEFLG_SPEED` without
`SEFLG_TRUEPOS`, which retards the planet by the heliocentric light-time r/c;
its 40.8″ Mercury longitude maximum was that retardation, not a pleiades
defect, and was misattributed to the planet-minus-Sun reconstruction.

**Latitude-speed artefact (not modelled):** for Mars–Pluto the latitude-speed
residual is a near-uniform ≈ 0.085″/day that regresses on dΔε/dt · sin λ
(slope −1.001, 99.7 % of the variance; ≤ 0.0065″/day remains after removing
it). Swiss Ephemeris' latitude speed carries a nutation-in-obliquity rate term
that is not the derivative of its own latitude; pleiades' latitude speed
matches the central difference of its own latitude to < 1e-5″/day. The gate's
latitude-speed ceilings absorb the term deliberately.

**Also in this change:** toolchain and MSRV moved from Rust 1.98.1 to 1.99.0.

## FU-17: Heliocentric follow-ups from the issue #89 final review

**Status:** open · **Opened:** 2026-10-01

Three items the final review of issue #89 found outside that change's scope:

- **(a) `validate-crossings` Tier-2 heliocentric reference.** The Tier-2
  heliocentric Swiss Ephemeris reference and
  `crates/pleiades-events/tests/heliocentric.rs` carry the same
  misattribution FU-16 corrected: their reference lacks `SEFLG_TRUEPOS`, so it
  is light-time-retarded, and that — not the planet-minus-Sun reconstruction —
  is the real source of the 35.09″ maximum. Regenerate the reference with
  `SEFLG_TRUEPOS` and tighten the ceilings in a separate change.
- **(b) Heliocentric lunar points.** Heliocentric `longitude_at` /
  `position_at` return `Ok` for `MeanNode`, `TrueNode`, `MeanApogee` and
  `TrueApogee`, although "node minus Sun" is meaningless. Rejecting them
  changes `longitude_at`'s public behaviour and needs its own decision.
- **(c) Geocentric reads near the range start.** Geocentric reads within a
  light-time of the packaged range start (for example Mars or the Moon at
  JD 2415020.5) fail with `Backend(OutOfRangeInstant)` although the instant is
  inside the documented window.

**Severity:** (a) validation accuracy, (b) API correctness, (c) edge-case
availability

---

## FU-18: Sidereal conventions differ from Swiss Ephemeris and the chart layer (issue #88)

**Status:** open · **Opened:** 2026-10-01

Issue #88 gave `pleiades-events` sidereal crossings: the longitude on the mean
equinox of date minus the mean ayanamsa. The chart layer (`pleiades-core`
`src/chart/sidereal.rs`) subtracts the same mean ayanamsa from whatever
longitude the chart holds, which gives two chart-layer differences, (a) and
(b). Both were measured for the Sun with Lahiri by
`measure_chart_sidereal_conventions` in
`crates/pleiades-events/tests/reference.rs`. A third difference, (c), is with
Swiss Ephemeris itself.

- **(a) A sidereal apparent chart keeps nutation.** The chart longitude is on
  the true equinox, so it exceeds the crossing engine's sidereal longitude by
  Δψ: measured 2.240″ (JD 2420000.5), −13.924″ (JD 2451545.0), −9.279″
  (JD 2460000.5) and −15.740″ (JD 2480000.5), equal to Δψ at each epoch.
  Swiss Ephemeris drops nutation from sidereal positions.
- **(b) A sidereal mean chart mixes frames.** A mean chart's longitude is the
  backend's J2000 place, and the of-date ayanamsa is subtracted from it:
  measured 4342.509″ (JD 2420000.5), 0.000″ (JD 2451545.0), −1164.290″
  (JD 2460000.5) and −3918.689″ (JD 2480000.5) against the mean-of-date
  sidereal longitude, which is the precession accumulated since J2000.
- **(c) Star-anchored ayanamsas in the apparent frame differ from Swiss
  Ephemeris's default.** For True Citra (anchored on Spica) and Galactic
  Center (anchored on Sgr A*), Swiss Ephemeris under plain `SEFLG_SIDEREAL`
  computes the ayanamsa from the anchoring star's place under the same flags
  as the body. With apparent flags the ayanamsa therefore carries the star's
  annual aberration, up to about 20″ and varying through the year. pleiades
  subtracts the mean ayanamsa (`pleiades_ayanamsa::sidereal_offset`) in every
  frame. Measured in the reference tool, Swiss Ephemeris's ayanamsa under
  apparent flags minus under `TRUEPOS|NOABERR|NOGDEFL` was 20.451″
  (TrueCitra, JD 2470276.058) and −19.737″ (GalacticCenter, JD 2480002.727);
  before the corpus reference was changed, the corresponding rows disagreed
  by 20.615″ and 20.101″. The corpus's `geo`-frame TrueCitra and
  GalacticCenter rows therefore use Swiss Ephemeris's mean ayanamsa. Matching
  its default would need an apparent-star ayanamsa in `pleiades-ayanamsa`.
  The affected set is every ayanamsa in the `TrueStar` and `Galactic`
  computation classes, not only these two; True Citra and Galactic Center are
  the ones measured, the rest follow from the same mechanism. About 20″ is
  roughly 8 minutes of crossing time for the Sun and hours for a slow planet
  such as Saturn, more near a station. Open work: measure the remaining
  `TrueStar` and `Galactic` ayanamsas with the reference tool, and decide
  whether to offer an opt-in apparent-star ayanamsa for Swiss Ephemeris
  default parity.
- **(d) Sidereal corpus coverage is thin.** The sidereal corpus groups use two
  start epochs for Sun and Moon and one for Jupiter (the spec asked for
  three). The sidereal Moon ceiling (1″) rests on 16 rows (2 per ayanamsa and
  frame), while the tropical Moon groups measure 2.6″, so a regeneration
  with more Moon rows should expect a ceiling near 4″.

Fixing (a) or (b) changes chart output and needs its own decision, a
regenerated chart golden and a gate against a Swiss Ephemeris sidereal
position corpus.

**Severity:** (a) convention, up to about 17″; (b) frame correctness, growing
with distance from J2000; (c) convention, up to about 20″, star-anchored
ayanamsas only

## FU-19: Civil datetime from a TT or TDB instant (issue #87)

**Status:** resolved (2026-10-02) · Spec
`docs/superpowers/specs/2026-10-02-civil-from-terrestrial-design.md`, plan
`docs/superpowers/plans/2026-10-02-civil-from-terrestrial.md`.

**What:** `pleiades-time` converted civil UTC/UT1 to TT/TDB but not back, so
callers of the event finders had to re-implement the leap-second table and
Delta-T. Resolved by `from_terrestrial` (and four scale-checked conveniences),
which quantizes to the millisecond and looks up leap seconds on the TAI axis.
The same change fixed the forward conversion of an inserted leap second:
`23:59:60.x` aliased the next day's `00:00:00.x` in the Julian day and landed
one second late in TT. `:60` is now accepted only at a real insertion.

**Deferred:**

- A civil-time convenience on `pleiades-events` results and CLI output of
  civil times for events. Callers pass the returned `Instant` to
  `from_terrestrial`.
- The 0.216 s Delta-T step at the 2020 node is unchanged; the UT1 inverse
  picks the post-node branch there.
- Sub-millisecond precision would need a (day, seconds-of-day) representation
  in both directions; a Julian day near 2.46e6 resolves about 40 µs.
- Known edge, kept: the forward accepts `2100-12-31T23:59:59.9996` (UTC or
  UT1), but its TT rounds to the millisecond of `2101-01-01T00:00:00.000`, so
  `from_terrestrial` returns `BeyondHorizon` for it. The window check applies
  after rounding and is not clamped.

## FU-20: READMEs described an older release than the one published (issue #86)

**Status:** resolved (2026-10-02) · Bounded change, no spec or plan document.

**What:** the workspace README listed eleven published crates at `0.2.x` and
called `pleiades-data` unpublished, when sixteen were published at 0.5.3 and
0.6.0; eleven crate READMEs carried the same `0.2.x` status line, and the
`pleiades-core` and `pleiades-data` READMEs denied capabilities the crates
have (apparent and topocentric charts, the full packaged body set). The text
is corrected and no README names a version any more.

**Guard:** `workspace-audit` (`mise run audit`, blocking tier) now fails when
the workspace README's marked published or unpublished crate list differs from
the manifests' `publish` settings, or when any README pins a release series
such as `0.2.x` (`crates/pleiades-validate/src/release/readme_audit.rs`).

**Deferred:**

- Capability prose in the READMEs is still hand-written; the guard covers the
  crate lists and version literals only.
- The crate list in `mise.toml`'s `package-check` task is a third hand-kept
  copy of the publishable set and is not covered by the guard.
- The published READMEs on crates.io change with the next release-plz release
  of each crate.

---

## FU-21: Planetary station finder (issue #85)

**Status:** resolved (2026-10-02) · Spec
`docs/superpowers/specs/2026-10-02-planetary-stations-design.md`, plan
`docs/superpowers/plans/2026-10-02-planetary-stations.md`.

`EventEngine::stations_in_range` and `EventEngine::next_station`
(`pleiades-events` `src/stations.rs`) find sign changes of the longitude speed
`position_at` reports. `EventError` gained `MissingSpeed` and is now
`#[non_exhaustive]` (breaking).

**Gate:** `validate-stations` (6576-row Swiss Ephemeris speed-zero corpus,
`tools/se-stations-reference`). Measured 2026-10-02:

```
Stations gate: 5542 stations validated across 15 series vs Swiss Ephemeris speed-zero corpus (planets station-for-station; true node on stations separated by >= 3 d), max time 117467.6 s, max lon 51.338"
geo Mercury: 1260 compared (engine 1260, corpus 1260), max time 9.3 s, mean signed time -4.2 s, max lon 0.381"
geo Venus: 250 compared (engine 250, corpus 250), max time 21.5 s, mean signed time -1.3 s, max lon 0.542"
geo Mars: 188 compared (engine 188, corpus 188), max time 54.9 s, mean signed time -5.0 s, max lon 1.119"
geo Jupiter: 366 compared (engine 366, corpus 366), max time 101.0 s, mean signed time -5.8 s, max lon 0.695"
geo Saturn: 386 compared (engine 386, corpus 386), max time 165.1 s, mean signed time -2.8 s, max lon 0.638"
geo Uranus: 396 compared (engine 396, corpus 396), max time 318.7 s, mean signed time -4.5 s, max lon 0.505"
geo Neptune: 398 compared (engine 398, corpus 398), max time 495.4 s, mean signed time +11.0 s, max lon 2.314"
geo Pluto: 398 compared (engine 398, corpus 398), max time 1292.7 s, mean signed time -56.6 s, max lon 1.255"
geo TrueNode: 1166 compared (engine 2002, corpus 2200), max time 117467.6 s, mean signed time +446.4 s, max lon 51.338"
mean Mercury: 252 compared (engine 252, corpus 252), max time 6.4 s, mean signed time -4.0 s, max lon 0.183"
mean Mars: 38 compared (engine 38, corpus 38), max time 28.8 s, mean signed time -5.5 s, max lon 0.638"
mean Saturn: 77 compared (engine 77, corpus 77), max time 30.1 s, mean signed time -3.4 s, max lon 0.554"
sid Mercury: 252 compared (engine 252, corpus 252), max time 6.5 s, mean signed time -4.1 s, max lon 0.312"
sid Mars: 38 compared (engine 38, corpus 38), max time 28.9 s, mean signed time -5.5 s, max lon 0.769"
sid Saturn: 77 compared (engine 77, corpus 77), max time 30.4 s, mean signed time -3.5 s, max lon 0.679"
```

The gate runs in two tiers (2026-10-02):

- **Release battery** (`run_all_numeric_gates`: blocking `release-smoke`, and
  the battery tests in nightly `test-full`): `validate_stations_corpus_subset`
  verifies the whole corpus's checksum and row count, then compares the
  `mean` and `sid` series only (Mercury, Mars, Saturn over 1990–2030; 734
  stations, floor 734), about 24 s in the dev profile. Its summary line reads
  `Stations gate (mean/sid subset): 734 stations validated across 6 series`.
  `release-smoke` went from 553 s to 150 s with this change.
- **Full gate** (`validate-stations`, and the
  `stations_gate_passes_within_ceilings` test in nightly `test-full`, which
  `release-gate` depends on): all 15 series, 5542 stations, 173 s in release
  and 339 s in the dev profile.

**Open items:**

- **(a) The true node grazes zero.** Its speed touches zero about every two
  weeks; over two years the engine finds 96, 98, 98 and 102 stations at steps
  of 0.5, 0.25, 0.1 and 0.02 day. Pairs closer than the 0.25-day step are not
  reported, and the gate compares only stations at least 3 days from their
  neighbours. The separation was raised from the design's 2 days to 3 days by
  measurement, and the true-node comparison is an existence-and-kind check
  with a window of about three days; a tighter true-node comparison needs a
  different metric. A concrete one: a distribution check on the compared
  stations, e.g. "at least 90 % of compared true-node stations within 0.5 d
  of their counterpart" (measured 2026-10-02: 1130 of 1166, 96.9 %). It would
  catch a regression that moves every true-node station by a day, which the
  present three-day existence window does not. Not implemented. A caller who
  needs every graze has no way to ask for a finer step.
- **(b) `previous_station`** is not provided; it would inherit FU-13's
  backward-search caveat.
- **(c) No user-facing CLI stations command.**
- **(d) Ungated bodies.** Asteroids, fictitious bodies and the osculating
  apogee are accepted by the finders but have no reference corpus (FU-7
  records asteroid speed defects).
- **(e) Cost for bodies that never station.** `next_station` scans to the end
  of the window before returning `None` (about 0.3 ms per step).
- **(f) Swiss Ephemeris speed convention.** Moshier planet speed is a
  backward difference over `PLAN_SPEED_INTV` = 0.0001 day (`swemplan.c`), so
  its speed zero lands 4.32 s late. That convention explains Mercury's
  engine-minus-corpus mean of about -4.2 s (99.5 % of stations negative, both
  kinds); it is included in the measured maxima and is not an engine defect.
  For the slower planets the mean is dominated by scatter. **Open
  observation:** Pluto's mean is -56.6 s (turning retrograde about -65 s,
  turning direct about -48 s in the probe), which the 4.32 s convention does
  not explain. It is well inside Pluto's 2000 s time ceiling and has not been
  investigated.
- **(g) Reference-tool NaN.** Swiss Ephemeris Moshier returns a NaN true-node
  longitude speed at isolated grid instants (jd_tt 2451544.9 and 2451545.1);
  `tools/se-stations-reference` skips an isolated non-finite grid sample
  (logged to stderr) and aborts on anything else non-finite. No station lies
  near them.

**Severity:** (a) documented limit, (b)–(d) feature gaps, (e) performance,
(f)–(g) documented notes · **Opened:** 2026-10-02

---

## FU-22: Exact-aspect event finder (issue #84)

**Status:** resolved (2026-10-02; full gate measured 1110.8 s as a test, 880 s
as the command) · Spec
`docs/superpowers/specs/2026-10-02-aspect-exactness-design.md`, plan
`docs/superpowers/plans/2026-10-02-aspect-exactness.md`.

`EventEngine::aspects_in_range` and `EventEngine::next_aspect`
(`pleiades-events` `src/aspects.rs`) find the instants the ecliptic
separation of two bodies equals an unsigned angle, on both sides. The scan
(`root.rs` `level_crossings_in_range`) splits each step at the turning points
of the separation. `EventError` gained `InvalidAspect` (not breaking).

**Gate:** `validate-aspects` (10359-row Swiss Ephemeris exact-aspect corpus,
`tools/se-aspects-reference`). Measured 2026-10-02:

```
Aspects gate: 10359 exact aspects validated across 11 pairs vs Swiss Ephemeris corpus (event for event at 0, 60, 90, 120 and 180 degrees), max separation residual 2.894", max time 744.8 s, max lon 2.324"
geo Sun-Moon: 3959 compared (0°: 494, 60°: 990, 90°: 990, 120°: 990, 180°: 495), max sep 2.894", max time 5.2 s, mean signed time +0.5 s, max lon 0.394"
geo Sun-Mercury: 1261 compared (0°: 1261, 60°: 0, 90°: 0, 120°: 0, 180°: 0), max sep 0.146", max time 2.4 s, mean signed time +0.3 s, max lon 0.400"
geo Mercury-Venus: 972 compared (0°: 496, 60°: 476, 90°: 0, 120°: 0, 180°: 0), max sep 0.473", max time 101.7 s, mean signed time +0.8 s, max lon 2.324"
geo Venus-Mars: 1046 compared (0°: 167, 60°: 281, 90°: 268, 120°: 236, 180°: 94), max sep 0.958", max time 44.5 s, mean signed time +1.6 s, max lon 1.489"
geo Mars-Jupiter: 842 compared (0°: 91, 60°: 209, 90°: 209, 120°: 201, 180°: 132), max sep 2.211", max time 177.7 s, mean signed time -0.7 s, max lon 0.801"
geo Mars-Saturn: 872 compared (0°: 101, 60°: 213, 90°: 219, 120°: 223, 180°: 116), max sep 1.114", max time 70.6 s, mean signed time +1.6 s, max lon 1.075"
geo Jupiter-Saturn: 210 compared (0°: 14, 60°: 42, 90°: 52, 120°: 66, 180°: 36), max sep 1.119", max time 533.3 s, mean signed time +0.5 s, max lon 2.231"
geo Saturn-Pluto: 109 compared (0°: 10, 60°: 28, 90°: 26, 120°: 30, 180°: 15), max sep 1.565", max time 744.8 s, mean signed time -28.5 s, max lon 1.662"
mean Mercury-Venus: 196 compared (0°: 100, 60°: 96, 90°: 0, 120°: 0, 180°: 0), max sep 0.384", max time 102.0 s, mean signed time +0.8 s, max lon 2.319"
mean Mars-Saturn: 176 compared (0°: 20, 60°: 43, 90°: 43, 120°: 46, 180°: 24), max sep 0.825", max time 70.0 s, mean signed time +1.0 s, max lon 0.639"
helio Mars-Jupiter: 716 compared (0°: 89, 60°: 179, 90°: 179, 120°: 179, 180°: 90), max sep 0.530", max time 34.4 s, mean signed time -1.1 s, max lon 0.747"
```

The gate runs in two tiers:

- **Release battery** (`run_all_numeric_gates`: blocking `release-smoke`, and
  the battery tests in nightly `test-full`): `validate_aspects_corpus_subset`
  verifies the whole corpus's checksum and row count, then compares the
  `mean` group only (372 events), about 16.3 s in the dev profile.
  `release-smoke` went from 181 s to 198 s with this change.
- **Full gate** (all 11 pairs, 10359 events): `mise run gate-aspects` (the
  `validate-aspects` command, dev profile, 880 s wall), in its own nightly job
  `aspects-gate` (`.github/workflows/nightly.yml`, 90-minute cap, own
  failure-issue template), and a `release-gate` dependency. It is not in
  nightly `test-full`: the in-crate test `aspects_gate_passes_within_ceilings`
  took 1110.8 s in the test profile, so it runs only with
  `PLEIADES_FULL_ASPECTS_GATE=1`. No release-profile time was measured.

**Open items:**

- **(a) The turning-point split is untested by the corpus.** Over the eight
  geocentric corpus pairs and 200 years, a plain sign-change scan and the
  turning-point scan return the same events; the closest two events are 1.59
  days apart (Mercury–Venus conjunctions) against a 1-day step. The split is
  covered by the synthetic `root::level_tests` only.
- **(b) Two turning points within two steps of each other** may go unseen,
  and with them a pair of exact moments between them. No planetary pair does
  this; an oscillating lunar point (true node, osculating apogee) paired with
  a slow body might.
- **(c) No graze in the corpus.** The reference tool fails if a separation
  turns within 30″ of an angle, so a near-tangent pair of events, whose
  existence would depend on the ephemeris, is never compared. The closest
  turn in the corpus pairs is 102″ (Venus–Mars at 60°). Adding a pair or an
  angle that grazes needs a gate rule for it first.
- **(d) `previous_aspect`** is not provided; it would inherit FU-13's
  backward-search caveat.
- **(e) No user-facing CLI aspects command**, and no batch call over several
  angles or pairs. The gate scans each pair once per angle for that reason.
- **(f) Ungated bodies.** The lunar points, asteroids and fictitious bodies
  are accepted by the finders but have no reference corpus.
- **(g) Cost for a pair that never reaches the angle.** `next_aspect` scans to
  the end of the window before returning `None` (about 0.15 ms per step).

**Severity:** (a)–(c) documented limits, (d)–(f) feature gaps, (g)
performance · **Opened:** 2026-10-02

---

## FU-23: Remaining `test-full` / nightly wall-clock

**Status:** partly resolved (2026-10-04) · Items (a) to (f) are done and (g)
is measured; (h) and (i) remain open. Measurements, the two changes from
#112 and the 2026-10-03 and 2026-10-04 changes are in
`docs/superpowers/plans/test-timings.md` (Section 0).

**Where the time is:** nightly run 37119694132 (4-core GitHub runner) spent
963 s in `ci-nightly`: 220 s building for `test`, 61 s running it, then
680 s in `test-full` itself (50 s rebuild, 444 s `pleiades-validate` lib suite,
154 s `pleiades-cli` ignored tests, 31 s `pleiades-data` ignored tests).

**Open items**, ranked by estimated saving against risk. All savings are
estimates from that run's timestamps, not measurements of a fix.

- **(a) `release-gate` runs the smoke battery twice.** The `release-smoke`
  dependency and the task's own `release-gate` command both call
  `validate_release_smoke_at` (numeric battery, bundle render and verify).
  Dropping the dependency from `release-gate` (not from `ci`) removes one
  run: about 100 s on a CI runner, several minutes on a loaded machine. It
  changes the release procedure, so it was left for a maintainer decision.
  → **Resolved 2026-10-04:** `release-smoke` is no longer a `release-gate`
  dependency; the task comment in `mise.toml` and the README say why. `ci`
  still depends on it, so the blocking tier's smoke run is unchanged.
- **(b) Workspace crates recompile on every CI run.** The `target` cache
  restores, but a fresh checkout gives every source file a new mtime, so
  cargo rebuilds all first-party crates: 220 s in the nightly `test` step,
  and the same in blocking CI. Restoring mtimes from git history, or a
  content-hash fingerprint once cargo offers one on stable, could save up to
  about 3 minutes per run. Risk: a wrong mtime restore yields stale builds.
  → **Resolved 2026-10-03 (#117):** `.github/scripts/cargo-cache-mtimes.sh`
  runs after the cache restore in all three jobs. It does not restore
  history; it floors every tracked file's mtime and touches back only the
  files whose content differs from the commit the cached `target` was built
  from (a marker the script keeps inside `target`), so a stale build is
  impossible by construction and a missing or unreachable marker leaves the
  run as it was. `target` is now its own cache entry, rolled forward every
  non-`pull_request` run, because `actions/cache` never overwrites a key.
  Measured on the blocking job with nothing changed since the cached
  build: 0 first-party crates compiled, `mise run ci` 331 s → 159 s, job
  405 s → 196 s. Mechanism and the full table in the timings plan.
- **(c) `summary_commands_render_compact_reports` is one 153 s test.** It
  makes 485 `render_cli` calls on one thread and alone sets the length of
  `test-full`'s second step while the other cores sit idle for about 110 s.
  Splitting it into several tests by command family would save about 100 s.
  Mechanical, but a 3000-line diff in `pleiades-cli`. → **Resolved
  2026-10-03:** split into 13 `#[ignore]`d tests, one per command family
  (same 483 `render_cli` calls and 816 assertions, moved verbatim). The
  release family also stops running `release-gate-summary` in full, see (d).
- **(d) Command and alias tests re-run whole gates.** In
  `pleiades-validate`'s lib suite the crossings gate runs five times, the
  occultations gate five times, and
  `release_gate_command_aliases_the_release_checklist` plus
  `release_smoke_command_renders_the_smoke_report` each run the battery and
  a bundle again (154 s and 96 s). Sharing a per-process outcome where the
  test's subject is the report rather than the dispatch would save roughly
  250–300 thread-seconds, about 60–75 s of wall-clock on four cores. Each
  case needs a check that the alias dispatch is still exercised. →
  **Resolved 2026-10-03:** per-process `OnceLock` outcomes in
  `tests::test_support` (`crossings_gate_report`, `occultations_gate_report`,
  the `*_via_cli` dispatches and `release_gate_via_cli`). Runs per test
  process: crossings 6 → 3, occultations 7 → 3, release battery plus bundle
  3 → 1 (one of each is the battery's own run). Aliases are proven to share
  the primary's match arm by their extra-argument error, which names the
  primary command; `release-gate-summary` and `release-smoke` assert the
  shared gate run plus their pure renderers, and `release-smoke` still runs
  end to end in the blocking `ci` tier.
- **(e) The full stations gate is a 253 s single-threaded test** and becomes
  the suite's long pole. `validate_scoped` loops over independent series, so
  `std::thread::scope` per series would cut it to roughly 70 s on four
  cores. It saves wall-clock only while other cores are idle (perhaps
  30–60 s), and it changes gate code, so report ordering must stay
  deterministic. → **Resolved 2026-10-03:** `compare_series` runs under
  `std::thread::scope`, one thread per series, folded in corpus order; the
  report lines are byte-identical to the sequential code. Locally (24
  cores) 343 s → 175 s. The floor is now the longest single series, so a
  4-core runner should land near that 175 s rather than the 70 s estimated
  above; splitting the long series into time chunks would be the next step
  if the gate is still the long pole. → **Chunked 2026-10-04:** every series
  scans in ten-year windows (`CHUNK_DAYS`, a multiple of every engine step,
  so a window brackets exactly as the single scan does and
  `join_chunks` keeps a boundary station once) on a pool of one thread per
  core. Full gate under a 4-core affinity mask: 228 s → 130 s, report lines
  byte-identical; `chunked_scan_matches_the_single_scan_at_every_seam` pins
  the seam rule bit for bit over 292 seams.
- **(f) Run `test-full`'s two steps concurrently.** At most about 80 s, at
  the price of eight test threads on four cores. → **Resolved 2026-10-03:**
  `test-full` now depends on two tasks, `test-full-validate` and
  `test-full-ignored`, which mise runs in parallel after `test` and
  `doctest`; cargo serializes their builds on the build-directory lock, the
  test runs overlap.
- **(g) Build profile.** `dev` and `test` are already `opt-level = 2`, so
  the numeric crates are optimized. Untried: `debug = "line-tables-only"` to
  shorten compile and link, and `opt-level = 3` for the gate-heavy crates.
  Neither was measured. → **Measured 2026-10-03, nothing applied:** clean
  `cargo test -p pleiades-validate --no-run` on 24 cores took 112 s and
  112 s at the committed profile and 119 s with `debug = "line-tables-only"`
  (a 227 s first run was the cold dependency cache), so the debug-info
  setting does not pay. `[profile.test] opt-level = 3` for every crate ran
  the parallel full stations gate test in 180 s against 173 s at
  `opt-level = 2` (and rebuilt in 75 s against 47 s), so it does not pay
  either. Details in the timings plan.
- **(h) Splitting the nightly into parallel jobs** gains little: the
  non-test tasks already run beside the tests inside `mise`, and each extra
  job pays its own cache restore and build.
- **(i) Returning the slow families to nextest** with an on-disk fixture
  cache keyed by content is the largest change and is not needed while
  libtest keeps the tier inside its budget.

**Severity:** performance (developer and CI time) · **Opened:** 2026-10-03

---

## FU-24: A placement whose apparent reduction fails is reported under the snapshot's `Apparent` label

**Status:** open · Surfaced while fixing issue #113 (apparent place on every
claim tier). The common case the issue reported, every non-`ReleaseGrade` body
returned in mean J2000 under an `Apparent` label, is gone: the chart layer now
reduces every body the backend serves, and a backend that cannot serve the Sun
fails closed. This entry tracks the remaining, narrower case.

**Where:** `crates/pleiades-core/src/chart/mod.rs`, the `Err(_)` arm of the
per-body `apparent_place` match in `ChartEngine::chart`.

**What:** when the reduction fails for one body (no `distance_au` on the mean
result, a light-time retarded epoch outside the backend's range, the light-time
sanity cap), the engine keeps that body's mean J2000 place, sets
`position.apparent = Mean` and attaches no provenance, but
`ChartSnapshot::apparentness` still echoes the requested `Apparent`. A caller
reading only the snapshot-level field sees a chart it believes is of date with
one body about 50″ per year from J2000 adrift from the rest. The test
`release_grade_body_falls_back_to_mean_when_apparent_unavailable` pins the
graceful fallback deliberately (the 433-Eros unreliable-distance scenario), so
this is a design decision to revisit, not a regression.

**Suggested fix:** either (a) fail closed per body, as the original
2026-06-22 apparent-place-corrections design specified, with the Eros distance
channel fixed at its source; or (b) keep the fallback but make it visible at
the snapshot level: a derived `ChartSnapshot::apparentness_applied()` (or a
rendered "n of m placements reduced" line) so `summary_line()` and `Display`
cannot read as fully apparent when a placement is not. Either way the
per-placement truth (`position.apparent`, `BodyPlacement::apparent`) is
already correct and should stay the source of record.

**Severity:** honesty of reported output (narrow case) · **Opened:** 2026-10-03
