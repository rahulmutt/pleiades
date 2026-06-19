# SP3 — Published Accuracy Thresholds & Size/Latency Budgets

Status: design approved (2026-06-19)
Phase: 2 (Release-Grade Compressed Ephemeris), slice SP3
Predecessor: SP2 (heliocentric-planet reframe) — complete and merged
Branch base: `main` (SP2 landed)

## Goal

Close Phase 2 by turning the *measured* accuracy and size/latency numbers into a
**published, enforced contract**: per-body-class, per-channel accuracy thresholds
(longitude, latitude, distance, **and** speed/motion) and size/latency budgets,
with hard gates where the inputs are deterministic and tracked summaries where
they are environment-sensitive.

After SP3 the packaged artifact can be described as release-grade for its
advertised window and channel profile, not merely "draft with measured error."

## Scope decisions (resolved during brainstorming)

1. **Coverage window: 1900–2100.** Thresholds are published and enforced against
   the actual default artifact and the hold-out corpus, which are 1900–2100. The
   spec's earlier 1600–2600 target is reframed as documented future expansion,
   not a first-release gate. (The hold-out corpus only exists for 1900–2100, so
   1600–2600 cannot be honestly measured today.)
2. **Budget posture: hard size gate, tracked latency.** Encoded artifact size is
   deterministic → hard-fail gate. Latency is environment-sensitive (two
   benchmark tests are already known to fail under concurrent CI load and pass
   serially) → published target + non-gating tracked summary; the ceiling check
   is `#[ignore]`/env-opt-in so CI stays green.
3. **Channels: all four (lon, lat, distance, speed).** Speed is included by
   explicit choice, which pulls the Phase 4 motion-output decision forward.
4. **Speed mechanism: full support, flip to Derived.** The packaged artifact
   moves from `SpeedPolicy::Unsupported` to `SpeedPolicy::FittedDerivative`, the
   motion lookup path is implemented, and the public profile declares
   `Motion = Derived`. This is a published-capability change recorded against
   Phase 4.
5. **Threshold structure: two-tier (approach A).** A published, stable ceiling
   per body-class × channel (generous headroom = the public promise) **plus** the
   existing tight golden drift test retained as the sensitive regression catcher.
   The two have distinct jobs; both are kept.

## Architecture

### Integration with the existing target-threshold subsystem

Discovery during planning: an elaborate target-threshold subsystem already
exists in `crates/pleiades-data/src/coverage/target.rs` (745 lines) and
`coverage/threshold.rs` (967 lines). It already provides:

- `PackagedArtifactTargetThresholdState { Draft, ProductionReady }` with
  `is_production_ready()` / `validate_production_ready()` — currently
  `ProductionReady`.
- Body-class **scopes** already named:
  `PACKAGED_ARTIFACT_TARGET_THRESHOLD_SCOPES = ["luminaries", "major planets",
  "pluto", "lunar points", "selected asteroids", "custom bodies"]`.
- Per-scope **measured fit envelopes** (`PackagedArtifactFitEnvelopeSummary`,
  mean/max Δlon/Δlat/Δdist in **degrees / AU**) — artifact vs the fit-truth
  backend it was fit from.
- CLI summaries (`packaged-artifact-target-threshold-summary`,
  `-state`, `-scope-envelopes-summary`), golden/drift validation, and
  release-bundle alignment checks (`bundle_verify_helpers.rs`).

