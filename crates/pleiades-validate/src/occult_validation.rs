//! Fail-closed two-tier `validate-occultations` gate over the committed
//! Swiss-Ephemeris `swe_lun_occult_*` reference corpus (Task 9,
//! `data/occultations-corpus/{occultations.csv,manifest.txt}`).
//!
//! Tier 1 (self-consistency, no SE reference): every real occultation event
//! row (`occ_type` 1 or 2) is recomputed via
//! `pleiades_events::EventEngine::occultation` at SE's own reported maximum
//! instant, and the recomputed [`pleiades_events::LocalOccultation`] is
//! checked for internal consistency: `first_contact <= maximum <=
//! fourth_contact`; `second_contact`/`third_contact` present iff the target is
//! a planet disc AND the classification is `Total` (never for a point star);
//! `magnitude`/`obscuration` finite and `obscuration > 0 iff magnitude > 0`.
//!
//! Tier 2 (SE parity): the recomputed geometry is compared against the SE
//! corpus columns — see the module-level corrections below for the two SE
//! quirks that shape this comparison.
//!
//! ## Correction 1 — magnitude/obscuration are unbounded for planets ON SE'S
//! SIDE in the FULLY-COVERED regime (see `KNOWN GAP 1`)
//!
//! SE reports, for a planet target, `magnitude` (`attr[0]`, covered diameter
//! fraction) and `obscuration` (`attr[2]`, covered disc-area fraction) that
//! can run far past 1.0 (up to ~82 and ~26652 in the committed corpus) when
//! the planet is FULLY covered (`Total`), because these are fractions of the
//! TARGET's (planet's) own tiny disc, and the Moon's disc is vastly larger.
//! `pleiades_events::occult::covered_diameter_fraction` (magnitude) is
//! deliberately unclamped and reproduces the same large values — intentional
//! SE parity, not a bug — in BOTH the `Total` and `Grazing` regimes.
//! `obscuration_fraction`, however, IS clamped to `[0,1]` in every branch of
//! its implementation (a correctly bounded disc-AREA fraction): in the
//! GRAZING (partial-coverage) regime this bounded value IS comparable to SE's
//! and IS gated (`PLANET_OBSCURATION_REL`, Task 15); in the fully-covered
//! (`Total`) regime it does NOT and structurally CANNOT reproduce SE's large
//! values — see `KNOWN GAP 1` for why this gate does not attempt to gate
//! planet Total obscuration numerically. Only a point star (`se_body == -1`)
//! has a binary magnitude/obscuration in `{0.0, 1.0}` on BOTH sides. So Tier-1
//! asserts `magnitude, obscuration ∈ [0,1]` only for star rows; for planet
//! rows it asserts only `>= 0` (no upper bound) — this Tier-1 bound is honored
//! by both engine quantities regardless of `KNOWN GAP 1` (obscuration's bound
//! is `[0,1]`, a strict subset of `>= 0`).
//!
//! Consequently Tier-2 gates magnitude differently for stars vs. planets (see
//! "Comparison-mode choice" below); obscuration is gated for stars only.
//!
//! ## Correction 2 — `central` is structurally always 0 for `loc` rows
//!
//! SE's `swe_lun_occult_when_loc` never sets `SE_ECL_CENTRAL` (only
//! `SE_ECL_NONCENTRAL`), so every `loc` row's `central` column is `0`
//! regardless of geometry. Only `glob` rows (via `swe_lun_occult_where`) carry
//! a meaningful `central` flag. This gate compares `central` ONLY on `glob`
//! rows; it is never asserted on `loc` rows.
//!
//! **Correction 2b (discovered, not in the original plan; superseded by
//! SP-6-FU, see `KNOWN GAP 2` below):** even within `glob` rows, `central`
//! was, prior to SP-6-FU, only meaningfully compared/counted for PLANET
//! targets. Every star `glob` row in the committed corpus has `central == 0`
//! — SE structurally never marks a point-target occultation as central. Our
//! own engine's OLD `central` formula (tied to `occ_type == Total`)
//! degenerated to always-`true` for a point target (`s_tgt_deg == 0`
//! collapsed the "found" and "central" thresholds in
//! `next_global_occultation` to the identical inequality), so this gate used
//! to skip the `central` comparison for star `glob` rows entirely rather than
//! run a tautological (always-mismatching) check. SP-6-FU's axis-pierce
//! decoupling (`KNOWN GAP 2` below) is not definitionally tied to `occ_type`,
//! so that degeneracy no longer holds: star `glob` rows are now compared too
//! (measured 0/12 mismatched in the committed corpus) and hard-gated exactly
//! like planet rows — see `KNOWN GAP 2` below for both.
//!
//! ## Correction 3 — two distinct kinds of `occ_type == 0` (no-event) rows
//!
//! - **Un-occultable target** (the `Sirius@*` rows): the target can NEVER be
//!   occulted from anywhere on Earth because its ecliptic latitude exceeds
//!   the Moon's ~6.6° maximum reach
//!   (`pleiades_events::occult::MOON_MAX_REACH_DEG`, mirrored locally as
//!   [`MOON_MAX_REACH_DEG`] since the original is `pub(crate)`). This gate
//!   asserts `engine.next_occultation(..)` returns `Ok(None)` (the engine's own
//!   fast-reject path).
//! - **Geometric-miss occultable target** (e.g. `Aldebaran@miss`, `Venus@miss`):
//!   the target IS occultable in general, but this particular
//!   observer/instant is a miss. This gate asserts
//!   `engine.occultation(..)` returns a timed record with
//!   `occultation_type == Miss` (NOT `None` — `occultation()` always returns a
//!   record, timed at the closest approach).
//!
//! The discriminator: for a star row, recompute the star's apparent ecliptic
//! latitude (mirroring `target_never_occultable`'s own logic bit-for-bit —
//! see `star_never_occultable`) and compare against
//! [`MOON_MAX_REACH_DEG`]; a planet is never permanently un-occultable (it
//! moves), so every planet `occ_type == 0` row takes the geometric-miss path.
//!
//! **Anchor fix (gate-only, discovered by whole-branch review):** a
//! geometric-miss row's OWN `jd_tt` is not the real conjunction — it is the
//! sibling `@center`/`@graze` row's `max_jd` MINUS 0.5 day (see the corpus
//! generator), a full half-day before the actual Moon–target close approach.
//! `occultation()` only searches `± OCC_CONTACT_HALF_WINDOW_DAYS` (0.15 day)
//! around its anchor instant, so recomputing at the row's own `jd_tt` never
//! reaches the real conjunction — the Moon is still several degrees from the
//! target there, and `classify` trivially returns `Miss` for ANY observer,
//! making the assertion vacuous. `build_miss_sibling_anchors` performs a
//! first pass over the corpus building a `(se_body, star, jd_tt) ->
//! sibling_max_jd` map from every `loc` row's real `@center`/`@graze` sibling
//! (occ_type 1 or 2, which shares the exact same `jd_tt` token within a
//! sibling group). The geometric-miss path below now anchors
//! `engine.occultation(..)` at that sibling's `max_jd` — the REAL conjunction
//! instant — so the assertion genuinely exercises whether THIS observer is
//! beyond the graze limit AT the real event. The Sirius (`never`) path is
//! unaffected: those rows have no sibling (Sirius is never occultable from
//! anywhere) and keep asserting `next_occultation(..) == Ok(None)`. This
//! strengthened, non-vacuous check surfaced a real accuracy limitation — see
//! `KNOWN GAP 3` below.
//!
//! ## Comparison-mode choice: magnitude/obscuration, star vs. planet
//!
//! An absolute ceiling meaningful for a star (exactly 1.0, ceiling ~0.01) is
//! unachievable for a planet, whose magnitude values are 18-82 and
//! hypersensitive to sub-arcsecond separation and semidiameter differences
//! (magnitude ∝ 1/`s_tgt`). This gate therefore uses TWO modes for
//! MAGNITUDE: absolute residual for star rows (gated under
//! `STAR_MAGNITUDE_ABS`), and RELATIVE residual (`|recomputed − se| /
//! |se|`) for planet rows (gated under `PLANET_MAGNITUDE_REL`, measured
//! max 4.9%). The planet ABSOLUTE magnitude residual is still measured and
//! reported (informational, ungated) so a reviewer can sanity-check the
//! relative ceiling against the raw magnitude of the numbers involved.
//!
//! OBSCURATION for planet GRAZING rows is now gated the same way
//! (`PLANET_OBSCURATION_REL`, Task 15); planet TOTAL-row obscuration remains
//! ungated — see `KNOWN GAP 1` below (Correction 1b).
//!
//! ## KNOWN GAP 1 (Correction 1b) — planet `obscuration` is a different
//! quantity on the two sides ONLY when the planet is fully covered
//!
//! Star `obscuration` (binary `{0,1}`) matches SE exactly and IS gated
//! (`STAR_OBSCURATION_ABS`). For PLANET rows, this gate discovered that
//! `pleiades_events::occult::obscuration_fraction` is a properly, correctly
//! bounded `[0,1]` disc-AREA fraction (every branch of its implementation
//! clamps or returns an exact `0.0`/`1.0`) — this is the CORRECT physical
//! obscuration (a lens-area fraction can never exceed 1, since the lens area
//! is bounded by the target disc's own area). In the GRAZING (partial-
//! coverage) regime SE's `attr[2]` IS the same bounded lens-area fraction, and
//! the two sides ARE comparable — measured relative residual 0.30-4.93% in
//! the committed corpus, the same order as the gated `planet_magnitude_rel`,
//! so Task 15 gates it too (`PLANET_OBSCURATION_REL`,
//! `planet_obscuration_rel_grazing` bucket). In the FULLY-COVERED (`Total`)
//! regime, though, SE's reported `attr[2]` for a target much smaller than the
//! Moon is empirically NOT bounded (up to ~26652 in the committed corpus,
//! same order of magnitude disproportion as the (correctly) unclamped
//! magnitude in Correction 1) — a different (coverage-depth-ratio) quantity
//! that a bounded `[0,1]` area fraction cannot and should not reach. Our
//! bounded obscuration can NEVER reproduce SE's unbounded Total-regime one —
//! the measured relative residual there saturates near 1.0 (our value is a
//! rounding error next to SE's), which is not "close but out of tolerance",
//! it is proof the two sides are not the same physical quantity in that
//! regime specifically. Any ceiling loose enough to pass the Total case would
//! be vacuous (the brief's own warning against a "toothless" ceiling), so
//! `planet_obscuration_{abs,rel}` (which mix in Total rows) remain measured
//! and reported (informational, see [`OccultReport`]) but NOT part of the
//! fail-closed gate — only the Grazing-only bucket is gated. This is a
//! discovered SE/engine semantic difference specific to the fully-covered
//! regime — not a residual a tighter/looser threshold can resolve there.
//!
//! ## KNOWN GAP 2 (RESOLVED, SP-6-FU) — central axis-pierce exactness
//!
//! Prior to Task 15, this gate discovered that
//! `GlobalOccultation::sublunar_latitude/longitude` reported the point where
//! the MOON is at geocentric zenith (`moon_dec`, `moon_ra − GAST`) — NOT the
//! point on Earth where the occultation is centrally/best observed, which is
//! what SE's `swe_lun_occult_where` actually returns (and what the corpus's
//! `sublunar_lat/lon` columns are, despite the shared name). Independent
//! verification at the time: calling a LOCAL `occultation()` at the OLD
//! reported sub-lunar point gave `Miss` (no occultation there at all), while
//! calling it at SE's reported point gave `Total` matching SE's magnitude to
//! 4 significant figures. The measured residual was 2545-5344 ARCMINUTES
//! (42-89 DEGREES) across every glob row — squarely the "residuals are
//! DEGREES not arcsec, indicating a real engine bug" case this gate is
//! supposed to catch rather than hide.
//!
//! **Task 15 fixed this**: `next_global_occultation` now reports the
//! geographic point that actually MINIMIZES the topocentric Moon–target
//! separation at the greatest-occultation instant (golden-section coordinate
//! descent over `occ_geom`'s already-tested topocentric path, seeded at the
//! old sub-Moon point). The residual collapsed to arcmin-scale (measured max
//! ~70', see `occult_thresholds::SUBLUNAR_ARCMIN`'s doc) and `sublunar_arcmin`
//! is now GATED.
//!
//! `central`, however, was ALSO retied to the newly-correct point (`central`
//! ⟺ the target is fully behind the Moon's disc AT the minimized point) and
//! Saturn's two `glob` rows STILL disagree with SE (engine `true`/`Total`, SE
//! `central=false`) — an UNCHANGED 2-row mismatch. Diagnosis: this is not a
//! positional error. At SE's own reported central point our engine already
//! agreed with SE's magnitude to several significant figures (independently
//! verified) even before the point fix; the disagreement is in the boolean
//! flag itself. SE's `SE_ECL_CENTRAL` is evidently a stricter "the exact
//! Moon–target center-line axis strikes the Earth" condition, distinct from
//! merely "the target is fully covered somewhere" (`Total`) — analogous to a
//! solar eclipse being `Total` without being flagged `Central` (the umbral
//! axis passes just outside Earth while the umbral cone still grazes the
//! surface near the limb). This task's own brief-recommended retie formula
//! ties `central` definitionally to the `Total`/`Grazing` split
//! (`central` ⟺ `occ_type == Total`), so it structurally cannot represent
//! SE's finer distinction — a conceptual gap in `next_global_occultation`'s
//! `central`/`occ_type` coupling that THIS task's positional fix does not
//! close. This gap is deferred, not unresolvable: SE's `SE_ECL_CENTRAL` is a
//! computable closed-form axis-pierce test (per `swecl.c`'s
//! `eclipse_where`: `de·cosf1 >= r0`, the perpendicular distance from
//! geocenter to the Moon–target axis compared against an angular-radius-
//! derived threshold), so a bounded future task can decouple `central` from
//! `occ_type` and implement that exact test. At the time of the SP-6 diagnosis,
//! `central_planet_{checked, mismatched}` remained measured/reported but not
//! hard-gated, and Correction 2b excluded star `glob` rows from the comparison
//! entirely — both since superseded (see RESOLVED below).
//!
//! **RESOLVED (SP-6-FU):** the deferred fix above has been implemented. The
//! engine now ports `eclipse_where`'s own `de·cosf1 >= r0` axis-pierce test
//! (`pleiades_events::occult::axis_pierce_central`) rather than deriving
//! `central` from the `Total`/`Grazing` classification, so `central` is fully
//! decoupled from `occ_type` — an eclipse (occultation) can be `Total` at a
//! given observer without the Moon–target center-line axis itself striking
//! the Earth, and vice versa is structurally representable even though it
//! doesn't occur in the committed corpus. Saturn's previously-unchanged 2/6
//! `glob`-row mismatch collapsed to 0/6 measured — the same closed-form test
//! SE uses, evaluated against the same minimized central-observation
//! geometry Task 15 already fixed. Because `central` is now an exact
//! closed-form boolean rather than an approximation with a known residual,
//! the planet comparison is hard-gated (`central_planet_exact` in
//! [`validate_occultations_corpus`]): ANY planet `glob`-row mismatch fails
//! the gate closed, not just a count above a pinned ceiling. Decoupling
//! `central` from `occ_type` also removed the Correction 2b degeneracy for
//! star `glob` rows (a point target's `central` no longer collapses to
//! always-`true`), so those are now measured too (SP-6-FU Task 3) — measured
//! 0/12 mismatched in the committed corpus, so they are hard-gated as well
//! (`central_star_exact`). `central_planet_{checked,mismatched}` and
//! `central_star_{checked,mismatched}` remain on [`OccultReport`] as the
//! measured counts (always 0 mismatched for a passing gate), documenting what
//! was checked even though the ceiling itself is now an exact-match assertion
//! rather than a numeric residual.
//!
//! ## KNOWN GAP 3 (RESOLVED, SP-6-FU) — when_loc visibility semantics
//! reconciled
//!
//! **History (SP-6):** the "Anchor fix" above (`build_miss_sibling_anchors`)
//! strengthened the geometric-miss `loc`-row check to recompute at the
//! sibling `@center`/`@graze` row's REAL `max_jd` — the actual Moon–target
//! conjunction — instead of the row's own `jd_tt` (a vacuous half-day-early
//! anchor at which `classify` trivially returns `Miss` for any observer).
//! Recomputing at the real conjunction is a genuine, non-vacuous test, and it
//! surfaced a real accuracy limitation: of the 18 committed geometric-miss
//! observers checked this way, our engine classified 8 as `Total` where SE
//! reported `Miss`. SP-6's diagnosis split those 8 into 3 knife-edge (SE's
//! own graze-boundary margin <= 1 arcminute, within the corpus generator's
//! 0.25° observer-placement step) and 5 "genuine" (margins 3.7-11.6
//! arcminutes, our topocentric track apparently too WIDE at high geographic
//! latitude) with the root cause of the 5 left an open follow-up: not the
//! parallax formula (checked), suspected epoch/region ephemeris or
//! UT1/timing (unconfirmed). The count was pinned fail-closed at 8 via
//! [`MAX_MISS_CLASSIFICATION_DISAGREEMENTS`] pending that follow-up.
//!
//! **Resolution (SP-6-FU):** the follow-up (Tasks 4-8) built a differential
//! harness (`tests/occult_graze_diagnosis.rs`) that computed, per disagreeing
//! row, ΔT drift, geocentric/topocentric Moon and target position offsets,
//! semidiameter differences, and both sides' graze margins. It found: our
//! ΔT, Moon position (agrees with `de440` to ~0.0001″), and topocentric
//! transform are all correct — `ourMargin ≈ seMargin` at every disagreeing
//! row, i.e. NOT a position or timing error (full write-up:
//! `docs/superpowers/notes/2026-07-09-sp6-fu-graze-diagnosis.md`). 7 of the
//! (then-)8 disagreeing rows had the occulted target's true (unrefracted)
//! altitude — per the evidence note's approximate (±~0.5°) estimate —
//! 0.9°-2.3° BELOW the observer's horizon at the event. Reading SE's
//! vendored source (`swecl.c:2700-2732`, `swe_lun_occult_when_loc`)
//! confirmed the mechanism: SE folds VISIBILITY into event existence,
//! rejecting an otherwise-real occultation (retflag 0 under
//! `SE_ECL_ONE_TRY`) whenever the target's apparent (refracted) altitude is
//! <= 0 at every one of {max, C1..C4} (for a star occultation SE collapses
//! C1=C2 and C4=C3, `swecl.c:2696-2699`, so effectively 3 distinct
//! instants). Our engine deliberately keeps these
//! concerns separate: `occultation_type` is a pure geometric verdict, and
//! `LocalOccultation::any_phase_visible` (a continuous 30-second scan of
//! apparent altitude over `[C1, C4]`, `occult.rs` ~line 891) reports
//! visibility independently — so the two sides were being compared on
//! DIFFERENT quantities (our geometry-only verdict vs. SE's
//! geometry-AND-visibility verdict), not disagreeing on the same one.
//!
//! **Human decision (2026-07-09): gate-side reconciliation, not an engine
//! change** — `occultation_type`/`any_phase_visible` staying separate is the
//! more useful API (a caller can ask either question), so the fix is in the
//! comparison: this gate's SE-equivalent-Miss verdict is now "geometric
//! `Miss` OR `!any_phase_visible`" (commit 4bf919bd; see the per-row
//! comment where `se_equivalent_miss` is computed below), making the
//! comparison like-for-like with SE's own folded semantics. Measured count
//! dropped from 8/18 to **3/18** the same day. The 3 remaining rows (all
//! star targets) are NOT position error (engine numerics stay exonerated).
//! One is the original SP-6 knife-edge, persisting on pure geometry
//! regardless of visibility semantics: `Spica@miss` (real conjunction
//! 2005-09-07, lat 42.83°N) — SE's own margin there ≈ -0.005′, dead on the
//! graze limb, target well above the horizon (evidence-note estimate ≈ +7°
//! true altitude). The other two — `Aldebaran@miss` (real conjunction
//! 2033-08-18, lat 71.59°N) and `Spica@miss` (real conjunction 2012-07-25,
//! lat 65.85°S) — are expected (estimated, not per-instant measured) to be
//! residual visibility-MAPPING deltas: our continuous 30-second `[C1,C4]`
//! scan vs. SE's discrete apparent-altitude sample at {max, C1..C4}
//! (effectively 3 distinct instants for a star, per the collapse above),
//! where a scan can find a brief visible interval between contacts that a
//! few-instant sample misses (or vice versa). The evidence note's
//! approximate true (unrefracted) altitudes put those two targets ~1.0-1.8°
//! below the horizon at maximum with a ±~0.5° error bar — close enough to
//! the horizon for the two sampling strategies to disagree. See
//! [`MAX_MISS_CLASSIFICATION_DISAGREEMENTS`]'s doc for the full per-row
//! detail (jd_tt, lat/lon).
//!
//! Deep-`Total` and clear-`Miss` cases DO agree with SE closely: contact
//! instants to within tens of seconds (`contact_seconds`/
//! `contact_seconds_grazing`, gated) and star magnitude exactly
//! (`star_magnitude_abs`, gated) — the reconciled disagreement is specific
//! to the graze/visibility boundary, not a general timing or geometry
//! error.
//!
//! Because the residual is a genuinely different (and much smaller)
//! phenomenon than the original geometric disagreement — a knife-edge plus
//! a suspected scan-granularity mapping delta rather than an unexplained
//! track-width error — this gate does NOT hard-fail the 3 known rows (that
//! would require matching SE's exact instant sampling, not a meaningful
//! improvement in accuracy) and does NOT skip them silently either. The
//! COUNT of disagreements remains pinned fail-closed via
//! [`MAX_MISS_CLASSIFICATION_DISAGREEMENTS`] (now 3, reported as
//! `miss_classify_disagree` in [`OccultReport::summary_line`]): the 15
//! agreeing rows are still genuinely checked per-row, and a regression that
//! WIDENS the disagreement count trips the gate, while a future mapping
//! refinement (e.g. sampling `any_phase_visible` at SE's exact instants)
//! that narrows it still passes (and the pin can then be tightened
//! further). See `tests/occult_graze_diagnosis.rs` for the reproducible
//! differential harness (`cargo test -p pleiades-validate --test
//! occult_graze_diagnosis -- --ignored --nocapture`) and
//! `docs/superpowers/notes/2026-07-09-sp6-fu-graze-diagnosis.md` for the
//! full evidence trail, including the direct SE source citation.
//!
//! A sibling `manifest.txt` records the fnv1a64 digest of the CSV (drift
//! guard); a mismatch fails the gate closed.

