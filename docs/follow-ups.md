# Follow-ups / deferred items

Tracked engineering items deferred out of the scope that surfaced them. Each
entry: what, where, evidence, impact, suggested fix, and origin.

---

## FU-2: True (osculating) lunar apsides sub-project

**Status:** resolved (2026-06-30) · Implemented by `feat/true-lilith-osculating-apsides` branch (Tasks 1–8). `TrueApogee` and `TruePerigee` are now served release-grade by `PackagedDataBackend` via the `crates/pleiades-apsides` crate (osculating Kepler apse from Moon pos+vel+mu). Gated against Swiss Ephemeris `SE_OSCU_APOG` Moshier corpus (3177 samples, 1900–2100) by `validate-lilith`; gate parity as of 2026-06-30: max longitude residual ~306″ (~5.1′), latitude ~53″, distance ~1.6e-4 relative, vs ceilings 460″/80″/2.34e-4. Of-date frame = true ecliptic of date via precession + nutation-in-longitude only (no light-time, no aberration — geometric direction). · **Next queued:** equatorial/declination output for `TrueApogee`/`TruePerigee` (chart-layer apparent equatorial shipped 2026-06-30 on `feat/equatorial-declination-output` for release-grade bodies; apsides equatorial follows when their release-grade status expands). · **Build-env note:** the reference tool `tools/se-lilith-reference` (used to generate the committed SE_OSCU_APOG corpus CSV) requires `libclang-dev` + `LIBCLANG_PATH` to build Rust bindings to the vendored Swiss Ephemeris. This is NOT required to run the `validate-lilith` gate or build the workspace — the gate reads the committed corpus CSV via `include_str!` and never rebuilds the tool. · **Severity:** feature gap (now closed) · **Opened:** 2026-06-30

---

## FU-1: Latent geocentric-Sun aberration double-count in `pleiades-core` apparent path

**Status:** resolved (2026-06-30) · Fixed by `apparent_sun_position` in pleiades-apparent (cc575c04); chart Sun path applies aberration once (a6113705); eclipse delegates to the shared routine (70a2adf2); Sun golden tolerance tightened 26″ → 5.0″, measured residual max 2.83″ (eb4339f2). · **Severity:** important (accuracy) · **Opened:** 2026-06-29

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
ecliptic longitude. (This is Sun-specific: for the planets, light-time and
stellar aberration are genuinely distinct — "planetary aberration" = both — so
the standard `apparent_position` is correct for them. The Moon should be
checked but is likely unaffected for the same reason as planets.)

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

- **Whole-branch review — ELP raw backend equatorial is intentionally of-date:** The ELP backend emits a J2000 `ecliptic` but derives its `equatorial` from the raw of-date lon/lat (preserving prior mean-mode values), so a direct ELP consumer who self-converts the J2000 ecliptic with mean obliquity will not reproduce the provided equatorial. Coherent and test-asserted (`assert_ne!`), and overridden by the chart layer for apparent bodies. **Remains open** (documented for any future direct-backend consumer).

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
  `docs/time-observer-policy.md`. → **Resolved 2026-07-01 (`4c79c6c2`, `bd0da1bc`):**
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
slice:** triaged from `33` → `3` documented equivalents, whole crate.

