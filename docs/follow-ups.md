# Follow-ups / deferred items

Tracked engineering items deferred out of the scope that surfaced them. Each
entry: what, where, evidence, impact, suggested fix, and origin.

---

## FU-1: Latent geocentric-Sun aberration double-count in `pleiades-core` apparent path

**Status:** open · **Severity:** important (accuracy) · **Opened:** 2026-06-29

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
