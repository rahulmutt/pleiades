# Target Compatibility Catalog

Unless stated otherwise, the conformance terms defined in [`SPEC.md`](../SPEC.md) apply here.

This document enumerates the **target compatibility catalog**: the binding end-state set of
house systems and ayanamsas Pleiades commits to for Swiss-Ephemeris-class interoperability.
It exists so that "all house systems" and "all ayanamsas" from `prompts/bootstrap.md` resolve
to a concrete, bounded list rather than an open-ended promise.

Phased delivery is allowed. The **baseline compatibility milestone** entries (marked
**[baseline]**) are the minimum shipping floor for early releases; every other entry is part
of the committed end-state and must be reachable without public API or enum redesign. A
release must not narrow this catalog, and each release's
**release compatibility profile** must report which of these entries actually ship in that
version, with aliases, constraints, and known gaps.

## House Systems

The target set is the published Swiss Ephemeris house-system set, identified by its
conventional `swe_houses` letter code. Each implemented system must document its formula,
assumptions, aliases, and latitude/numerical failure modes per
[`astrology-domain.md`](astrology-domain.md).

| Code | House system | Status |
| --- | --- | --- |
| P | Placidus | **[baseline]** |
| K | Koch | **[baseline]** |
| O | Porphyry | **[baseline]** |
| R | Regiomontanus | **[baseline]** |
| C | Campanus | **[baseline]** |
| A / E | Equal (from Ascendant) | **[baseline]** |
| W | Whole Sign | **[baseline]** |
| B | Alcabitius | **[baseline]** |
| X | Meridian / Axial Rotation | **[baseline]** |
| T | Polich-Page (Topocentric) | **[baseline]** |
| M | Morinus | **[baseline]** |
| D | Equal (from MC) | shipped |
| N | Whole Sign / Equal from 0° Aries | shipped |
| V | Vehlow Equal | shipped |
| U | Krusinski-Pisa-Goelzer | shipped |
| Y | APC houses | shipped |
| S | Sripati | shipped |
| F | Carter poli-equatorial | shipped |
| H | Horizontal / Azimuthal | shipped |
| G | Gauquelin sectors | shipped |
| L | Pullen SD (sinusoidal delta) | shipped |
| Q | Pullen SR (sinusoidal ratio) | shipped |
| I | Sunshine (Makransky) | shipped |

The eleven **[baseline]** entries match the baseline floor in
[`requirements.md`](requirements.md) FR-4 and [`astrology-domain.md`](astrology-domain.md).

**Status legend:** **[baseline]** = baseline shipping floor (implemented);
*shipped* = implemented beyond the baseline; *target* = committed end-state not
yet implemented. As of the current release line, every entry in the table above
is implemented — all twelve non-baseline systems are *shipped*, so no house-system
entry remains in *target* state. The first-party crates ship 25 built-in house
systems in total (24 numerically gated by `validate-houses`); the extra built-in,
Albategnius / Savard-A, sits beyond this 23-code SE set and is not yet
corpus-gated. Per-release shipping status remains authoritative in the release
compatibility profile.

## Ayanamsas

The target set is the full Swiss Ephemeris sidereal-mode catalog, referenced by the
`SE_SIDM_*` constants as the authoritative enumeration. New SE sidereal modes added upstream
are considered in-scope for the target catalog. The catalog model must also accept
**user-defined** ayanamsas (offset or formula) that remain clearly distinguishable from
project-defined built-ins, per [`astrology-domain.md`](astrology-domain.md).

The **[baseline]** entries are Lahiri, Raman, Krishnamurti, Fagan/Bradley, and True Chitra
(with documented near-equivalent variants exposed as distinct built-ins or explicit aliases).

The target set includes at least the following SE sidereal modes:

- Fagan/Bradley **[baseline]**
- Lahiri **[baseline]**, Lahiri 1940, Lahiri VP285, Lahiri ICRC
- De Luce
- Raman **[baseline]**
- Usha/Shashi
- Krishnamurti **[baseline]**, Krishnamurti-Senthilathiban
- Djwhal Khul
- Yukteshwar
- J.N. Bhasin
- Babylonian (Kugler 1/2/3, Huber, Eta Piscium, Aldebaran=15 Tau, Britton)
- Hipparchos
- Sassanian
- Galactic Center=0 Sag, Galactic Center (Gil Brand), Galactic Center (Cochrane),
  Galactic Center/Mula (Wilhelm)
- Galactic Equator (IAU1958), Galactic Equator, Galactic Equator mid-Mula,
  Galactic Equator (Fiorenza)
- J2000, J1900, B1950
- Suryasiddhanta, Suryasiddhanta (mean Sun), Aryabhata, Aryabhata (mean Sun), Aryabhata 522
- SS Revati, SS Citra, True Citra **[baseline]**, True Revati, True Pushya (PVRN Rao),
  True Mula (Chandra Hari)
- Skydram (Mardyks)
- Dhruva (Wilhelm)
- "Vedic"/Sheoran
- Vettius Valens

This list reflects the SE catalog at time of writing and is maintained against the upstream
`SE_SIDM_*` enumeration; the binding commitment is "the SE sidereal-mode set," not this
transcription. Where Pleiades and another tool disagree on a name, the difference must be
recorded as an alias in the release compatibility profile.

As of the current release line, the first-party crates ship 59 built-in sidereal
modes, of which 48 are numerically gated by `validate-ayanamsa`; the remaining 11
are descriptor-only (6 without a computation path). This is well beyond the five
**[baseline]** modes. Per-release shipping and gating status is authoritative in
the release compatibility profile.
