# SP-6-FU — Occultation `central` axis-pierce exactness + high-latitude graze-boundary root-cause

**Date:** 2026-07-09
**Status:** Approved design (brainstormed, user-validated)
**Predecessor:** SP-6 lunar occultations (PR #17, merged 2026-07-09), which
documented `KNOWN GAP 2` and `KNOWN GAP 3` in
`crates/pleiades-validate/src/occult_validation.rs`'s module doc.

## Problem

PR #17 left two diagnosed-but-open gaps in the occultation engine:

1. **KNOWN GAP 2** — `GlobalOccultation::central` is definitionally tied to
   `occ_type == Total` at the minimized central-observation point
   (`occult.rs`, `next_global_occultation`). SE's `SE_ECL_CENTRAL` is a
   stricter "the Moon–target center-line axis strikes the Earth" condition,
   so 2 of 6 Saturn `glob` corpus rows disagree (engine `true`, SE `false`).
   The flag is measured but not gated.

2. **KNOWN GAP 3** — the strengthened sibling-anchored geometric-miss check
   revealed that of 18 committed geometric-miss observers recomputed at the
   real conjunction, the engine classifies 8 as `Total` where SE reports
   `Miss`. 3 are knife-edge (SE's own graze margin <= 1', inside the corpus
   generator's 0.25 deg observer-placement noise — settled, not actionable).
   5 are genuine: the topocentric occultation track is measurably too WIDE at
   high geographic latitude, margins 3.7–11.6 arcmin. Root cause unknown; the
   parallax formula was independently ruled out; ephemeris source and
   UT1/timing are suspected but unconfirmed. The count is pinned fail-closed
   at 8 via `MAX_MISS_CLASSIFICATION_DISAGREEMENTS`.

## Goals and success criteria

One slice, one branch (`feat/sp6-fu-occult-central-graze`), two deliverables:

- **GAP 2 (bounded):** implement SE's closed-form axis-pierce test for
  `central`. Success: `central_planet_mismatched` drops 2 → 0 across all 6
  planet glob rows and the comparison is promoted from informational to
  hard-gated.
- **GAP 3 (diagnose + fix):** root-cause the 5 genuine disagreements with a
  reproducible differential harness, fix the diagnosed stage. Success:
  `miss_classify_disagree` drops 8 → <= 3 (only the knife-edge rows remain)
  and `MAX_MISS_CLASSIFICATION_DISAGREEMENTS` is tightened to the new
  measured count.
  - **Escape hatch (pre-agreed):** if the evidence proves the cause is
    outside our control (e.g., an inherent compact-lunar-theory vs
    SE-Moshier accuracy bound — category (c) below), the deliverable becomes
    that documented evidence: KNOWN GAP 3 rewritten as a diagnosed,
    evidence-backed accuracy bound with the harness as proof, a
    compatibility-profile note, and the pin kept at the measured count. No
    speculative engine change.

Ripple in both cases: compatibility profile 0.7.12 → 0.7.13, KNOWN GAP 2/3
module-doc rewrites preserving the diagnosis history (as the sublunar-point
fix did), summary-line updates.

**Out of scope:** central-path cartography (full path polygon — future SP
slice), corpus regeneration/densification, the 3 knife-edge rows, widening
any body or backend claims.

## Design — GAP 2: `central` axis-pierce test

**Component:** one new closed-form helper in
`crates/pleiades-events/src/occult.rs`, shape
`fn central_axis_pierce(&self, target: &OccultTarget, jd: f64) -> Result<bool, EventError>`,
called from `next_global_occultation` to set `central`. `occ_type` keeps its
current semantics (Total ⟺ target fully covered at the minimized
central-observation point); only the boolean decouples from it.

**Data flow:** at `max_jd`, take geocentric positions of the Moon and the
target (Cartesian, km; a star contributes a unit direction only), form the
shadow axis — the line from the target's direction through the Moon's center
— and compute the perpendicular distance from the geocenter to that axis.
`central = true` iff that distance is within SE's angular-radius-derived
threshold: the `de * cosf1 >= r0` test from `swecl.c`'s `eclipse_where`,
ported EXACTLY (including SE's Earth-figure/flattening handling) from the
vendored `swecl.c` source shipped with the `libswisseph-sys` crate already
used by `tools/se-occultations-reference` — not reconstructed from memory.

**Gate changes** (`occult_validation.rs` + `occult_thresholds.rs`):

- Planet glob rows: `central` promoted from informational to hard-gated
  exact-bool (6 rows, expected 0 mismatches).
- Star glob rows: currently excluded from the `central` comparison entirely
  (Correction 2b). Measure star-row `central` parity under the new test; if
  exact, gate those rows too; otherwise keep the exclusion with the measured
  reason documented in the module doc. Fail-closed either way — no silent
  skip.

**Error handling:** closed-form, no iteration; reuses existing
`EventError::Backend` propagation from the position calls. No new error
variants.

**Testing:** unit tests on the helper with synthetic geometry (axis through
the geocenter → `true`; axis missing Earth by a wide margin → `false`;
near-threshold on both sides), the corpus gate as the integration check, and
the existing `occult_integration.rs` suite staying green.

## Design — GAP 3: differential harness, probes, fix

**Scale fact that shapes the probes:** at the graze boundary, geographic
margin amplifies Moon-position error — the ground boundary shifts roughly
1 arcmin of latitude (~2 km) per ~1 arcsec of topocentric Moon-position
error. The observed 3.7–11.6 arcmin margins therefore correspond to only
~4–12 arcsec of Moon position: exactly the order at which a compact
Meeus-style lunar theory and SE's Moshier ephemeris can legitimately differ.
Note also that the corpus was generated with `SEFLG_MOSEPH` (corpus header):
SE's reference truth here is Moshier, not de440.

**Component 1 — SE intermediates fixture.** Extend
`tools/se-occultations-reference` with a diagnostic mode that, for each of
the 18 geometric-miss rows' sibling-anchor instants (`@center`/`@graze`
sibling `max_jd`, the real conjunction), emits SE's intermediates:
geocentric and topocentric Moon RA/Dec/distance, target RA/Dec,
semidiameters, separation, and the Delta-T SE used. Committed as a small
checksummed CSV next to the existing corpus (same `manifest.txt` fnv1a64
drift-guard pattern; a mismatch fails the harness closed), so the
differential comparison runs offline and reproducibly.

**Component 2 — our-side stage dump.** A `#[doc(hidden)] pub` diagnostics
function in `pleiades-events` (thin wrapper over the existing `pub(crate)`
`moon_target_radec` / `occ_geom` path) exposing the same intermediates per
pipeline stage. A harness in `pleiades-validate` (an `#[ignore]`-by-default
test, run via `cargo test -- --ignored`; no new CLI surface) joins the two
sides into a stage-by-stage residual table per row:

geocentric Moon → apparent-place corrections → GAST/UT1 → topocentric
transform → semidiameters → classification margin.

The first stage whose residual is large enough to explain the row's margin
is the root cause, by construction. The harness also re-identifies the 8/18
disagreeing rows deterministically (the current gate only counts them).

**Probes, in order (cheapest and most-suspected first):**

1. **Delta-T / UT1.** The corpus epochs run to ~2036, beyond real UT1
   tables, and `moon_target_radec` falls back SILENTLY with
   `ut1_jd_from_tt(jd).unwrap_or(jd)` — a full-Delta-T-sized hour-angle
   error (~17 arcmin) if that path is ever hit. Check whether the fallback
   triggers at the affected epochs and compare our Delta-T against SE's
   emitted Delta-T per row.
2. **Moon-source substitution.** Kernel-gated behind `PLEIADES_DE_KERNEL`
   (same pattern as `corpus_regen`): recompute the disagreeing rows with
   de440 Moon positions in place of the packaged/ELP chain. Interpretation
   is two-sided because SE's truth is Moshier: if de440 ALSO disagrees with
   SE, that is itself evidence the residual is Moshier-vs-reality rather
   than an engine bug.
3. **Full bisection** via the residual table, only if probes 1–2 are
   inconclusive.

**Fix, by diagnosis category:**

- (a) **Timing/UT1 bug** → correct it in `pleiades-time` / call sites, with
  unit tests pinning the fallback path's behavior.
- (b) **Transform / apparent-place bug** → fix that stage, with targeted
  unit tests.
- (c) **Inherent lunar-theory accuracy limit** → the escape hatch above: no
  speculative change; document with evidence, keep the pin at the measured
  count, note the bound in the compatibility profile.

**Gate updates on a successful fix:**
`MAX_MISS_CLASSIFICATION_DISAGREEMENTS` tightened 8 → measured post-fix
count (target <= 3), `OccultReport::summary_line` updated, KNOWN GAP 3
module-doc section rewritten with the resolution history.

**Testing:** the harness doubles as the reproducible diagnosis record;
regression coverage is the tightened corpus gate; any fixed stage gets
targeted unit tests. The full `run_all_numeric_gates` battery must stay
green — a Moon-path timing fix could ripple into the rise/set, eclipse, and
crossing gates; those gates are the safety net, and any residual shifts
there are re-measured and re-pinned, never blanket-loosened.

## Execution order

1. GAP 2 (bounded, independent of GAP 3): helper + retie + gate promotion.
2. GAP 3 component builds: SE intermediates fixture, stage-dump diagnostics,
   differential harness.
3. GAP 3 probes 1 → 2 → (3 if needed); diagnosis recorded in the module doc.
4. GAP 3 fix per category (a)/(b), or escape-hatch documentation per (c).
5. Gate re-measure + pin tightening, compatibility profile 0.7.13, status
   files, PR.
