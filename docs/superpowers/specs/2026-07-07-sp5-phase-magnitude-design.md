# SP-5 — Phase, Phase Angle & Magnitude (`swe_pheno` analogue)

Phase: Event-engine track (SP series), slice SP-5 — the second of the two
"astronomy-flavored" engine gaps deferred out of SP-3 (`swe_pheno`; the first,
`swe_nod_aps`, shipped as SP-4). This closes the pair.

## Summary

Add a Swiss-Ephemeris `swe_pheno()` analogue: for a body and instant, return the
five illumination/apparition quantities — **phase angle, illuminated fraction
(phase), elongation, apparent disc diameter, and apparent magnitude** — as a
structured `PhenoData`. Full 5-output SE parity for the ten major bodies
(Sun, Moon, Mercury–Pluto), which are the bodies for which SE ships named
photometric magnitude models.

The surface is an **engine function** on `EventEngine`, not new `CelestialBody`
variants, matching the shape of the rest of the SP-series (crossings, rise/set,
nodes/apsides). Implementation is Approach A: a `pheno.rs` module plus a
`magnitude.rs` submodule inside `pleiades-events`, reusing the crate's existing
apparent-position helpers and physical-radius table.

## SE function targeted

`swe_pheno()` / `swe_pheno_ut()`, whose `attr[20]` return array carries:

| SE slot | Meaning | This design field |
| --- | --- | --- |
| `attr[0]` | phase angle (Sun–planet–Earth), degrees | `phase_angle_deg` |
| `attr[1]` | phase (illuminated fraction of disc), 0–1 | `phase_fraction` |
| `attr[2]` | elongation (Sun–Earth–planet), degrees | `elongation_deg` |
| `attr[3]` | apparent diameter of disc, **arcsec** | `apparent_diameter_arcsec` |
| `attr[4]` | apparent magnitude | `apparent_magnitude` |

`swe_pheno` returns no rates/speeds for these quantities, so `PhenoData` carries
none either (this is the notable structural difference from SP-4's `NodApsPoint`,
which did carry central-difference speeds).

## Public API (in `pleiades-events`)

```rust
pub struct PhenoData {
    /// Sun–body–Earth phase angle, degrees, [0, 180].
    pub phase_angle_deg: f64,
    /// Illuminated fraction of the disc, [0, 1] = (1 + cos phase_angle) / 2.
    pub phase_fraction: f64,
    /// Sun–Earth–body elongation, degrees, [0, 180].
    pub elongation_deg: f64,
    /// Apparent angular diameter of the disc, arcseconds (SE attr[3] unit).
    pub apparent_diameter_arcsec: f64,
    /// Apparent visual magnitude. `None` where no photometric model exists
    /// (any body outside the ten majors); `Some` for Sun/Moon/Mercury–Pluto.
    pub apparent_magnitude: Option<f64>,
    /// The body actually served.
    pub body: CelestialBody,
}

impl<B: EphemerisBackend> EventEngine<B> {
    pub fn pheno(
        &self,
        body: CelestialBody,
        jd_tdb: f64,
    ) -> Result<PhenoData, EventError>;
}
```

### Why `Option<f64>` for magnitude

The crate's convention is never-NaN, fail-closed outputs. Magnitude is the only
one of the five outputs that requires a per-body photometric model, and SE ships
named models only for the ten majors. Rather than emit a NaN sentinel or a
bogus number for bodies with no model, `apparent_magnitude` is `None` there. It
is `Some` for every one of the ten majors (the gated set), so the common case is
unaffected. The four geometric outputs are always populated.

## Computation

Frame/time: **geocentric apparent-of-date**, `jd_tdb`, matching the engine's
other surfaces and SE's default `SEFLG_MOSEPH` call. Geometry is the apparent
Sun–body–Earth triangle, reusing the existing
`geocentric_apparent_ecliptic` helper:

- `Δ` = apparent geocentric distance of the body (AU)
- `R` = apparent geocentric distance of the Sun = Earth–Sun distance (AU)
- `r` = heliocentric distance of the body (AU). Primary path: the body's
  heliocentric apparent position (geocentric-apparent body vector minus the
  geocentric-apparent Sun vector). It is equivalently recoverable from the
  triangle, but the vector path is authoritative.

From these:

- Phase angle: `cos α = (r² + Δ² − R²) / (2 r Δ)`
- Elongation: `cos ε = (Δ² + R² − r²) / (2 Δ R)`
- Phase fraction: `k = (1 + cos α) / 2`
- Apparent diameter: `2 · semidiameter_deg(Δ)`, converted to arcsec. This
  reuses the existing per-body physical-radius table (`radius_au`), promoted
  from module-private to shared crate visibility.

All three law-of-cosines arguments are clamped to `[-1, 1]` before `acos` to keep
outputs never-NaN under pathological near-degenerate geometry, per the crate's
fail-closed convention.

### Magnitude model (`magnitude.rs`)

