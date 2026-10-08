# Data Compression Specification

Unless stated otherwise, the conformance terms defined in [`SPEC.md`](../SPEC.md) apply here.

## Objective

Define a compressed ephemeris representation optimized for astrology software doing frequent lookups in the date range **1600-2600 CE**.

## Design Goals

- compact enough for local bundling in desktop/mobile/server applications
- fast random access by body and timestamp
- deterministic decode behavior in pure Rust
- accuracy suitable for astrological chart generation
- regenerable from public astronomical source data

## Representation Strategy

The preferred design is a **segmented polynomial/residual format**:

1. Divide the time span into fixed or body-specific segments.
2. For each body and coordinate channel, fit a low-order polynomial or Chebyshev approximation over the segment.
3. Quantize coefficients into integer fields with per-segment scale metadata.
4. Store optional residual correction tables for high-curvature bodies, especially the Moon.
5. Store only the channels required to support the advertised result set for the artifact profile.

## Why This Design

Compared with raw sampled tables, polynomial segments:

- reduce storage significantly
- preserve fast evaluation
- support predictable interpolation error
- compress especially well when paired with delta and entropy coding

## File Layout

A compressed artifact should contain:

- file header with magic/version/endian policy
- source provenance and generation metadata
- body index table
- segment directory for each body/channel
- coefficient blocks
- optional residual blocks
- checksums
- an artifact capability/profile section describing which outputs are stored directly and which are reconstructed at query time

## Access Pattern

The runtime lookup algorithm should:

1. resolve body id
2. locate segment by time
3. decode quantized coefficients
4. evaluate polynomial
5. apply residual correction if present
6. reconstruct any derived outputs promised by the artifact profile
7. normalize angle output

## Compression Techniques

The implementation may combine:

- integer quantization with per-block scale factors
- delta encoding between adjacent coefficients
- variable-length integer packing
- entropy coding if justified by complexity/performance tradeoff
- memory-mappable read layouts where practical

## Stored vs Derived Outputs

Packaged artifacts do not need to store every field of the backend result model verbatim.

Instead, each artifact profile must declare:

- which coordinate channels are stored directly
- which channels are derived deterministically during decode
- which optional outputs are unsupported by that artifact

Every built-in artifact output must be classified explicitly; profiles must not leave a built-in output implicitly unlisted.

Profiles may also mark an output as approximated when it is reconstructed deterministically by a numerical approximation rather than by a direct stored or analytic derivation. For example, motion values may be approximated from neighboring decoded samples while ecliptic longitude, latitude, and distance are stored directly and equatorial coordinates are derived from those values plus auxiliary parameters available to the runtime.

If speed values are provided, the profile must state whether they are:

- stored directly,
- derived from fitted derivatives, or
- approximated numerically from neighboring samples.

This keeps the artifact format smaller while still making result semantics explicit.

## Per-Body Storage Frame

Each body in the artifact carries a `StoredFrame` tag that controls how its
stored ecliptic coordinates are interpreted at lookup time.

### Heliocentric storage (planets Mercury–Pluto and the dense asteroids)

The eight major planets (Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune,
Pluto) and the five dense asteroids (Ceres, Pallas, Juno, Vesta and 433-Eros,
fitted from the JPL `sb441-n373s` kernel) are stored **heliocentrically**: their ecliptic longitude, latitude, and
distance channels represent the planet's position relative to the Sun, not the
Earth.

At lookup the runtime reconstructs geocentric ecliptic coordinates via Cartesian
vector addition:

```
P_geo = P_helio + S_geo
```

where `P_helio` is the planet's heliocentric Cartesian position and `S_geo` is
the geocentric Cartesian position of the Sun, both decoded from the artifact at
the same epoch.

**Co-frame invariant.** Both channels are stored in the same reference frame —
J2000 ecliptic Cartesian (a fixed-ε₀ reduction) — so their Cartesian sum is
valid in-frame with no obliquity rotation at lookup.  The planet-heliocentric
channel and the Sun-geocentric channel are co-frame by construction: de440
provides the planets and the Sun, and for the five asteroids sb441-n373s
provides the heliocentric state while de440 provides the Sun; all are reduced
to the same J2000 ecliptic frame, the artifact fits them in that frame, and the
runtime adds them in that frame.

### Geocentric storage (Sun, Moon) — `StoredFrame::Geocentric`

The Sun and Moon carry `StoredFrame::Geocentric`: their channels
represent the body's position relative to the Earth directly.  No
Sun-subtraction reconstruction is applied at lookup; the stored coordinates are
returned as-is.

