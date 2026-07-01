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
