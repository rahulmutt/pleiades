# Astrology Domain Specification

Unless stated otherwise, the conformance terms defined in [`SPEC.md`](../SPEC.md) apply here.

## Supported Body Model

The domain layer must define a stable body taxonomy covering:

- luminaries: Sun, Moon
- planets: Mercury through Pluto
- lunar points: mean node, true node, and mean or true apogee/perigee where modeled
- baseline asteroids: Ceres, Pallas, Juno, Vesta
- extensible identifiers for additional numbered or named bodies

## Zodiac Modes

The system must support:

- tropical zodiac
- sidereal zodiac via ayanamsa selection

Sidereal conversion should normally be implemented in the domain layer rather than duplicated across backends.

## House Systems

The house module must model the target compatibility catalog in a way that is complete in scope and extensible in implementation.

The baseline compatibility milestone is the minimum built-in set required for early releases. It must not be treated as the final catalog, and the representation of house systems, aliases, and failure constraints must remain open to the full intended interoperability set without enum churn or breaking redesign.

The common interface must accept:

- instant
- geographic latitude and longitude
- obliquity and related astronomical quantities as needed
- selected house system

The common interface must return:

- 12 house cusp positions where the system defines cusps
- derived angles including ASC, MC, IC, and DSC where meaningful
- explicit error or status values for latitude-driven or numerical failure cases

## Baseline Compatibility Milestone

The initial milestone must include at minimum:

- Placidus
- Koch
- Porphyry
- Regiomontanus
- Campanus
- Equal
- Whole Sign
- Alcabitius
- Meridian and documented Axial variants
- Topocentric (Polich-Page)
- Morinus

Each implemented system must document its formula, assumptions, aliases, and failure modes.

## Ayanamsa Model

Ayanamsa support must include:

- a built-in catalog model that can grow to the full target compatibility catalog
- named built-in definitions
- epoch, offset, or formula metadata
- custom ayanamsa registration
- deterministic tropical-to-sidereal conversion
- compatibility-profile metadata for aliases and naming differences versus other astrology software

The baseline compatibility milestone must include Lahiri, Raman, Krishnamurti, Fagan/Bradley, True Chitra, and any documented near-equivalent variants exposed either as distinct built-ins or explicit aliases.

As with house systems, this baseline is a minimum shipping floor. The ayanamsa identifier model must remain open to the full target catalog plus user-defined definitions without requiring breaking redesign.

## Derived Quantities

The domain layer should support, either initially or in later phases:

- retrograde and stationary classification
- planetary speed bands
- aspects and orb-ready angular separations
- house placement and sign placement
- optional higher-level chart helpers, such as dignities, built above the core domain layer

## Time Scales

The system must clearly model at least:

- UTC input convenience
- Julian day or Julian ephemeris day style internal representations
- Delta T handling policy
- the distinction between UT-based and dynamical-time-sensitive calculations where needed

## Catalog Management

Built-in house-system and ayanamsa identifiers must support:

- stable programmatic identifiers
- human-readable display names
- explicit alias metadata for interoperability with external astrology software
- compatibility-profile annotations for constraints, equivalence claims, and known gaps

User-defined house or ayanamsa extensions may exist, but they must remain clearly distinguishable from project-defined built-ins in serialization and compatibility reporting.

## Numerical Rules

All angle values must define normalization rules, recommended precision, and wrap semantics. Public APIs must document whether longitudes are returned in `[0, 360)` or another canonical range.