use crate::occult_thresholds::*;
use pleiades_apparent::{fnv1a64, true_obliquity_degrees, Atmosphere};
use pleiades_backend::{CompositeBackend, RoutingBackend};
use pleiades_data::PackagedDataBackend;
use pleiades_elp::ElpBackend;
use pleiades_events::{
    fixed_star_apparent, EventEngine, LocalOccultation, OccultTarget, OccultationType,
};
use pleiades_fict::FictitiousBackend;
use pleiades_jpl::JplSnapshotBackend;
use pleiades_types::{
    Angle, CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
    OBLIQUITY_J2000_DEG,
};
use pleiades_vsop87::Vsop87Backend;
use std::collections::BTreeMap;

const CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/occultations-corpus/occultations.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/occultations-corpus/manifest.txt"
));
const CSV_FILE: &str = "occultations.csv";

/// Fixture row count pinned by the corpus (Task 9). Update when the corpus is
/// regenerated.
pub const EXPECTED_ROWS: usize = 62;

/// Mirrors `pleiades_events::occult::MOON_MAX_REACH_DEG` (`pub(crate)`, not
/// exported): the Moon's maximum reach in ecliptic latitude — beyond this a
/// star can never be occulted from anywhere on Earth. ~5.3° orbital
/// inclination + ~0.27° semidiameter + ~0.95° horizontal parallax ≈ 6.6°.
pub const MOON_MAX_REACH_DEG: f64 = 6.6;