This is **not** part of the closed 2026-07-18 three-crate baseline, nor of the
closed `pleiades-houses` campaign. Both of those are finished. This is the first
slice of the expansion those closures explicitly anticipated ("any future
expansion to further `pleiades-*` crates opens new slices under this follow-up:
new work, not part of either closed body"), taking the first crate off the
`2026-07-25` roadmap queue.

**Measured, not estimated.** Baseline re-measured on this branch at
`7aefa946c`: `223 mutants tested in 2m: 33 missed, 186 caught, 4 unviable`
(exit 2). Final, after six kill tasks plus the review-driven underflow-lens
kill:

```
223 mutants tested in 3m: 3 missed, 216 caught, 4 unviable
```

Exit 2 both times — the report-only tier's expected outcome. Command:
`cargo mutants -p pleiades-apsides --test-tool nextest --test-workspace=false
--baseline run`. `30` mutants killed by `11` new tests and `1` new independent
reference helper (`state_from_elements`, a published perifocal-basis forward
construction deliberately *not* the crate's own inverse formulation, so a sign
or operator error in `apsides` cannot be masked by a shared expression). Crate
suite `19 → 21` tests, all passing.

**Per-task kills, each verified by its own whole-crate run:** forward-reference
geometry `10` (`33 → 23`), southern-perihelion branch `5` (`→ 18`),
`points_from_elements` guards `7` (`→ 11`), overflow-lens guards + negative μ
`3` (`→ 8`), `elements_from_state` node guards `3` (`→ 5`), derived-eccentricity
floor `1` (`→ 4`), then review-driven underflow lens `1` (`→ 3`).

**The three documented equivalents**, each carrying a written reachability
argument as a comment at its site in `crates/pleiades-apsides/src/lib.rs`. **No
`#[mutants::skip]` was added** — the arguments are the record, and the mutants
stay visible in every future run:

- `104:43` `||` → `&&` in `apsides`' input guard. **Rust precedence plus a
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
- `226:20` `<` → `<=` in `elements_from_state`'s southern-perihelion branch.
  **Symmetric straddle of the truth — not, as first claimed, exactness.** The
  two differ only at `peri_vec[2] == 0.0` exactly, where the true `omega` is
  `0` or `π`. They are *not* numerically identical there: `cos_omega` arrives
  one ulp off `1.0` through the longitude/latitude round-trip, so `acos`
  returns `1.49e-8` rad rather than `0`, and the two branches land **`1.71e-6`
  deg apart** in `peri_lon_deg` (measured at `pos [0.5, 0.25, 0.0]`,
  `vel [0.5, -1.0, 1.0]`, `mu 1`). It is nonetheless un-killable because the
  pair *straddles the truth symmetrically* — `acos` gives `+ε` where the truth
  is `0`, so the original lands at `node + ε` and the mutant at `node − ε`.
  Over a 1,272-case search of states reaching `peri_vec[2] == 0.0` exactly, the
  asymmetry `||orig − truth| − |mut − truth||` stayed `≤ 5.69e-14` deg. Any
  *symmetric* tolerance against an independent reference admits both or rejects
  both; only a **signed** assertion could separate them, and that pins the sign
  of rounding noise. (Its sibling `226:20` `<` → `==` is *not* equivalent and
  was killed in the southern-perihelion task — two mutants at one site with
  opposite dispositions.)
- `287:46` `+` → `-` in the aphelion argument of latitude. **Symmetric
  straddle again — not, as first claimed, a small bounded displacement.** In
  exact reals `cos(ω+π) = cos(ω−π) = −cos ω`, so both branches denote the same
  point and differ only in how the arguments round. But **the displacement is
  unbounded, not sweep-bounded**: it grows continuously as `|latitude| → 90`,
  where `atan2`'s two arguments both collapse toward zero. Measured at
  `node 0, omega 90, r 2.4` — `incl 89°` → `8.53e-13` deg, `incl 89.99°` →
  `8.04e-11`, `incl 89.9999°` → `8.04e-9` (**~8× this suite's own `1e-9` deg
  tolerance**), `incl 89.999999°` → `8.04e-7`. Latitude is `-89.9999` at the
  third point, so longitude is perfectly well defined and the
  "undefined at the pole" escape does not apply. A 1-degree sweep grid cannot
  see any of this, which is why two successive sweeps in this slice reported
  figures `3.7×` apart and both were wrong as bounds.

  It is un-killable for the same reason as `226:20`: the pair **straddles the
  truth symmetrically**, all the way in. Against the exact-in-real reference
  (substituting `−cos ω`, `−sin ω` for the mutated argument's trig), at
  `incl 89.9999°` the original sits at `−4.020250798930647e-9` deg and the
  mutant at `+4.020307642349508e-9` deg — opposite sides, equal magnitudes —
  and the asymmetry `||orig − truth| − |mut − truth||` stays at `5.68e-14` deg
  *however large the displacement grows*. Over a sweep of near-pole states the
  most a symmetric tolerance could ever exploit,
  `max(|mut − truth| − |orig − truth|)`, is `1.14e-13` deg, itself rounding
  noise. Only a **signed** assertion could separate them, and that pins the
  sign of rounding noise. (Measured against the *ideal* `ω = π/2` geometry
  instead, the mutant is the one **closer** to truth — `2.01e-9` deg versus the
  original's `6.03e-9` — so a tolerance admitting the original admits the
  mutant a fortiori.) At `|lat| == 90` exactly the separation reaches
  `116.56505117707803` deg, but longitude is mathematically undefined there and
  *both* branches are `atan2` of pure rounding noise.

Campaign-wide running tally **45 → 48** (`9` from the closed three-crate
baseline, `36` from the closed houses campaign, `3` from this slice), extending
the series `9 → 22 → 30 → 36 → 41 → 44 → 43 → 45 → 48`. In-crate
documented-equivalent sub-total for `pleiades-apsides`: **3**. (An earlier
draft of this entry claimed `4` and a tally of `49`; review proved the fourth
killable — see prediction 4 below.)

**Four predictions that measurement overturned.** Recorded so none is silent —
in each case the claim was rewritten from the measurement, not the other way
round. The first three were design-phase predictions overturned during
implementation; the fourth was overturned by **review**, after this slice had
already written the equivalence argument down:

1. **`122:10` (`<` → `<=` on the eccentricity floor) was predicted an
   unreachable boundary; it is killable, and was killed.** Unlike
   `points_from_elements`, `e` is *derived* here through a cancellation that
   makes the reachable grid ~1e6× coarser than the target's precision, and the
   design's searches failed. The reason they failed is instructive: they swept
   `r_mag` over **powers of two**, which makes the final `c1 * r_mag` product
   *exact* and so never lands on the boundary. Sweeping over consecutive
   doubles gives that product an independent rounding, and a state whose
   osculating eccentricity is bit-identical to `MIN_ECCENTRICITY` falls out.
2. **`104:43` was predicted killable; it is an equivalent.** The prediction
   assumed the mutant was `((a || b) && c) || d`. Rust's precedence makes it
   `a || (b && c) || d`, which has a far smaller distinguishing region — and
   that region is entirely absorbed by the downstream `!e.is_finite()` check,
   as argued above.
3. **`204:27` was predicted killable "by any radial state"; plain radial motion
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

**Per-mutant margin summary.** Full table in the slice report. Per campaign
discipline the rows are **never aggregated**, and no margin is fabricated for a
kill that has none: `15` of the `30` kills carry a genuine scalar displacement
against an assertion tolerance, and the remaining `15` are **exact
error-variant or exact-equality pins** with no displacement to report, disclosed
as such rather than given an invented number (the review-driven underflow-lens
kill is the fifteenth pin: `Ok`-carrying-`NaN` versus `Err(NonFinite)`). Each measured row states the
mutant's *strongest-firing* assertion (its kill signal) and residual. Across
those `15`, the **true minimum** kill margin is `3.65e10 ×` its assertion
tolerance — `227:21` `*` → `/`, which lands `peri_lon_deg` at `286.476°` where
`250°` is correct, a `36.48°` residual against a `1e-9` deg tolerance. The
largest is `8.08e15 ×` (`115:24` `*` → `/`, bifocal-sum residual `8.08e3` AU
against `1e-12`). The five southern-perihelion mutants land `36.5°`–`72.5°`
from the true `250°`; the ten forward-reference mutants run `4.11e10 ×` to
`8.08e15 ×`. No kill in this slice is marginal, and none relies on a
last-few-ulps distinction.

**One test's honest-naming gap closed.** `apsides_eccentricity_floor_is_exclusive`
pins only the *on*-boundary side; on its own it would also pass if the floor
check were deleted entirely. Its doc comment now cross-references the sibling
`near_circular_orbit_is_degenerate`, which holds the below-boundary side — the
two together give the exclusivity. Neither test was restructured.

**Two plan defects found and corrected, so neither is silent.** One wrong
figure: the southern-perihelion rationale reported the `*` → `+` mutant landing
at `144.6°`, which is the raw `omega` *before* the `+40°` node offset; the
correct final `peri_lon_deg` is **`184.6°`** (measured). One wrong mechanism:
the overflow-lens rationale said `104:27`'s mutant "proceeds and returns `Ok`".
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
