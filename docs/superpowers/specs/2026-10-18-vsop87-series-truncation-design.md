# Truncated VSOP87B series for faster planetary samples (design)

**Status:** design approved in chat, document under review · **Opened:** 2026-10-18 ·
**Issue:** #281 (reported by downstream user starfold) ·
**Crates:** `pleiades-vsop87` (code), `pleiades-validate` / docs (evidence)

## Intent

Starfold's slowest readings (Gochara 550 ms, Progressions 555 ms, Transits
183 ms on pleiades-core 0.11.0) are long scans over apparent planetary
longitudes from `CompositeBackend[ElpBackend, Vsop87Backend]`. On the web they
run on the main thread, so the page freezes for the whole reading. One apparent
planet sample (`EventEngine::longitude_at`) costs 130–330 µs. Swiss
Ephemeris' Moshier ephemeris costs 13–17 µs for the same quantity.

Almost all of that time is `cos` calls in the full, untruncated VSOP87B series
of the planet and the Earth. Most of those terms contribute far less than a
milli-arcsecond. The fix is to stop evaluating them by default, with a drift
that is measured, small against errors pleiades already has, and reported.

## Measurements (throwaway probe, 2026-10-18)

The probe sources and raw outputs are in a session scratchpad and were not
committed. The builder re-derives what the gates need. The method was:

- **Truncation rule:** keep a term when `|A| · t_max^α ≥ ε`, where `t` is in
  Julian millennia from J2000 and `α` is the term's power of `t`. The rule is
  applied to the L, B and R series (R in au) of both the planet and the Earth.
- **Truncation drift:** geocentric geometric J2000 longitude, truncated minus
  full, over 20,000 jittered instants per window. Each max is a sampled max,
  not a bound. The probe's full evaluator was bit-identical to
  `Vsop87Backend::position_without_motion`.
- **Context columns:** apparent-of-date longitude against DE440.
  - pleiades: the pleiades apparent pipeline run with `Vsop87Backend` and with
    `SpkBackend(de440)`, so only the ephemeris differs.
  - SE: `swe_calc(SEFLG_MOSEPH)` minus SE's own `SEFLG_JPLEPH` DE440, so both
    go through SE's reduction.
- **Timing:** one geometric planet+Earth evaluation, taken on a loaded machine
  (load average 41–69). Trust the ratios; the absolute µs are rough.

Values are worst-case geocentric longitude in arcseconds.

**1900–2100**

| body | today's VSOP87 − DE440 | SE Moshier − DE440 | drift ε=1e-7 | drift ε=1e-8 | drift ε=1e-9 |
|---|---|---|---|---|---|
| Sun | 0.11 | 0.10 | 0.33 | 0.04 | 0.005 |
| Mercury | 0.14 | 0.14 | 0.52 | 0.06 | 0.009 |
| Venus | 0.13 | 0.56 | 1.01 | 0.16 | 0.016 |
| Mars | 0.26 | 1.11 | 1.28 | 0.27 | 0.023 |
| Jupiter | 0.53 | 0.79 | 0.34 | 0.05 | 0.004 |
| Saturn | 0.43 | 0.64 | 0.36 | 0.05 | 0.006 |
| Uranus | 1.46 | 0.68 | 0.28 | 0.04 | 0.004 |
| Neptune | 2.45 | 2.08 | 0.19 | 0.03 | 0.003 |

**1600–2600 (cutoff computed with t_max = 0.6)**

| body | today's VSOP87 − DE440 | SE Moshier − DE440 | drift ε=1e-8 | kept terms planet+Earth (of full) | geometric eval µs, full → ε=1e-8 |
|---|---|---|---|---|---|
| Sun | 0.27 | 0.13 | 0.05 | 578 (2564) | 56 → 12 |
| Mercury | 0.39 | 0.22 | 0.07 | 1250 (9687) | 202 → 24 |
| Venus | 0.50 | 0.66 | 0.17 | 997 (4274) | 82 → 22 |
| Mars | 8.06 | 1.24 | 0.25 | 1968 (8964) | 203 → 45 |
| Jupiter | 1.02 | 2.09 | 0.06 | 2232 (6189) | 140 → 50 |
| Saturn | 1.45 | 0.64 | 0.09 | 3693 (8929) | 190 → 72 |
| Uranus | 10.27 | 3.51 | 0.07 | 3737 (7833) | 163 → 74 |
| Neptune | 8.69 | 6.88 | 0.04 | 1856 (4588) | 91 → 33 |

Latitude drift is about half the longitude drift (at ε=1e-8 over 1900–2100 it
peaks at 0.14″, for Mars).