/// Pinned ceiling (`KNOWN GAP 3` in the module doc, RESOLVED SP-6-FU) on how
/// many sibling-anchored geometric-miss `loc` rows the engine's
/// SE-equivalent-Miss verdict ("geometric Miss OR `!any_phase_visible`", see
/// the per-row comment where `se_equivalent_miss` is computed below) is
/// allowed to disagree with SE's `Miss` at the real conjunction before the
/// gate fails closed. Prior to SP-6-FU the comparison was purely geometric
/// (ignoring SE's own visibility gate) and measured 8 of 18 disagreeing; root
/// cause was diagnosed as SE `swe_lun_occult_when_loc`'s visibility
/// semantics (target apparent altitude > 0 at one of {max, C1..C4},
/// vendored `swecl.c:2700-2732`, source-confirmed) — SE folds visibility
/// into event existence, our engine deliberately keeps geometry and
/// visibility separate. Once the gate's comparison was made like-for-like
/// (commit 4bf919bd, this pin tightened same day), the measured count
/// dropped to 3 of 18 (2026-07-09) — engine numerics are exonerated
/// (`ourMargin ≈ seMargin`, Moon position agrees with `de440` to
/// ~0.0001″, ΔT sub-second); these are NOT position error. Of the 3
/// remaining rows (all star targets), one is the pre-existing SP-6
/// knife-edge, persisting on pure geometry regardless of visibility
/// semantics: `Spica@miss` (jd_tt 2453620.280228757, real conjunction
/// 2005-09-07, lat 42.830897°) — SE's own margin at its anchor ≈ -0.005′,
/// dead on the graze limb, with the target well ABOVE the horizon
/// (evidence-note estimate ≈ +7° true altitude). The other two —
/// `Aldebaran@miss` (jd_tt 2463827.536661250, real conjunction 2033-08-18,
/// lat 71.588516°) and `Spica@miss` (jd_tt 2456133.683363608, real
/// conjunction 2012-07-25, lat -65.851805°) — are EXPECTED (an estimate,
/// not a measurement: the identifying instrumentation recorded labels, not
/// per-instant altitudes) to be residual MAPPING deltas between our
/// continuous 30-second `[C1,C4]` visibility scan and SE's discrete
/// apparent-altitude sample at {max, C1..C4} (for a star SE collapses
/// C1=C2 and C4=C3, so effectively 3 distinct instants): the evidence
/// note's approximate true (unrefracted) altitudes put both targets
/// ~1.0-1.8° below the horizon at maximum, with a ±~0.5° error bar —
/// close enough to the horizon for the two sampling strategies to disagree
/// on whether the target was ever visible. A future fix that narrows the
/// mapping delta (e.g. sampling `any_phase_visible` at SE's exact instants
/// instead of a continuous scan) still passes this
/// ceiling and the pin can then be tightened further; a regression that
/// WIDENS the disagreement trips it. See
/// `docs/superpowers/notes/2026-07-09-sp6-fu-graze-diagnosis.md` for the full
/// differential diagnosis and `tests/occult_graze_diagnosis.rs` for the
/// reproducible harness.
///
/// Narrowing acknowledgment: because the SE-equivalent-Miss rule is an OR
/// (`geometric Miss OR !any_phase_visible`), this metric can no longer catch
/// a geometry-only regression on a below-horizon row — a wrongly-computed
/// `Total` there is masked by `!any_phase_visible`, so it still counts as
/// SE-equivalent-Miss and stays "agreeing" regardless of the geometry bug.
/// Geometry regressions on such rows remain covered by the other gated
/// metrics (contact seconds, sublunar distance, magnitudes) and by the 15
/// agreeing rows, whose per-row geometry is still genuinely checked above.
pub const MAX_MISS_CLASSIFICATION_DISAGREEMENTS: usize = 3;

