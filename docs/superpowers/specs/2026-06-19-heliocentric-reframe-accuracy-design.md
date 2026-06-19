# Phase 2 SP2 — Heliocentric Re-Frame for Outer-Planet Accuracy

Status: design approved (brainstorming), pending implementation plan
Date: 2026-06-19
Scope: `pleiades-data`, `pleiades-compression`, `pleiades-jpl`
Relates to: `plan/stages/02-production-compressed-ephemeris.md` (SP2 accuracy tuning),
`spec/data-compression.md` (accuracy targets)

## Problem

The packaged 1900–2100 artifact fits **geocentric** ecliptic coordinates directly with
low-order per-segment polynomials plus optional residual channels and a curvature-driven
adaptive splitter. The measured per-body accuracy baseline
(`crates/pleiades-data/src/accuracy_baseline.rs`, decoded artifact vs. the de440-derived
hold-out) shows:

- inner bodies + Sun + Moon: sub-arcsecond longitude;
- outer planets: draft-level — Uranus ~192″, Neptune ~109″, Pluto ~62″, Saturn ~9.5″,
  Jupiter ~1.5″.

Root cause: a planet's **geocentric** longitude inherits the ~1-year retrograde signal
driven by Earth's orbit. Over an outer-planet segment span (currently 768 days, ~2 annual
cycles) a low-order polynomial cannot represent that oscillation. Inner bodies and the
Moon do not suffer because their own motion dominates the same span. The adaptive-split
and residual machinery is fighting a signal that should not be in the fit at all.

The spec's own intuition — *"for outer planets, longer segments are acceptable"* — is true
only in a **heliocentric** frame, where outer-planet motion is smooth, slow, and nearly
polynomial. That observation is the design.

## Target

Astrology-grade longitude envelope for the packaged 1900–2100 major bodies:

- inner planets, Sun, Moon: sub-arcsecond (no regression);
- outer planets: a few arcsec (≤ ~5″), down from ~100–190″.

Exact published thresholds and size/latency budgets are finalized in SP3; this design
delivers the accuracy mechanism and the before/after measurement, not the enforced gate.

## Approach (chosen)

Fit the planets where their motion is smooth (heliocentric), keep the Sun and Moon stored
exactly as today (geocentric), and recombine at lookup with a vector identity.

### Core identity

For planet `P`, Sun `S`, Earth `E`, in a common ecliptic Cartesian frame at one instant:

- heliocentric (to be stored): `P_h = P − S`
- geocentric Sun (already stored): `S_g = S − E`
- geocentric planet (desired output): `P_g = P − E = (P − S) + (S − E) = P_h + S_g`

A planet's geocentric position is its stored heliocentric vector **plus** the stored
geocentric-Sun vector. No "Earth" body is added; the existing Sun channel is the
Earth-position reference. The annual retrograde signal is contributed entirely by the
Sun channel, which is already sub-arcsecond, so the geocentric error budget becomes
"heliocentric fit error ⊕ Sun fit error" — both small.

### What changes vs. what stays

| Body | Stored frame | Lookup |
| --- | --- | --- |
| Sun | geocentric (unchanged) | direct, as today |
| Moon | geocentric (unchanged) | direct, as today |
| Mercury–Pluto | heliocentric (new) | reconstruct `P_h + S_g` → ecliptic |
| Eros (asteroid) | heliocentric (new) | same reconstruction |

The codec, channel structure (3 spherical `PolynomialChannel`s: lon/lat/r per segment),
quantization (`scale_exponent`), residual-channel mechanism, span limits, and the Moon/Sun
pipelines are all unchanged. Only the *meaning* of the planet coefficients changes, plus a
reconstruction step at lookup.

## Artifact format

1. **Per-body stored frame.** Add an explicit tag to `BodyArtifact` (body-level, not
   per-segment — a body is fit in one frame throughout):

   ```
   enum StoredFrame { Geocentric, Heliocentric }
   ```

   Sun, Moon → `Geocentric`; Mercury–Pluto, Eros → `Heliocentric`. This keeps the artifact
   self-describing and makes the lookup branch on data, not a body-name allowlist. It also
   admits future bodies without code changes.

2. **`ARTIFACT_VERSION` bump 5 → 6.** Payload gains the per-body frame byte. Decoders reject
   other versions as today; the kernel-gated reproduce test re-pins the new bytes.

3. **Profile / summaries.** Generation-provenance and body-cadence summaries gain a frame
   column so the published posture states "planets stored heliocentric, recombined with the
   geocentric Sun reference at lookup" (transparent-provenance requirement).

4. **Output contract unchanged.** `lookup_ecliptic` still returns geocentric
   `EclipticCoordinates`; `lookup_equatorial` and batch paths keep identical signatures.
   Callers see only better outer-planet numbers.

## Generation pipeline

1. **New sampling primitive** (`pleiades-jpl/src/spk/chain.rs`). Mirror the existing
   `geocentric_icrf` (which is `position_wrt_ssb(target) − position_wrt_ssb(399 = Earth)`):

   ```
   heliocentric_icrf(pool, target, et) = position_wrt_ssb(target) − position_wrt_ssb(10 = Sun)
   ```

   then reuse `icrf_to_ecliptic` to produce heliocentric ecliptic lon/lat/r. This is the only
   genuinely new astronomy code and it mirrors an existing function.