### Sun-presence structural invariant

An artifact that contains one or more heliocentric bodies **must** include the
Sun stored geocentrically.  The codec enforces this as a fail-closed invariant
at artifact-construction time: if any body carries `StoredFrame::Heliocentric`
and the Sun is absent or is itself stored heliocentric, construction is rejected
with an error.  This prevents a lookup-time reconstruction failure where a
heliocentric body has no geocentric Sun reference to add to.

## Channel Recommendations

### Slow-Moving Bodies

For outer planets and many asteroids, longer segments are acceptable.

### Fast/Irregular Bodies

For the Moon and some near-Earth-sensitive quantities, use shorter segments and/or higher-order fits with residual correction.

## Accuracy Targets

The packaged artifact (first release: **1900–2100 CE**; documented future expansion to 1600–2600 CE
is planned but not yet gated) publishes per-body-class accuracy ceilings enforced as a CI gate.
These ceilings are the public contract, defined in `pleiades-data::thresholds::accuracy_ceiling`
and measured against the de440-derived hold-out corpus.

### Published ceilings (SSOT: `crates/pleiades-data/src/thresholds.rs`)

| Class | Bodies | lon ≤ | lat ≤ | dist ≤ | lon/lat speed ≤ | radial speed ≤ |
|-------|--------|-------|-------|--------|-----------------|----------------|
| Luminary | Sun, Moon | 1.0″ | 1.0″ | 50 km | 0.5 ″/day | 1×10⁻⁴ AU/day |
| Inner planet | Mercury, Venus, Mars | 1.0″ | 1.0″ | 50 km | 0.5 ″/day | 1×10⁻⁴ AU/day |
| Outer planet | Jupiter, Saturn, Uranus, Neptune, Pluto | 5.0″ | 5.0″ | 1,000 km | 0.05 ″/day | 1×10⁻⁴ AU/day |
| Asteroid | Ceres, Pallas, Juno, Vesta, Eros | 30″ | 30″ | 5,000,000 km | 120 ″/day | 1×10⁻² AU/day |

**Asteroid note:** the five asteroids are not checked against the hold-out corpus; their
evidence is the blocking gate `packaged_asteroids_match_the_sb441_rows`
(`crates/pleiades-data/src/tests/asteroid_gate.rs`), which compares the packaged backend with
the 407 `sb441-n373s` rows per body of `crates/pleiades-jpl/data/corpus/asteroid_reference.csv`
against `ASTEROID_CORPUS_CEILING` (measured maximum x 1.4, rounded up): 0.42″ longitude x
cos latitude and 0.38″ latitude. That is the row-gate ceiling against those corpus rows, not a
bound on every date. Row maxima: 0.2938″ longitude (Juno) and 0.2646″ latitude (Ceres). Dense
0.5-day sampling against the kernel over 1900-2100
(`docs/superpowers/specs/notes/2026-10-08-asteroid-span-measurement.md`) finds larger maxima:
at most 0.52″ longitude (Juno 0.5131″) and 0.37″ latitude (Juno 0.3682″) across the five bodies,
inside the 1″ acceptance rule. They are excluded from the fit envelope. The slow claims audit
compares them with the looser per-class `accuracy_ceiling` (30″, the table row above).

**Size budget:** Encoded artifact ≤ 12,000,000 bytes (measured ~10.4 MB, 10,421,155 B); enforced as a hard CI
gate via `PACKAGED_BUDGETS.max_encoded_bytes`.

**Latency budget:** Decode/single-lookup/batch targets are tracked in `PACKAGED_BUDGETS`
(decode ≤ 400 ms, single lookup ≤ 6 ms, batch ≥ 1,000 lookups/s, chart workload ≤ 50 ms).
Latency is not a hard CI gate by default; opt-in enforcement via `PLEIADES_ENFORCE_LATENCY`.

**Motion output:** Speed channels (lon/lat speed, radial speed) are classified as `Motion = Derived`
in the artifact profile, computed via `SpeedPolicy::FittedDerivative`.

Every packaged artifact must publish measured error against its generation source.

## Generation Pipeline

The artifact builder must:

- ingest authoritative source backend outputs
- fit segments deterministically
- emit validation summaries
- stamp source versions and generation parameters
- produce reproducible binary artifacts

## Crate Responsibility

- `pleiades-compression`: codec, fit/evaluate logic, artifact format
- `pleiades-data`: bundled artifacts and a backend that reads them