**Where else the time could go.** These look like levers but are not. The
light-time loop deliberately re-reads the Earth together with the body at
`t − τ`, because that retarded direction is how annual aberration enters (#93).
So the Earth evaluation per query is necessary work. The loop already
interpolates instead of re-querying once it has two samples (#128), and #247
removed the redundant queries. A sample costs about 1.6 geocentric
evaluations today (333 µs for Mercury, against 208 µs per evaluation).
Truncation is the lever.

## Decisions (made with the maintainer)

1. **Default to ε = 1e-8, with the cutoff computed for 1600–2600
   (`t_max = 0.6`).** That window is the project's long-term target, so drift
   stays at 0.27″ or less everywhere inside it, not only near 2000. The
   bar is "no worse than SE Moshier, and not meaningfully worse than pleiades
   against DE440 today". At ε=1e-8 the drift RMS (≈0.01–0.02″) is small
   against today's RMS error vs DE440 (0.08–1.4″). Mars is the closest case:
   its worst case of 0.27″ is about the size of its current error.
2. **Rejected ε = 1e-7.** It would come close to SE's speed, but Venus and Mars
   would move by about 1″, measurably worse than today and than SE.
3. **Keep the full series as an explicit opt-in.** A caller who wants today's
   output bit for bit can still get it. `Vsop87Backend::new()` becomes the
   truncated backend.
4. **Outside 1600–2600 the default backend uses the full series.**
   `Vsop87Backend` places no limit on dates (`nominal_range` is unbounded).
   The terms that are dropped grow as `(|t|/0.6)^α` beyond the window: about
   72× for an α=3 term at 500 BCE. So the default evaluates the truncated
   series for instants inside 1600–2600 CE and the full series outside. Output
   outside the window is bit-identical to today's. There is a step of up to
   about 0.27″ at the two window edges; document it in the backend's
   rustdoc, and cover it with a test at each edge.
5. **Report the change.** The release notes state the measured change in
   returned longitudes against current output (starfold asked for this) and
   the new per-sample cost.

## Settled during review (author's calls, not discussed with the maintainer)

- **No source-breaking change.** `Vsop87Backend` stays a unit struct, and bare
  `Vsop87Backend`, `Vsop87Backend::new()` and `Default` all give the default
  (truncated inside the window) behaviour. The full-series opt-in is a
  separate small type implementing `EphemerisBackend`.
- **Validations that compare the default backend with Swiss Ephemeris stay on
  the default backend.** These are the pheno, nod-aps, occultation and
  frame-consistency validations: they should validate what users get. If drift
  pushes a check past its margin, widen that tolerance by no more than the
  measured drift and record the justification next to the tolerance and in
  the PR. Do not move such a check to the full-series type to dodge it.
  Canonical-evidence checks that assert reproduction of the source files
  (bit-for-bit against VSOP87B) belong on the full series.
- **Metadata.** The default backend's source profiles and quality must
  describe the truncated series honestly; the full-series type keeps today's
  full-series descriptions.

## Goals

- Apparent planetary samples on the default `Vsop87Backend` are several times
  cheaper. The expectation from the measurements is roughly 40–100 µs instead
  of 130–330 µs; measure it, don't assume it.
- The drift from the full series is bounded and gated. A test fails if a
  regeneration or a cutoff change pushes it past the documented bound.
- Every claim about VSOP87 accuracy, provenance and truncation in docs and
  metadata stays true.

## Non-goals

- Matching SE Moshier's 13–17 µs. That needs either ε=1e-7, which was
  rejected, or a different evaluation technique.
- Changes to the apparent pipeline, ELP (the Moon is already on par with SE),
  the Pluto paths, or the true node.
- Pointing starfold at `PackagedDataBackend` or other backends. This change is
  about the algorithmic backend they chose.

## Anti-goals (a failure even if it is faster)

- Any body drifting by more than SE Moshier's own error against DE440, or by
  more than about 0.3″ anywhere in 1600–2600.
- Silently changing output: no gate, no changelog entry, or stale
  "full, untruncated series" wording left in docs and metadata.
- Losing the ability to reproduce today's output.

## Constraints

- Pure Rust, no new dependencies.
- Generated data stays whole, per AGENTS.md. Do not hand-edit or re-split the
  `data/VSOP87B.*` files or the tables' data modules, so regeneration tooling
  keeps round-tripping.
- Behaviour change to public output. Follow the release-plz conventions in
  `docs/release-process.md` (conventional commit type and scope) and do not
  bump versions by hand.
- Blocking CI must stay green. Regenerate any golden or corpus that pins
  VSOP87 output in the same PR as the change.

## Left to the builder

- **Where truncation happens.** Two options:
  - Filter the existing full tables when they are loaded (the `OnceLock`
    init). This means no new artifacts, and both variants come from one
    source.
  - Emit separate truncated tables from `regenerate-vsop87b-tables`.

  Either respects the decisions above. Filtering at load is the smaller change.
- **The opt-in API:** its name and shape (constructor, configuration value),
  and how `CompositeBackend` users reach it.
- **How metadata reflects truncation.** The relevant places are
  `Vsop87BodySourceKind` (a `TruncatedVsop87b` kind already exists),
  `truncation_policy` strings, the source-docs and evidence reports, and the
  per-result `QualityAnnotation`.
- **The form of the drift gate:** a grid, its density, and the bound per body.
  The probe's figures are the starting point.
- **How to measure the before/after per-sample cost.** Starfold's probe shape
  works: `longitude_at` over 4,000 scattered and daily instants.
- **Finding and regenerating** every test, golden or validation corpus whose
  expectations move.