Per-planet photometric coefficient tables **transcribed verbatim from Swiss
Ephemeris 2.10.03 (`sweph.c`, `swe_pheno` magnitude branch)**, of the form

```
m = 5·log10(r · Δ) + a₀ + a₁·α + a₂·α² + a₃·α³
```

per body (`α` = phase angle), with the special cases SE encodes:

- **Sun** and **Moon**: SE's special-cased forms (the Sun is the light source;
  the Moon uses its own phase-dependent law).
- **Saturn**: the additional **ring term**, computed from the Saturnicentric
  opening of the ring plane as seen from Earth and Sun. This is the most complex
  and highest-residual term in the model.

Transcribing SE's own coefficients and branch structure is what guarantees
parity; the committed reference corpus (below) pins the numeric agreement and
fails closed on any drift.

## Coverage & edge cases

- **Sun**: phase angle 0, elongation 0, phase 1, disc from the radius table,
  SE's solar magnitude.
- **Moon**: full model over the geocentric Sun–Moon–Earth triangle.
- **Mercury–Pluto**: full model; the inner planets exercise the full phase-angle
  range through inferior conjunction.
- **Earth**: not geocentrically addressable → `EventError` (mirrors SP-4's
  "Earth not addressable" handling).
- **Any other backend-served body** (Tier-A asteroids, `seorbel.txt` fictitious
  bodies): the four geometric outputs are computed normally;
  `apparent_magnitude` is `None`. This is a documented **coverage bound**
  identical in spirit to SP-4's fictitious/asteroid `nod_aps` bound —
  engine-covered geometry, gate-unreferenced magnitude — because SE ships no
  photometric model for those bodies.

## Gate & tooling

Follows the established SP-series reference pattern exactly:

- **Generator**: `tools/se-pheno-reference/` — links Swiss Ephemeris 2.10.03 via
  `libswisseph-sys`, runs the Moshier ephemeris (`SEFLG_MOSEPH`) so no Swiss
  Ephemeris `.se1` data files are bundled or read, and emits the corpus as CSV.
  Ships a `LICENSE-NOTES.md` matching its siblings (SE is a build-time reference
  dependency of the generator only; no SE source, binaries, or data enter the
  shipped product).
- **Corpus**: committed under the gate's data directory, checksum-pinned
  (fnv1a64) and row-count-pinned. ~180 rows: the ten majors × ~18 epochs across
  the 1900–2100 TDB window, deliberately sampling inner-planet inferior
  conjunctions (full phase-angle sweep) and a spread of Saturn ring-plane
  openings (to exercise the ring term at both edge-on and wide-open geometry).
- **Gate**: `validate-pheno` (CLI aliases `validate-pheno` / `pheno-gate` /
  `pheno`), wired into `run_all_numeric_gates`, `release-smoke`, and
  `release-gate`.
- **Ceilings**: per-output-class, **measured-then-pinned** (land provisional,
  then pin from measured residuals — the SP-4 method). Expected posture:
  - geometric outputs tight — phase angle / elongation sub-arcsecond-equivalent,
    phase fraction ~1e-6, apparent diameter arcsecond-class;
  - magnitude per-planet, millimag to a few hundredths, with **Saturn the
    widest** owing to the ring term.

  The final numeric ceilings are set from the first measured run and recorded in
  the plan status line at completion, as prior SP slices did.

## Testing

- **Unit tests** (`pheno.rs` / `magnitude.rs`): phase-angle/elongation triangle
  identities, `phase_fraction ∈ [0, 1]`, Sun/Moon special-case values, magnitude
  monotonicity in phase angle where expected, and never-NaN behavior under
  clamped degenerate geometry.
- **Integration invariants** over the routing chain (an `EventEngine` built on
  the packaged backend), analogous to SP-4's `nod_aps` integration test.
- **Fail-closed `validate-pheno` gate** as the SE-parity acceptance test.

## Docs & release posture

- Update `PLAN.md`, `plan/status/01-current-execution-frontier.md`,
  `plan/status/02-next-slice-candidates.md`, `README.md`, and `CHANGELOG.md`;
  declare SP-5 done and record the measured accuracy summary.
- **Compatibility profile: 0.7.10 → 0.7.11.**
- **API stability profile: unchanged at 0.2.2** (purely additive surface — new
  struct, new method, no breaking change).
- Remaining event-engine follow-ups after SP-5 (not scoped here): custom
  fictitious-body orbital elements, occultations, and central-path cartography
  for solar eclipses.

## Non-goals

- Apparent magnitude for asteroids (SE's H–G-system model) or fictitious bodies —
  explicitly out of scope; `None` with a documented coverage bound.
- Topocentric phase/diameter — geocentric apparent-of-date only, matching the
  engine default and SE's typical MOSEPH call.
- Any new `CelestialBody` variants or a separate crate (Approach B rejected).
- Rates/speeds of the pheno quantities (SE returns none).
