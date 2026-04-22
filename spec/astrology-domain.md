# Astrology Domain Specification

## Supported Body Model

The domain layer must define a stable body taxonomy covering:

- luminaries: Sun, Moon
- planets: Mercury through Pluto
- lunar points: mean node, true node, mean apogee, osculating/true apogee where modeled
- baseline asteroids: Ceres, Pallas, Juno, Vesta
- extensible identifiers for additional numbered/named bodies

## Zodiac Modes

The system must support:

- tropical zodiac
- sidereal zodiac via ayanamsa selection

Sidereal conversion must be a domain-layer transformation rather than a backend-specific special case whenever practical.

## House Systems

The house module must model a complete and extensible catalog of astrological house systems. Development may be phased, but stable project conformance requires that the full target catalog be implemented.

The house module must provide a common interface that accepts:

- instant
- geographic latitude/longitude
- obliquity and related astronomical quantities as needed
- selected house system

The module must return:

- 12 house cusp positions where the system defines cusps
- derived angles including ASC, MC, IC, DSC where meaningful
- explicit error/status for systems that fail at extreme latitudes or special cases

## Initial Implementation Milestone

The target end state is support for the full house-system catalog targeted by astrology software. The first implementation milestone must include at minimum:

- Placidus
- Koch
- Porphyry
- Regiomontanus
- Campanus
- Equal
- Whole Sign
- Alcabitius
- Topocentric
- Morinus
- Meridian when represented as a distinct system
- Vehlow Equal may be included as an extension

Each implemented system must be documented for formula, assumptions, and failure modes. The API must remain open to the rest of the targeted house-system ecosystem without redesign or breakage.

## Ayanamsa Model

Ayanamsa support must include a complete extensible catalog model. Development may be phased, but stable project conformance requires the full target ayanamsa catalog.

Ayanamsa support must include:

- a complete extensible catalog model for built-in definitions
- named built-in definitions
- epoch/offset or formula metadata
- custom ayanamsa registration
- deterministic conversion from tropical longitude to sidereal longitude

## Derived Quantities

The domain layer should support, either initially or in later phases:

- retrograde/stationary classification
- planetary speed bands
- aspects and orb-ready angular separations
- house placement and sign placement
- dignities and interpretive helpers as optional higher layers

## Time Scales

The system must clearly model relevant time concepts, including:

- UTC input convenience
- Julian day / Julian ephemeris day style internal representation
- Delta T handling policy
- distinction between UT-based and dynamical-time-sensitive calculations where needed

## Numerical Rules

All angle values must define normalization rules, recommended precision, and wrap semantics. API output must document whether longitudes are returned in `[0, 360)` or another canonical range.