That subsystem is **fit-quality evidence + provenance + drift detection** ("did
the measured numbers change?"). It is NOT a published-ceiling pass/fail gate
("is measured under the promised bound?"), it has no speed channel, and it has
no size/latency budgets. SP3 adds exactly those missing layers and **reuses the
existing scope vocabulary, summary/golden conventions, and CLI/help-sync
mechanics** rather than duplicating them. The separate per-body hold-out
measurement in `accuracy_baseline.rs` (artifact vs independent hold-out, in
**arcsec / km**) stays the home for the published accuracy ceilings + pass/fail,
because the hold-out is the independent check.

Extending the `target.rs` fit-envelope summaries themselves to carry a speed
channel is **out of SP3 scope** (speed is gated via the accuracy-baseline +
thresholds path); this keeps the heavy provenance/release-bundle surface
untouched.

### Single source of truth: `crates/pleiades-data/src/thresholds.rs`

A new module, sibling to `accuracy_baseline.rs`, is the one place published
**ceilings and budgets** live (the existing `target.rs` keeps owning fit-envelope
evidence and the Draft/ProductionReady state). It exports plain data/lookup
functions, no I/O:

```rust
pub struct AccuracyCeiling {
    pub lon_arcsec: f64,
    pub lat_arcsec: f64,
    pub dist_km: f64,
    pub lon_speed_arcsec_per_day: f64,
    pub lat_speed_arcsec_per_day: f64,
    pub radial_speed_au_per_day: f64,
}

pub enum BodyClass { Luminary, InnerPlanet, OuterPlanet, Asteroid }

pub fn body_class(body: &CelestialBody) -> BodyClass;
pub fn accuracy_ceiling(body: &CelestialBody) -> AccuracyCeiling; // body -> class -> ceiling

pub struct ArtifactBudgets {
    pub max_encoded_bytes: usize,        // hard
    pub decode_latency_target_ms: f64,   // tracked
    pub single_lookup_target_ms: f64,    // tracked
    pub batch_throughput_target: f64,    // tracked
    pub chart_workload_target_ms: f64,   // tracked
}
pub const PACKAGED_BUDGETS: ArtifactBudgets;
```

**Body classes** (so the contract is per-class, not per-body):
- `Luminary` — Sun, Moon
- `InnerPlanet` — Mercury, Venus, Mars
- `OuterPlanet` — Jupiter, Saturn, Uranus, Neptune, Pluto
- `Asteroid` — Eros

Three consumers read from this module; nothing hardcodes numbers inline:
1. Gate tests — assert `measured <= ceiling` and `encoded_size <= budget`.
2. CLI summaries — render the published contract + live measured + pass/margin.
3. The existing golden drift test stays as a separate regression concern.

The inline ceilings currently living in
`outer_planet_longitude_meets_astrology_grade_envelope` are **moved into this
module** so there is exactly one definition.

## Accuracy thresholds (the published ceilings)

Published per body-class × channel. Measured longitude maxima are all sub-arcsec,
so ceilings carry deliberate headroom — a stable public promise, not a ratchet
(the golden catches drift under the ceiling).

| Class | Longitude | Latitude | Distance | (measured lon, ref) |
| --- | --- | --- | --- | --- |
| Luminary (Sun, Moon) | 1.0″ | 1.0″ | per-class km | ≤ 0.001″ |
| Inner (Mer/Ven/Mars) | 1.0″ | 1.0″ | per-class km | ≤ 0.0011″ |
| Outer (Jup→Pluto) | 5.0″ | 5.0″ | per-class km | ≤ 0.0036″ |

Speed-channel ceilings (lon/lat speed in arcsec/day, radial in km/day) share the
same per-class structure; exact values are finalized from first measurement (see
"Threshold finalization").

Decisions:

1. **Hard accuracy gates cover the 10 major bodies only.** The hold-out corpus
   *is* those 10 bodies. **Eros has no independent truth** — it is re-derived
   from the same committed snapshot it would be checked against. Eros therefore
   gets a **documented/constrained published target plus a self-consistency
   check**, explicitly *not* an independent-truth gate. This preserves the repo's
   existing honesty posture.
2. **Distance = absolute km per class.** The Moon's ~384,000 km and Pluto's
   ~5.9 billion km cannot share one number. The baseline already measures
   absolute km, so the gate stays trivial. Relative/fractional error was
   considered; absolute-per-class is sufficient as a regression guard (distance
   is low-priority for astrology).
3. **Latitude ceilings mirror longitude per class** (measured lat is also
   sub-arcsec).
4. **Exact ceiling values finalized from measurement** during implementation;
   the spec explicitly permits this ("thresholds should be finalized through
   validation data"). The round numbers above are the published contract; the
   implementation verifies measured sits comfortably under each.

### Threshold finalization

For each channel, the published ceiling is a round number chosen with a clear
margin (target ≥ ~10× headroom for position channels) over the measured maximum
across the hold-out, so routine regeneration does not nudge the public contract.
Speed ceilings are set the same way once speed is first measured against velocity
truth. The golden drift test (below) pins the measured values tightly so a
regression that is still under the ceiling cannot pass silently.

## Motion/speed implementation

### Deriving speed (no new stored bytes)

Each `PolynomialChannel` (lon/lat/dist) is a polynomial in normalized segment
time, so speed is its analytic time-derivative with the `dt/d(julian_day)`
normalization scaling applied. `SpeedPolicy::FittedDerivative` already exists in
the format. The packaged artifact flips `Unsupported -> FittedDerivative`, and
the lookup path at `lookup.rs:510` (currently `Motion -> Unsupported`) gains a
real motion branch.

### Velocity truth in the corpus

The accuracy gate needs truth velocity at each hold-out epoch; de440/SPK provides
position + velocity. Plan:

- Add an **optional** velocity field to the corpus row (`SnapshotEntry`),
  **populated for the de440-sourced major-body hold-out slice** (the slice the
  accuracy gate consumes), left `None` where the source cannot supply it
  (Horizons / sb441 fixtures).
- Regen is kernel-gated (`PLEIADES_DE_KERNEL`), checksums re-pinned, and the
  fail-closed `validate-corpus` gate updated to verify the new column.
- The accuracy gate *requires* velocity present for the 10 majors.

This keeps the baseline kernel-free at test time and limits blast radius to one
schema field plus the hold-out slice, rather than rewriting every slice.

### Units & capability claim

- Public speed values use the existing `pleiades_types::Motion` type:
  `longitude_deg_per_day`, `latitude_deg_per_day`, `distance_au_per_day` — i.e.
  **°/day** (lon/lat) and **AU/day** (radial). (My earlier km/day note is
  superseded by the real type.)
- Error thresholds: **arcsec/day** (lon/lat speed), **AU/day** (radial speed),
  per body class.
- Coverage profile flips `Motion -> Derived`; the `speed_policy` summary becomes
  `FittedDerivative`; the "motion rejected" profile gate (`profile.rs:869`) and
  its tests flip to "motion derived." This is the published-capability change
  recorded against Phase 4.

### Versioning caveat

Artifact *bytes* do not change (speed is derived, not stored). If `speed_policy`
is serialized into the artifact, bump `ARTIFACT_VERSION 6 -> 7` and regen; if it
is declared in code (coverage profile), no byte change. Confirm which during
implementation and bump only if bytes actually move.

## Size & latency budgets

- **Size — hard gate.** Current encoded artifact ~10.0 MB (1900–2100). Published
  budget e.g. **≤ 12 MB** (headroom). Deterministic gate: `encoded_len <=
  max_encoded_bytes`.
- **Latency — tracked, non-gating.** Published targets (decode, single-lookup,
  batch throughput, chart-workload) live in `PACKAGED_BUDGETS`. A summary command
  reports *measured vs target with margin*. The actual ceiling-check test is
  `#[ignore]` / env-opt-in (e.g. `PLEIADES_ENFORCE_LATENCY`) so CI stays green
  given the known concurrent-load flakiness.

## CLI surfaces & gates

- **CLI:** a `packaged-artifact-thresholds-summary` command rendering the
  published contract + live measured + pass/margin across all four accuracy
  channels and size; plus a tracked latency-budget summary. Dispatch is added in
  `crates/pleiades-validate/src/render/cli.rs` (alongside the existing
  `packaged-artifact-target-threshold-summary` arm), the help string in that
  file's `help_text()` gets the new lines, and the help-sync assertions in
  `crates/pleiades-cli/src/cli/tests/help.rs` are updated. Render functions
  follow the existing `OnceLock` + `validate()` + `summary_line()` + committed-
  golden pattern (see `accuracy_baseline.rs` and `coverage/threshold.rs`).
- **Hard gates (tests):** accuracy-ceiling gate (10 majors × 4 channels),
  size-budget gate, Eros self-consistency check. The existing golden drift test
  is retained (approach A). The latency gate is the opt-in one.

## Documentation reconciliation

Truthfulness of release claims is a cross-cutting plan rule, so SP3 updates:

- `spec/data-compression.md` §Accuracy Targets — replace the "for example"
  envelopes with the actual published per-class thresholds; state the 1900–2100
  first-release window; note 1600–2600 as documented future expansion.
- `plan/stages/02-production-compressed-ephemeris.md` — change the exit-criteria
  window 1600–2600 → 1900–2100; mark SP3 done when gates land.
- `plan/status/01-current-execution-frontier.md` and
  `02-next-slice-candidates.md` — refresh the stale "SP2 is next" text (SP2 done;
  SP3 is the active/closing slice).
- Phase 4 plan doc (`plan/stages/04-advanced-request-modes.md`) — record that
  motion output (FittedDerivative/Derived) was implemented in SP3; narrow the
  remaining Phase 4 motion scope.
- `README.md` and `PLAN.md` — drop "SP3 not yet completed"; note motion
  capability and budgets.

## Testing strategy (TDD)

- **Unit:** derivative math (analytic speed vs finite-difference of the same
  polynomial), °/day and km/day unit conversions, `accuracy_ceiling(body)` class
  mapping, budget constants.
- **Integration:** accuracy gate across all four channels including speed vs
  velocity truth; size-budget gate; Eros self-consistency; coverage-profile flip
  (`Motion = Derived`, `FittedDerivative`); corpus velocity column +
  `validate-corpus` gate update; golden drift retained/extended.
- **Determinism:** artifact regen byte-identity (or +version bump if
  `speed_policy` is serialized); corpus regen kernel-gated.
- **Known caveats carried in:** benchmark/latency tests stay `#[ignore]`/opt-in
  (documented flakiness); do not run `cargo` concurrently with subagents during
  accuracy regen (per the SP2 SDD log).

## Exit criteria

- Published per-class × per-channel accuracy thresholds (lon, lat, distance,
  speed) defined in one module and enforced by hard gates for the 10 major bodies
  over 1900–2100.
- Eros published as a documented/constrained target with a self-consistency
  check (not an independent-truth gate).
- Motion output implemented (`FittedDerivative`, `Motion = Derived`) with speed
  validated against de440 velocity truth.
- Encoded size hard-gated under budget; latency targets published and tracked.
- Spec, plan stage/status, Phase 4 plan, README, and PLAN.md reconciled to the
  1900–2100 window and the new motion capability.
- Deterministic regeneration / byte (or versioned) verification intact.