#[derive(Debug)]
pub enum OccultError {
    /// A committed CSV's digest disagrees with the manifest.
    ChecksumMismatch {
        file: &'static str,
        got: u64,
        want: u64,
    },
    /// The parsed corpus row count disagrees with [`EXPECTED_ROWS`].
    RowCountMismatch { expected: usize, got: usize },
    /// A Tier-2 residual exceeded its ceiling.
    ToleranceExceeded {
        metric: &'static str,
        label: String,
        jd: f64,
        residual: f64,
        ceiling: f64,
    },
    /// Malformed manifest/corpus row, a Tier-1 self-consistency invariant
    /// failing on the recomputed row, an unrecognized `se_body`/`mode`, or a
    /// Tier-2 exact-match assertion failing (`occultation_type`, `central`,
    /// or an unexpected `Some`/`None` on the never-occultable/global paths).
    /// Fail-closed: never a silent skip.
    Parse { row: String },
    /// The engine errored while recomputing a row.
    Engine(String),
}

impl std::fmt::Display for OccultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for OccultError {}

/// Summary of the measured per-metric maxima and checked-row count for the
/// gate. Fields marked "informational" below are measured and reported but
/// NOT part of the fail-closed ceiling gate in
/// [`validate_occultations_corpus`] — see `KNOWN GAP 1` in the module doc for
/// why. (`KNOWN GAP 2`, the `central` exact-bool comparison, was resolved in
/// SP-6-FU and is now hard-gated — see `central_planet_checked`/
/// `central_star_checked` below.)
#[derive(Debug, Default)]
pub struct OccultReport {
    pub rows: usize,
    pub max_contact_seconds: f64,
    pub max_contact_seconds_grazing: f64,
    /// GATED (Task 15) under `SUBLUNAR_ARCMIN`: great-circle arcmin between
    /// the recomputed central-observation point
    /// (`GlobalOccultation::sublunar_latitude/longitude`, fixed by Task 15 —
    /// it used to report the Moon's geocentric zenith point, off by 42-89
    /// DEGREES; see the module doc's former `KNOWN GAP 2`) and the corpus's
    /// SE `sublunar_lat/lon`, over all `glob` rows.
    pub max_sublunar_arcmin: f64,
    pub max_star_magnitude_abs: f64,
    pub max_star_obscuration_abs: f64,
    /// Informational: planet magnitude absolute residual (see
    /// `max_planet_magnitude_rel` for the gated metric).
    pub max_planet_magnitude_abs: f64,
    pub max_planet_magnitude_rel: f64,
    /// Informational (Correction 1b), planet TOTAL + GRAZING rows combined:
    /// planet obscuration absolute residual. Our engine's `obscuration` is a
    /// properly bounded `[0,1]` disc-area fraction; SE's `attr[2]` for a
    /// FULLY-COVERED (`Total`) planet is empirically NOT bounded (values up
    /// to ~26652 in the committed corpus) and is a genuinely different
    /// (coverage-depth-ratio) physical quantity at that size ratio — NOT
    /// gated (any ceiling loose enough to pass would be vacuous). See
    /// `max_planet_obscuration_rel_grazing` for the GRAZING-only residual,
    /// which IS gated (the two quantities ARE comparable there).
    pub max_planet_obscuration_abs: f64,
    /// Informational (Correction 1b), planet TOTAL + GRAZING rows combined:
    /// relative residual saturates near 1.0 because the Total-row cases
    /// dominate the max (our bounded value vs. SE's much larger one) —
    /// confirms the values are not comparable at this scale for a
    /// fully-covered planet, not that the engine is "close". See
    /// `max_planet_obscuration_rel_grazing` for the gated Grazing-only bucket.
    pub max_planet_obscuration_rel: f64,
    /// GATED (Task 15) under `PLANET_OBSCURATION_REL`: obscuration relative
    /// residual for PLANET GRAZING (`occ_type == 1`) rows only — bucketed
    /// exactly like `max_contact_seconds` vs `max_contact_seconds_grazing`.
    /// Comparable in scale to the gated `max_planet_magnitude_rel` (both are
    /// well-conditioned in the Grazing regime, unlike the Total regime — see
    /// `max_planet_obscuration_rel`'s doc).
    pub max_planet_obscuration_rel_grazing: f64,
    /// GATED (`central_planet_exact`, SP-6-FU): planet glob rows where
    /// `central` was compared. See `KNOWN GAP 2 (RESOLVED, SP-6-FU)` in the
    /// module doc.
    pub central_planet_checked: usize,
    /// GATED (`central_planet_exact`): of those, how many disagreed with SE.
    /// `central` now ports SE's `eclipse_where` axis-pierce test
    /// (`de·cosf1 >= r0`, `pleiades_events::occult::axis_pierce_central`)
    /// rather than being definitionally tied to `occ_type == Total`, so
    /// Saturn's former 2/6 mismatch (see the module doc's resolved `KNOWN
    /// GAP 2` for the prior diagnosis) collapsed to 0/6 measured in the
    /// committed corpus — [`validate_occultations_corpus`] fails closed
    /// (`central_planet_exact`) on ANY nonzero count here.
    pub central_planet_mismatched: usize,
    /// GATED (`central_star_exact`, SP-6-FU): star glob rows where `central`
    /// was compared. The pre-axis-pierce exclusion of Correction 2b (which
    /// skipped star rows as tautologically always-mismatching) no longer
    /// applies structurally now that `central` is decoupled from `occ_type`;
    /// measured 0/12 mismatched in the committed corpus, so this comparison
    /// is hard-gated too.
    pub central_star_checked: usize,
    /// GATED (`central_star_exact`): of those, how many disagreed with SE.
    /// Measured 0 in the committed corpus — [`validate_occultations_corpus`]
    /// fails closed on any nonzero count here.
    pub central_star_mismatched: usize,
    /// Geometric-miss `loc` rows re-anchored at the sibling's real `max_jd`
    /// and re-classified against SE's like-for-like SE-equivalent-Miss
    /// verdict (`KNOWN GAP 3`, RESOLVED SP-6-FU). Measured/reported; NOT
    /// hard-gated per-row — the COUNT is pinned by
    /// [`MAX_MISS_CLASSIFICATION_DISAGREEMENTS`] instead (see
    /// [`validate_occultations_corpus`]). Unlike `central_planet_*`/
    /// `central_star_*` (resolved to an exact hard gate in SP-6-FU, `KNOWN
    /// GAP 2`), the 3 residual rows here are a scan-granularity mapping
    /// delta (not a position/timing error) that a zero-tolerance gate would
    /// require exactly matching SE's 5-instant sampling to close, so this
    /// stays a pinned count rather than a zero-tolerance gate.
    pub miss_classify_checked: usize,
    /// Of `miss_classify_checked`, how many the engine's SE-equivalent-Miss
    /// verdict ("geometric `Miss` OR `!any_phase_visible`") disagreed with
    /// SE's `Miss` at the real conjunction — see `KNOWN GAP 3` in the module
    /// doc.
    pub miss_classify_disagree: usize,
}

impl OccultReport {
    /// The gate passed iff every committed row was checked (a silently
    /// truncated corpus is a failure, not a pass). Every GATED ceiling is
    /// enforced fail-closed by [`validate_occultations_corpus`], so reaching
    /// a report already implies every checked row was within its gated
    /// ceiling — informational fields (see field docs) are reported but not
    /// part of that gate.
    pub fn passed(&self) -> bool {
        self.rows == EXPECTED_ROWS
    }

