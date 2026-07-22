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
mutants to **19 documented equivalents**: `spherical_cotrans` (34),
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
never reached, and `19` are genuine equivalents. The `7` were killed by two
degenerate-axis `asc2` pins (x on the `sinx ≈ 0` axis, reaching the
`sinx.abs() < 1e-12` branch) and one `asc_mc_from` geometry where the vertex
flip actually fires (`vemc > 0`), which the plan's three geometries never
triggered. **Documented residual — 19 equivalent mutants**, all left visible
(no `#[mutants::skip]`), enumerated in
`asc_geometry_equivalent_mutants_are_documented` with a per-mutant reachability
argument grouped by structural reason:

- **360-periodicity of `asc2` in `x`** (it uses only `sin(x)`/`cos(x)`):
  `asc1` arm-3 `x1 - 180 → x1 + 180` and `delete match arm 3` (arm 3 is
  identical to the `_` arm); `asc_mc_from` `armc - 180 → + 180` at both call
  sites (`armc ± 180` differ by exactly 360). [4]
- **180-periodicity of `tan` in the pole height**: `f_pole = 90 - lat` vs
  `-90 - lat` differ by exactly 180 and both `asc2` and `ascendant_for` use
  only `tan(pole)`, so the `lat >= 0 → lat < 0` branch swap and the `delete -`
  that turns `-90 - lat` into `90 - lat` are unobservable. [2]
- **Exact mod-360 fold symmetry**: the vertex flip `vertex + 180 → vertex - 180`
  normalizes to the same longitude. [1]
- **Unreachable exact-equality boundaries**: `vemc == 180`, `vemc == 0`, and
  `asc2`'s `1e-12` guard thresholds (`< → <=`) — no representable input hits
  equality. [7, incl. `longitude_opposite`'s `x + 180 ≡ x - 180 (mod 360)`]
- **±90 / ±1e-12 sentinel fold**: `asc2`'s `value == 0` and longitude-fold
  branches (`< → ==`/`>`, `delete -`, `< → <=`) leave the folded result
  unchanged or shift it below the crate's 1e-9 parity tolerance. [5]

These bring the **running documented-equivalent tally to `9 + 19 = 28`** (the
`19` here are the first from the post-baseline houses campaign). No parity gate
was touched; the tier stays report-only; `mise run ci` is green. **Remaining
houses PRs:** great-circle (`apc_sector`/`krusinski`/`horizon`), sector
(`pullen_sr`/`pullen_sd`/`albategnius`/`gauquelin`), sunshine/solar-arc,
quadrant/projection, then catalog + thresholds (which adds `-p pleiades-houses`
to `[tasks.mutants]`).

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
