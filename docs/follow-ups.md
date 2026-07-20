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
slices** (priority order): `apparent.rs` (49), `aberration.rs` (28),
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
`apparent_from_true` direction), accounting for 20 of the 37 survivors including
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