    pub fn summary_line(&self) -> String {
        format!(
            "validate-occultations: {} rows — max residuals (gated): contact {:.3}s contact_grazing {:.3}s star_mag {:.4} star_obsc {:.4} planet_mag_rel {:.4} sublunar {:.1}' planet_obsc_rel_grazing {:.4} central_planet {}/{} (gated exact) central_star {}/{} (gated exact) — informational (ungated, see KNOWN GAP 1): planet_mag_abs {:.3} planet_obsc_abs {:.1} planet_obsc_rel {:.4} — pinned (KNOWN GAP 3 resolved: SE visibility semantics reconciled): miss_classify_disagree {}/{}",
            self.rows,
            self.max_contact_seconds,
            self.max_contact_seconds_grazing,
            self.max_star_magnitude_abs,
            self.max_star_obscuration_abs,
            self.max_planet_magnitude_rel,
            self.max_sublunar_arcmin,
            self.max_planet_obscuration_rel_grazing,
            self.central_planet_mismatched,
            self.central_planet_checked,
            self.central_star_mismatched,
            self.central_star_checked,
            self.max_planet_magnitude_abs,
            self.max_planet_obscuration_abs,
            self.max_planet_obscuration_rel,
            self.miss_classify_disagree,
            self.miss_classify_checked,
        )
    }
}

/// One metric's running maximum, tagged with the offending row's label and
/// `jd_tt` so a ceiling violation can be reported precisely.
#[derive(Debug, Clone)]
struct MetricMax {
    value: f64,
    label: String,
    jd: f64,
}

impl Default for MetricMax {
    fn default() -> Self {
        MetricMax {
            value: 0.0,
            label: String::new(),
            jd: f64::NAN,
        }
    }
}

impl MetricMax {
    fn observe(&mut self, value: f64, label: &str, jd: f64) {
        // A NaN residual never compares `Greater`, so treat it as a new max
        // explicitly — it must never be silently dropped (checked again,
        // fail-closed, at gate time via `residual.is_finite()`).
        let is_new_max = match value.partial_cmp(&self.value) {
            Some(std::cmp::Ordering::Greater) => true,
            Some(_) => false,
            None => true,
        };
        if is_new_max {
            self.value = value;
            self.label = label.to_string();
            self.jd = jd;
        }
    }
}

/// All measured residual maxima over the committed corpus. Tier-1
/// self-consistency is enforced during measurement (it never depends on the
/// numeric ceilings); ceiling gating is applied afterwards by
/// [`validate_occultations_corpus`].
#[derive(Debug, Default)]
struct Measured {
    rows: usize,
    /// Contact/maximum instant residuals from well-conditioned (`Total`,
    /// `occ_type == 2`) `loc` rows AND all `glob` rows (always `Total`-class
    /// in the committed corpus).
    contact_total: MetricMax,
    /// Contact/maximum instant residuals from `Grazing` (`occ_type == 1`)
    /// `loc` rows.
    contact_grazing: MetricMax,
    /// Great-circle residual (arcmin) between the recomputed central-
    /// observation point (`GlobalOccultation::sublunar_latitude/longitude`,
    /// fixed by Task 15) and the corpus's SE `sublunar_lat/lon`, for all
    /// `glob` rows. GATED under `SUBLUNAR_ARCMIN`.
    sublunar: MetricMax,
    star_magnitude: MetricMax,
    star_obscuration: MetricMax,
    planet_magnitude_abs: MetricMax,
    planet_magnitude_rel: MetricMax,
    planet_obscuration_abs: MetricMax,
    planet_obscuration_rel: MetricMax,
    /// PLANET GRAZING (`occ_type == 1`) `loc` rows only: obscuration relative
    /// residual, bucketed exactly like `contact_total` vs `contact_grazing`.
    /// GATED under `PLANET_OBSCURATION_REL` — see the field's note at its use
    /// site in `measure` for why this (unlike the Total-inclusive
    /// `planet_obscuration_rel` above) is comparable in scale and safe to
    /// gate.
    planet_obscuration_rel_grazing: MetricMax,
    /// Planet `glob` rows where `central` was compared / how many disagreed.
    /// GATED (`central_planet_exact`, SP-6-FU) — see `KNOWN GAP 2 (RESOLVED,
    /// SP-6-FU)` in the module doc: `central` now ports SE's `eclipse_where`
    /// axis-pierce test instead of being tied to `occ_type`, and Saturn's
    /// former 2/6 mismatch collapsed to 0/6. See field docs on
    /// [`OccultReport`].
    central_planet_checked: usize,
    central_planet_mismatched: usize,
    /// Star glob rows where `central` was compared / how many disagreed
    /// (SP-6-FU). GATED (`central_star_exact`) — measured 0/12 mismatched in
    /// the committed corpus. See field docs on [`OccultReport`].
    central_star_checked: usize,
    central_star_mismatched: usize,
    /// Geometric-miss `loc` rows (`occ_type == 0`, occultable target, has a
    /// `@center`/`@graze` sibling) re-anchored at the sibling's REAL `max_jd`
    /// and re-classified — see `KNOWN GAP 3` (RESOLVED, SP-6-FU) in the
    /// module doc. `checked` is every such row; `disagree` is how many the
    /// engine's SE-equivalent-Miss verdict ("geometric `Miss` OR
    /// `!any_phase_visible`") disagreed with SE's `Miss` at the real
    /// conjunction. NOT hard-failed per-row (that would defeat the gate's
    /// purpose given a diagnosed, bounded residual mapping delta) — instead
    /// the COUNT is pinned by [`MAX_MISS_CLASSIFICATION_DISAGREEMENTS`] in
    /// `validate_occultations_corpus`, so a regression that widens the
    /// disagreement still fails the gate.
    miss_classify_checked: usize,
    miss_classify_disagree: usize,
}

impl Measured {
    fn into_report(self) -> OccultReport {
        OccultReport {
            rows: self.rows,
            max_contact_seconds: self.contact_total.value,
            max_contact_seconds_grazing: self.contact_grazing.value,
            max_sublunar_arcmin: self.sublunar.value,
            max_star_magnitude_abs: self.star_magnitude.value,
            max_star_obscuration_abs: self.star_obscuration.value,
            max_planet_magnitude_abs: self.planet_magnitude_abs.value,
            max_planet_magnitude_rel: self.planet_magnitude_rel.value,
            max_planet_obscuration_abs: self.planet_obscuration_abs.value,
            max_planet_obscuration_rel: self.planet_obscuration_rel.value,
            max_planet_obscuration_rel_grazing: self.planet_obscuration_rel_grazing.value,
            central_planet_checked: self.central_planet_checked,
            central_planet_mismatched: self.central_planet_mismatched,
            central_star_checked: self.central_star_checked,
            central_star_mismatched: self.central_star_mismatched,
            miss_classify_checked: self.miss_classify_checked,
            miss_classify_disagree: self.miss_classify_disagree,
        }
    }
}

/// SE body number -> `CelestialBody`. Only -1 (star, handled separately), 3
/// (Venus), 5 (Jupiter), and 6 (Saturn) appear in the committed corpus, but
/// the full SE 0-9 major mapping is honored for robustness against corpus
/// regeneration; any other value is a fail-closed parse error.
fn body_from_se(se_body: i64) -> Option<CelestialBody> {
    match se_body {
        0 => Some(CelestialBody::Sun),
        1 => Some(CelestialBody::Moon),
        2 => Some(CelestialBody::Mercury),
        3 => Some(CelestialBody::Venus),
        4 => Some(CelestialBody::Mars),
        5 => Some(CelestialBody::Jupiter),
        6 => Some(CelestialBody::Saturn),
        7 => Some(CelestialBody::Uranus),
        8 => Some(CelestialBody::Neptune),
        9 => Some(CelestialBody::Pluto),
        _ => None,
    }
}

fn parse_manifest() -> Result<BTreeMap<String, (usize, u64)>, OccultError> {
    let mut map = BTreeMap::new();
    for line in MANIFEST.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("file:") else {
            continue;
        };
        let toks: Vec<&str> = rest.split_whitespace().collect();
        if toks.len() < 3 {
            return Err(OccultError::Parse {
                row: format!("malformed file line: {line}"),
            });
        }
        let name = toks[0].to_string();
        let mut rows = None;
        let mut checksum = None;
        for tok in &toks[1..] {
            if let Some(v) = tok.strip_prefix("rows=") {
                rows = Some(v.parse::<usize>().map_err(|e| OccultError::Parse {
                    row: format!("rows: {e}"),
                })?);
            } else if let Some(v) = tok.strip_prefix("checksum=") {
                checksum = Some(v.parse::<u64>().map_err(|e| OccultError::Parse {
                    row: format!("checksum: {e}"),
                })?);
            }
        }
        let rows = rows.ok_or_else(|| OccultError::Parse {
            row: format!("rows= missing: {line}"),
        })?;
        let checksum = checksum.ok_or_else(|| OccultError::Parse {
            row: format!("checksum= missing: {line}"),
        })?;
        map.insert(name, (rows, checksum));
    }
    if map.is_empty() {
        return Err(OccultError::Parse {
            row: "no `file:` lines found in manifest".to_string(),
        });
    }
    Ok(map)
}

/// Looks up `file` in the manifest and compares `fnv1a64(csv)` against the
/// recorded checksum, fail-closed. Returns the manifest's declared row count
/// on success.
fn check_checksum(file: &'static str, csv: &str) -> Result<usize, OccultError> {
    let manifest = parse_manifest()?;
    let (rows, want) = *manifest.get(file).ok_or_else(|| OccultError::Parse {
        row: format!("manifest missing entry for {file}"),
    })?;
    let got = fnv1a64(csv);
    if got != want {
        return Err(OccultError::ChecksumMismatch { file, got, want });
    }
    Ok(rows)
}