2. **Thread the frame choice up.** The dense-fit sampler in `pleiades-data/src/regenerate.rs`
   (`sample_fraction` → `reference_backend.position(...).ecliptic`) requests heliocentric for
   heliocentric-frame bodies by threading the body's `StoredFrame` into the sample request so
   the SPK backend resolves the right center. Geocentric stays the default; Sun/Moon untouched.

3. **Fitting reuses everything.** Segment fitting, span limits, residual layering, and the
   error-driven acceptance ratio (`PACKAGED_ARTIFACT_SEGMENT_FIT_ACCEPTANCE_RATIO`) are
   unchanged; they operate on smooth heliocentric samples and naturally emit fewer outer-planet
   segments (artifact shrinks, not grows).

4. **Eros.** Fit heliocentrically from its committed reference snapshot (absent from
   de440/sb441), reconstructing heliocentric ecliptic from the snapshot vectors.

5. **Span limits unchanged in this change.** Keep current per-body span limits and rely on
   error-driven acceptance; defer any upward re-tuning (smaller artifact) to the SP3 size pass
   so the accuracy re-frame stays cleanly isolated.

6. **Determinism.** Generation stays kernel-gated (`PLEIADES_DE_KERNEL`) and byte-deterministic;
   `artifact_regen.rs` re-pins the new bytes and checksum.

## Lookup & reconstruction

In `pleiades-compression`'s `lookup_ecliptic`, branch on the body's `StoredFrame`:

- `Geocentric` (Sun, Moon): evaluate the 3 channels → `EclipticCoordinates`. Exactly today.
- `Heliocentric` (planets, Eros): reconstruct.

Reconstruction for a heliocentric body at instant `t`:

```
1. Evaluate body channels → heliocentric (lon_h, lat_h, r_h)
2. Look up Sun at t → geocentric Sun (lon_s, lat_s, r_s)   // recursive Geocentric path
3. Convert both spherical → ecliptic Cartesian, one consistent unit (AU)
4. P_geo_cart = P_helio_cart + S_geo_cart
5. Convert P_geo_cart → ecliptic (lon, lat, dist)
6. Return EclipticCoordinates
```

- Spherical↔Cartesian is plain trig with **no frame rotation**: heliocentric and geocentric
  ecliptic share the same mean-ecliptic axes at the same instant. This co-frame assumption is
  the one correctness-critical invariant; it is documented and pinned by a test.
- **Sun dependency.** A planet lookup performs a Sun lookup at the same instant. Therefore the
  artifact must contain the Sun whenever it contains any heliocentric body — enforced as a
  structural invariant in `validate()` (fail-closed).
- **Cost.** One extra segment search + 3 polynomial evals + ~10 flops per planet lookup;
  negligible vs. the ~3.3 ms single-lookup baseline. Re-measured for the SP3 latency record.
- **Units.** Combine in AU; heliocentric distances (tens of AU) and the ~1 AU Earth–Sun term
  are well within f64 precision at arcsec level — no catastrophic cancellation.
- **No API surface change.**

## Error handling & invariants (fail-closed)

- Heliocentric body present but no Sun body → `validate()` rejects at decode.
- Sun lookup failure inside a reconstruction (e.g. instant outside Sun coverage) propagates as
  the existing `CompressionError`, not a partial result.
- Unknown/future `StoredFrame` byte → decode error, consistent with the version gate.
- Co-frame round-trip test: a known de440 epoch reconstructed via the identity matches the
  direct geocentric de440 sample to sub-arcsecond.

## Validation (the payoff and the gate)

- Re-run the per-body accuracy baseline (`accuracy_baseline.rs`, decoded artifact vs.
  de440-derived hold-out) before/after. Outer-planet longitude must drop from ~100–190″ into
  the few-arcsec target; inner/Sun/Moon must not regress.
- Commit the new baseline numbers (the SP2 deliverable) and update
  `packaged-artifact-accuracy-baseline-summary`.
- Add a regression test asserting each major body stays under its target envelope, so future
  regen cannot silently degrade.
- Kernel-gated reproduce test re-pins artifact bytes/checksum; `validate-corpus` and
  stale-sidecar gates continue to fail closed.

## Docs to keep aligned (PLAN.md maintenance rules)

- `plan/stages/02-production-compressed-ephemeris.md` SP2: record the re-frame as the accuracy
  mechanism and the new outer-planet numbers.
- `README.md` + `PLAN.md` size/perf/accuracy baseline table.
- `spec/data-compression.md`: document heliocentric storage + geocentric reconstruction and the
  co-frame invariant.

## Out of scope (deferred, named so they are not silently assumed done)

- SP3 threshold *enforcement* and size/latency *budgets*.
- Span-limit re-tuning (artifact size reduction).
- Chebyshev representation replacing the bespoke power-basis scheme.
- Pluto promotion (stays approximate / fallback-backed).
- Apparent-place / light-time / topocentric / native sidereal (Phase 4).

## Exit criteria for this change

- Planets and Eros stored heliocentric with an explicit per-body `StoredFrame`;
  `ARTIFACT_VERSION` 6; deterministic kernel-gated regeneration verified.
- `lookup_ecliptic` reconstructs geocentric for heliocentric bodies via `P_h + S_g`, with the
  Sun-presence structural invariant enforced.
- Per-body accuracy baseline re-measured and committed: outer planets ≤ ~5″ longitude, no
  regression on inner/Sun/Moon; regression test in place.
- README/PLAN/spec/profile summaries aligned with the new model.