fn parse_f64(s: &str, row: &str) -> Result<f64, OccultError> {
    s.trim().parse::<f64>().map_err(|_| OccultError::Parse {
        row: row.to_string(),
    })
}

fn parse_i64(s: &str, row: &str) -> Result<i64, OccultError> {
    s.trim().parse::<i64>().map_err(|_| OccultError::Parse {
        row: row.to_string(),
    })
}

/// Great-circle separation (degrees) between two geographic lat/lon points —
/// identical formula to `pleiades_events::occult::angular_separation_deg`
/// (RA/Dec there, lat/lon here; the spherical-law-of-cosines math is the
/// same).
fn great_circle_deg(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let (a1, d1) = (lon1.to_radians(), lat1.to_radians());
    let (a2, d2) = (lon2.to_radians(), lat2.to_radians());
    let cos_sep = (d1.sin() * d2.sin() + d1.cos() * d2.cos() * (a1 - a2).cos()).clamp(-1.0, 1.0);
    cos_sep.acos().to_degrees()
}

/// Whether a star (by curated catalog name) sits permanently outside the
/// Moon's reach — mirrors `pleiades_events::occult::EventEngine::target_never_occultable`'s
/// star branch bit-for-bit (that method is `pub(crate)`, so this
/// re-derives it from the public `fixed_star_apparent` + `to_ecliptic` path).
fn star_never_occultable(name: &str, jd: f64) -> Result<bool, OccultError> {
    let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
    let equ = fixed_star_apparent(name, at)
        .map_err(|e| OccultError::Engine(format!("{name} ({jd}): {e}")))?;
    let eps = true_obliquity_degrees(jd).unwrap_or(OBLIQUITY_J2000_DEG);
    let ecl = equ.to_ecliptic(Angle::from_degrees(eps));
    Ok(ecl.latitude.degrees().abs() > MOON_MAX_REACH_DEG)
}

/// Sibling-group key for a `loc`-mode row: the target identity
/// (`se_body`/`star`) plus the shared `jd_tt` search anchor that ties a
/// geometric-miss row (`occ_type == 0`) to its `@center`/`@graze` sibling(s)
/// (`occ_type` 2/1). Keyed on the RAW `jd_tt` token (not the parsed `f64`)
/// since the corpus writes byte-identical `jd_tt` text within a sibling
/// group — this sidesteps any float re-parse/equality pitfall.
type MissSiblingKey = (i64, String, String);

/// First pass over the corpus (§"Anchor fix" in the module doc): maps every
/// `loc`-mode sibling group to the REAL conjunction instant (`max_jd`)
/// reported by its `@center`/`@graze` row (`occ_type` 2 or 1). A group with
/// no such sibling (e.g. `Sirius@*`, which has no `@center`/`@graze` row at
/// all) is simply absent from the map — that absence is exactly the
/// un-occultable-target signal the geometric-miss branch below checks for.
fn build_miss_sibling_anchors(csv: &str) -> Result<BTreeMap<MissSiblingKey, f64>, OccultError> {
    // For planet groups (Venus/Jupiter/Saturn), the `@center` (occ_type 2)
    // and `@graze` (occ_type 1) rows share the identical `(se_body, star,
    // jd_tt)` key, so this insert is last-write-wins (currently `@graze`,
    // whichever comes later in the CSV). That's intentional and harmless:
    // both candidate `max_jd` anchors sit well inside `occultation()`'s
    // ±0.15-day self-refining search window, so either sibling is a valid
    // anchor for the real conjunction — there is no single "the" unambiguous
    // sibling per group, and it doesn't matter which one wins.
    let mut map = BTreeMap::new();
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("label,") {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        if f.len() != 19 {
            return Err(OccultError::Parse {
                row: line.to_string(),
            });
        }
        if f[1].trim() != "loc" {
            continue;
        }
        let occ_type = parse_i64(f[15], line)?;
        if occ_type == 0 {
            continue;
        }
        let se_body = parse_i64(f[2], line)?;
        let star = f[3].trim().to_string();
        let jd_tt_token = f[4].trim().to_string();
        let max_jd = parse_f64(f[8], line)?;
        map.insert((se_body, star, jd_tt_token), max_jd);
    }
    Ok(map)
}

/// Checks Tier-1 self-consistency for one recomputed `loc`-mode occultation
/// event row (`occ_type` 1 or 2). Returns an error identifying the offending
/// row on failure.
fn check_tier1_local(
    label: &str,
    rec: &LocalOccultation,
    is_star: bool,
) -> Result<(), OccultError> {
    if !(rec.magnitude.is_finite() && rec.obscuration.is_finite()) {
        return Err(OccultError::Parse {
            row: format!(
                "{label}: non-finite magnitude/obscuration: {} {}",
                rec.magnitude, rec.obscuration
            ),
        });
    }
    let max_jd = rec.maximum.instant.julian_day.days();
    let c1_jd = rec.first_contact.instant.julian_day.days();
    let c4_jd = rec.fourth_contact.instant.julian_day.days();
    if !(c1_jd <= max_jd && max_jd <= c4_jd) {
        return Err(OccultError::Parse {
            row: format!("{label}: contacts not ordered c1={c1_jd} max={max_jd} c4={c4_jd}"),
        });
    }
    // Interior contacts (c2/c3) exist iff the target is a planet disc AND the
    // classification is Total — a point star never has them (§Correction 1).
    let disc_total = !is_star && rec.occultation_type == OccultationType::Total;
    let has_interior = rec.second_contact.is_some() && rec.third_contact.is_some();
    if disc_total != has_interior {
        return Err(OccultError::Parse {
            row: format!(
                "{label}: c2/c3 presence mismatch (disc_total={disc_total} has_interior={has_interior})"
            ),
        });
    }
    if is_star {
        if !(0.0..=1.0).contains(&rec.magnitude) || !(0.0..=1.0).contains(&rec.obscuration) {
            return Err(OccultError::Parse {
                row: format!(
                    "{label}: star magnitude/obscuration out of [0,1]: {} {}",
                    rec.magnitude, rec.obscuration
                ),
            });
        }
    } else if rec.magnitude < 0.0 || rec.obscuration < 0.0 {
        return Err(OccultError::Parse {
            row: format!(
                "{label}: planet magnitude/obscuration negative: {} {}",
                rec.magnitude, rec.obscuration
            ),
        });
    }
    if (rec.obscuration > 0.0) != (rec.magnitude > 0.0) {
        return Err(OccultError::Parse {
            row: format!(
                "{label}: obscuration>0 iff magnitude>0 violated: mag={} obs={}",
                rec.magnitude, rec.obscuration
            ),
        });
    }
    Ok(())
}

/// Runs the checksum guard, parses the corpus, recomputes every row via a
/// freshly built production-style `EventEngine`, enforces Tier-1
/// self-consistency, and accumulates every Tier-2 residual maximum. Numeric
/// ceiling gating is NOT applied here (that is [`validate_occultations_corpus`]'s
/// job) — so this succeeds regardless of the ceiling constants.
fn measure() -> Result<Measured, OccultError> {
    check_checksum(CSV_FILE, CSV)?;
    let miss_sibling_anchors = build_miss_sibling_anchors(CSV)?;

    let backend = RoutingBackend::new(vec![
        Box::new(PackagedDataBackend::new()),
        Box::new(CompositeBackend::new(
            Vsop87Backend::new(),
            ElpBackend::new(),
        )),
        Box::new(JplSnapshotBackend::new()),
        Box::new(FictitiousBackend::new(PackagedDataBackend::new())),
    ]);
    let engine = EventEngine::new(backend);
    let mut m = Measured::default();

    for line in CSV.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("label,") {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        if f.len() != 19 {
            return Err(OccultError::Parse {
                row: line.to_string(),
            });
        }
        let label = f[0].to_string();
        let mode = f[1].trim();
        let se_body = parse_i64(f[2], line)?;
        let star = f[3].trim().to_string();
        let jd_tt = parse_f64(f[4], line)?;
        let lat = parse_f64(f[5], line)?;
        let lon = parse_f64(f[6], line)?;
        let elev = parse_f64(f[7], line)?;
        let max_jd = parse_f64(f[8], line)?;
        let c1_jd = parse_f64(f[9], line)?;
        let c2_jd = parse_f64(f[10], line)?;
        let c3_jd = parse_f64(f[11], line)?;
        let c4_jd = parse_f64(f[12], line)?;
        let se_magnitude = parse_f64(f[13], line)?;
        let se_obscuration = parse_f64(f[14], line)?;
        let occ_type = parse_i64(f[15], line)?;
        let se_sublunar_lat = parse_f64(f[16], line)?;
        let se_sublunar_lon = parse_f64(f[17], line)?;
        let central = parse_i64(f[18], line)?;

        let is_star = se_body == -1;
        let target = if is_star {
            OccultTarget::Star(star.clone())
        } else {
            let body = body_from_se(se_body).ok_or_else(|| OccultError::Parse {
                row: format!("{line} (unrecognized se_body {se_body})"),
            })?;
            OccultTarget::Body(body)
        };

        match mode {
            "loc" => {
                let observer = ObserverLocation::new(
                    Latitude::from_degrees(lat),
                    Longitude::from_degrees(lon),
                    Some(elev),
                );
                if occ_type == 0 {
                    // Two distinct kinds of no-event row (§Correction 3; see
                    // "Anchor fix" in the module doc for why the
                    // geometric-miss branch anchors at the SIBLING's
                    // max_jd, not this row's own jd_tt).
                    let never = is_star && star_never_occultable(&star, jd_tt)?;
                    let sibling_key = (se_body, star.clone(), f[4].trim().to_string());
                    let sibling_max_jd = miss_sibling_anchors.get(&sibling_key).copied();
                    if never {
                        let after = Instant::new(JulianDay::from_days(jd_tt), TimeScale::Tdb);
                        let result = engine
                            .next_occultation(target, observer, Atmosphere::default(), after)
                            .map_err(|e| OccultError::Engine(format!("{label} ({jd_tt}): {e}")))?;
                        if result.is_some() {
                            return Err(OccultError::Parse {
                                row: format!(
                                    "{label}: expected None (never-occultable) but got Some"
                                ),
                            });
                        }
                    } else {
                        // Geometric-miss occultable target: anchor at the
                        // sibling @center/@graze row's REAL max_jd (the
                        // actual conjunction), not this row's own jd_tt
                        // (which is sibling_max_jd - 0.5 day, outside
                        // occultation()'s ±0.15-day search window around the
                        // real event — see the module doc's "Anchor fix").
                        let sibling_max_jd = sibling_max_jd.ok_or_else(|| OccultError::Parse {
                            row: format!(
                                "{label}: geometric-miss row has no @center/@graze sibling with a real max_jd (se_body={se_body} star={star:?} jd_tt={jd_tt})"
                            ),
                        })?;
                        let at = Instant::new(JulianDay::from_days(sibling_max_jd), TimeScale::Tdb);
                        let rec = engine
                            .occultation(target, observer, Atmosphere::default(), at)
                            .map_err(|e| {
                                OccultError::Engine(format!("{label} ({sibling_max_jd}): {e}"))
                            })?;
                        // KNOWN GAP 3 (RESOLVED, SP-6-FU): NOT hard-failed
                        // per-row. SE's
                        // `swe_lun_occult_when_loc` (vendored swecl.c:
                        // 2700-2732) folds VISIBILITY into event existence:
                        // it reports Miss whenever the occulted target's
                        // apparent (refracted) altitude is <= 0 at every one
                        // of {max, C1..C4}, even when the geometry is a real
                        // occultation. Our engine deliberately keeps these
                        // separate — `occultation_type` is a pure geometric
                        // verdict, and `any_phase_visible` (continuous
                        // 30-second scan of apparent altitude over
                        // [C1, C4] — occult.rs ~line 891) reports visibility
                        // independently. So the like-for-like ("SE-
                        // equivalent Miss") comparison is "geometric Miss OR
                        // no phase visible" — see
                        // docs/superpowers/notes/2026-07-09-sp6-fu-graze-diagnosis.md.
                        // A residual mapping delta remains possible: our
                        // continuous scan can find a brief above-horizon
                        // interval between contacts that SE's 5-instant
                        // sample misses, so it can report `any_phase_visible
                        // = true` where SE reports Miss; such rows still
                        // disagree here and are counted. The per-row check
                        // now runs at the REAL conjunction (not a vacuous
                        // half-day-early anchor) for the majority that DO
                        // agree; the minority that don't are counted here and
                        // the COUNT is pinned fail-closed against regression
                        // by `MAX_MISS_CLASSIFICATION_DISAGREEMENTS` in
                        // `validate_occultations_corpus`.
                        m.miss_classify_checked += 1;
                        let se_equivalent_miss =
                            rec.occultation_type == OccultationType::Miss || !rec.any_phase_visible;
                        if !se_equivalent_miss {
                            m.miss_classify_disagree += 1;
                        }
                    }
                } else {
                    let at = Instant::new(JulianDay::from_days(max_jd), TimeScale::Tdb);
                    let rec = engine
                        .occultation(target, observer, Atmosphere::default(), at)
                        .map_err(|e| OccultError::Engine(format!("{label} ({max_jd}): {e}")))?;

                    check_tier1_local(&label, &rec, is_star)?;

                    let expected_type = if occ_type == 2 {
                        OccultationType::Total
                    } else {
                        OccultationType::Grazing
                    };
                    if rec.occultation_type != expected_type {
                        return Err(OccultError::Parse {
                            row: format!(
                                "{label}: expected {expected_type:?} got {:?}",
                                rec.occultation_type
                            ),
                        });
                    }

                    let contact_tracker = if occ_type == 2 {
                        &mut m.contact_total
                    } else {
                        &mut m.contact_grazing
                    };
                    let max_res = (rec.maximum.instant.julian_day.days() - max_jd).abs() * 86_400.0;
                    contact_tracker.observe(max_res, &label, max_jd);
                    let c1_res =
                        (rec.first_contact.instant.julian_day.days() - c1_jd).abs() * 86_400.0;
                    contact_tracker.observe(c1_res, &label, max_jd);
                    let c4_res =
                        (rec.fourth_contact.instant.julian_day.days() - c4_jd).abs() * 86_400.0;
                    contact_tracker.observe(c4_res, &label, max_jd);
                    if c2_jd > 0.0 {
                        if let Some(sc) = rec.second_contact {
                            let r = (sc.instant.julian_day.days() - c2_jd).abs() * 86_400.0;
                            contact_tracker.observe(r, &label, max_jd);
                        }
                    }
                    if c3_jd > 0.0 {
                        if let Some(tc) = rec.third_contact {
                            let r = (tc.instant.julian_day.days() - c3_jd).abs() * 86_400.0;
                            contact_tracker.observe(r, &label, max_jd);
                        }
                    }

                    let mag_res = (rec.magnitude - se_magnitude).abs();
                    let obs_res = (rec.obscuration - se_obscuration).abs();
                    if is_star {
                        m.star_magnitude.observe(mag_res, &label, max_jd);
                        m.star_obscuration.observe(obs_res, &label, max_jd);
                    } else {
                        m.planet_magnitude_abs.observe(mag_res, &label, max_jd);
                        m.planet_obscuration_abs.observe(obs_res, &label, max_jd);
                        let mag_rel = mag_res / se_magnitude.abs();
                        let obs_rel = obs_res / se_obscuration.abs();
                        m.planet_magnitude_rel.observe(mag_rel, &label, max_jd);
                        m.planet_obscuration_rel.observe(obs_rel, &label, max_jd);
                        // PLANET GRAZING obscuration relative residual — bucketed
                        // exactly like `contact_total` vs `contact_grazing`
                        // (occ_type 2 = Total vs 1 = Grazing). Unlike the
                        // combined `planet_obscuration_rel` above (which mixes
                        // in Total rows and saturates near 1.0 — see `KNOWN GAP
                        // 1`), the Grazing-only residual is comparable in scale
                        // to the gated `planet_magnitude_rel` (0.30-4.93%
                        // measured) and IS gated under `PLANET_OBSCURATION_REL`.
                        if occ_type == 1 {
                            m.planet_obscuration_rel_grazing
                                .observe(obs_rel, &label, max_jd);
                        }
                    }
                }
            }
            "glob" => {
                let after = Instant::new(JulianDay::from_days(max_jd - 1.0), TimeScale::Tdb);
                let rec = engine
                    .next_global_occultation(target, after)
                    .map_err(|e| OccultError::Engine(format!("{label} ({max_jd}): {e}")))?
                    .ok_or_else(|| OccultError::Parse {
                        row: format!("{label}: expected Some global occultation, got None"),
                    })?;

                let max_res = (rec.maximum.julian_day.days() - max_jd).abs() * 86_400.0;
                m.contact_total.observe(max_res, &label, max_jd);

                // Sub-lunar (central-observation) point: GATED under
                // `SUBLUNAR_ARCMIN` (Task 15). `next_global_occultation` used
                // to report the Moon's geocentric zenith point, off by 42-89
                // DEGREES from SE's `swe_lun_occult_where` point (the former
                // `KNOWN GAP 2`) — Task 15 fixed the engine to report the
                // geographic point that actually minimizes the topocentric
                // Moon–target separation at the greatest-occultation instant,
                // collapsing the residual to arcmin-scale (measured max ~70',
                // see `occult_thresholds::SUBLUNAR_ARCMIN`'s doc for the exact
                // figure/row/date).
                let arcmin = great_circle_deg(
                    rec.sublunar_latitude.degrees(),
                    rec.sublunar_longitude.degrees(),
                    se_sublunar_lat,
                    se_sublunar_lon,
                ) * 60.0;
                m.sublunar.observe(arcmin, &label, max_jd);

                // `central` is now HARD-GATED for both PLANET and STAR rows
                // (`central_planet_exact`/`central_star_exact` in
                // `validate_occultations_corpus`) — see `KNOWN GAP 2
                // (RESOLVED, SP-6-FU)` in the module doc for the full
                // history. Prior to SP-6-FU, `central` was definitionally
                // tied to `occ_type == Total`, which could not represent
                // SE's stricter "exact center-line axis strikes the Earth"
                // condition (`SE_ECL_CENTRAL`) and left Saturn's 2/6 planet
                // glob rows unexplained, measured-but-ungated. SP-6-FU
                // decoupled `central` from `occ_type` by porting SE's own
                // closed-form axis-pierce test (`de·cosf1 >= r0`,
                // `pleiades_events::occult::axis_pierce_central`), which
                // collapsed the Saturn mismatch to 0/6 and is exact enough to
                // hard-gate rather than merely measure.
                //
                // Correction 2b's degenerate always-true star `central`
                // (which used to make a star comparison tautological, since a
                // point target's "found" and "central" thresholds collapsed
                // to the same inequality under the old `occ_type`-tied
                // formula) no longer applies now that `central` is the
                // axis-pierce test — so star `glob` rows are compared too
                // (measured 0/12 mismatched in the committed corpus) and
                // hard-gated identically to planet rows.
                if !is_star {
                    let expected_central = central != 0;
                    m.central_planet_checked += 1;
                    if rec.central != expected_central {
                        m.central_planet_mismatched += 1;
                    }
                } else {
                    let expected_central = central != 0;
                    m.central_star_checked += 1;
                    if rec.central != expected_central {
                        m.central_star_mismatched += 1;
                    }
                }
            }
            other => {
                return Err(OccultError::Parse {
                    row: format!("{line} (unrecognized mode {other})"),
                })
            }
        }

        m.rows += 1;
    }

    if m.rows != EXPECTED_ROWS {
        return Err(OccultError::RowCountMismatch {
            expected: EXPECTED_ROWS,
            got: m.rows,
        });
    }
    Ok(m)
}

// NOTE: every gated bucket below assumes >= 1 row was actually measured into
// it. `MetricMax` defaults its `value` to `0.0` (see its `Default` impl), so
// an EMPTY bucket would trivially pass `check_metric` (0.0 <= any positive
// ceiling) rather than fail closed. This is safe only because the committed
// corpus is known to populate every gated bucket with >= 5 rows (see each
// bucket's row count in `occult_thresholds`'s doc comments / the corpus CSV);
// a regenerated corpus that dropped a bucket to zero rows would silently lose
// that check rather than error — a caveat inherited from the corpus's own
// `RowCountMismatch`/checksum guards, which catch a truncated corpus overall
// but not a zeroed-out sub-bucket specifically.
fn check_metric(
    metric: &'static str,
    tracked: &MetricMax,
    ceiling: f64,
) -> Result<(), OccultError> {
    let residual = tracked.value;
    if !residual.is_finite() || residual > ceiling {
        return Err(OccultError::ToleranceExceeded {
            metric,
            label: tracked.label.clone(),
            jd: tracked.jd,
            residual,
            ceiling,
        });
    }
    Ok(())
}

/// Full two-tier gate: Tier-1 self-consistency (via `measure`) plus Tier-2 SE
/// parity gated under the provisional ceilings in `crate::occult_thresholds`.
/// Fails closed on any exceeded ceiling.
///
/// Seven metrics are gated by numeric ceiling: `contact_seconds`,
/// `contact_seconds_grazing`, `star_magnitude_abs`, `star_obscuration_abs`,
/// `planet_magnitude_rel`, `sublunar_arcmin` (Task 15), and
/// `planet_obscuration_rel_grazing` (Task 15). Two more are gated exact-bool
/// (SP-6-FU, `KNOWN GAP 2 (RESOLVED)`): `central_planet_exact` and
/// `central_star_exact` fail closed on ANY planet or star glob row where
/// `central` disagrees with SE's `SE_ECL_CENTRAL`. A tenth is gated by
/// pinned COUNT rather than per-row: `miss_classify_disagree` (SP-6,
/// comparison semantics reconciled and pin tightened to the measured count
/// in SP-6-FU, [`MAX_MISS_CLASSIFICATION_DISAGREEMENTS`]) — see `KNOWN GAP
/// 3 (RESOLVED, SP-6-FU)` in the module doc. `planet_obscuration_{abs,rel}`
/// (Total-inclusive) remain
/// measured/counted and reported (see [`OccultReport`]'s field docs) but
/// deliberately NOT gated at all — see `KNOWN GAP 1` in the module doc for
/// why: gating planet Total obscuration would be vacuous (a different
/// physical quantity at that size ratio).
pub fn validate_occultations_corpus() -> Result<OccultReport, OccultError> {
    let m = measure()?;

    check_metric("contact_seconds", &m.contact_total, CONTACT_SECONDS)?;
    check_metric(
        "contact_seconds_grazing",
        &m.contact_grazing,
        CONTACT_SECONDS_GRAZING,
    )?;
    check_metric("star_magnitude_abs", &m.star_magnitude, STAR_MAGNITUDE_ABS)?;
    check_metric(
        "star_obscuration_abs",
        &m.star_obscuration,
        STAR_OBSCURATION_ABS,
    )?;
    check_metric(
        "planet_magnitude_rel",
        &m.planet_magnitude_rel,
        PLANET_MAGNITUDE_REL,
    )?;
    check_metric("sublunar_arcmin", &m.sublunar, SUBLUNAR_ARCMIN)?;
    check_metric(
        "planet_obscuration_rel_grazing",
        &m.planet_obscuration_rel_grazing,
        PLANET_OBSCURATION_REL,
    )?;

    // KNOWN GAP 2 (resolved, SP-6-FU): `central` is now SE's axis-pierce
    // test and must match SE exactly on every planet glob row.
    if m.central_planet_mismatched > 0 {
        return Err(OccultError::ToleranceExceeded {
            metric: "central_planet_exact",
            label: format!(
                "{} of {} planet glob rows mismatched SE's SE_ECL_CENTRAL",
                m.central_planet_mismatched, m.central_planet_checked
            ),
            jd: f64::NAN,
            residual: m.central_planet_mismatched as f64,
            ceiling: 0.0,
        });
    }

    // Star glob rows (Task 3, SP-6-FU): measured exact (0/12 mismatched in
    // the committed corpus) now that `central` is the axis-pierce test
    // rather than the degenerate always-true flag Correction 2b excluded —
    // so this is hard-gated too, same shape as `central_planet_exact`.
    if m.central_star_mismatched > 0 {
        return Err(OccultError::ToleranceExceeded {
            metric: "central_star_exact",
            label: format!(
                "{} of {} star glob rows mismatched SE's SE_ECL_CENTRAL",
                m.central_star_mismatched, m.central_star_checked
            ),
            jd: f64::NAN,
            residual: m.central_star_mismatched as f64,
            ceiling: 0.0,
        });
    }

    // KNOWN GAP 3 (RESOLVED, SP-6-FU): the COUNT of sibling-anchored
    // geometric-miss rows whose SE-equivalent-Miss verdict disagrees with SE
    // (at the real conjunction) is pinned fail-closed here rather than
    // hard-failing each disagreeing row — see
    // MAX_MISS_CLASSIFICATION_DISAGREEMENTS's doc and the module doc's KNOWN
    // GAP 3 for the resolution. A regression that widens the count beyond
    // the pinned ceiling fails the gate; the majority-agreeing rows are
    // still genuinely (non-vacuously) checked per-row above.
    if m.miss_classify_disagree > MAX_MISS_CLASSIFICATION_DISAGREEMENTS {
        return Err(OccultError::ToleranceExceeded {
            metric: "miss_classify_disagree",
            label: format!(
                "{} of {} sibling-anchored geometric-miss rows disagreed with SE (SE-equivalent non-Miss vs SE Miss)",
                m.miss_classify_disagree, m.miss_classify_checked
            ),
            jd: f64::NAN,
            residual: m.miss_classify_disagree as f64,
            ceiling: MAX_MISS_CLASSIFICATION_DISAGREEMENTS as f64,
        });
    }

    Ok(m.into_report())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_row_count_is_pinned() {
        let report = validate_occultations_corpus().expect("occult gate passes");
        assert_eq!(report.rows, EXPECTED_ROWS);
    }

    #[test]
    fn checksum_drift_fails_closed() {
        assert!(check_checksum("occultations.csv", "mutated,corpus\n").is_err());
    }

    #[test]
    fn gate_passes_on_committed_corpus() {
        let report = validate_occultations_corpus().expect("occult gate passes");
        eprintln!("{}", report.summary_line());
    }

    #[test]
    fn unrecognized_se_body_fails_closed() {
        assert!(body_from_se(15).is_none());
        assert!(body_from_se(20).is_none());
        assert!(body_from_se(40).is_none());
    }
}
