# Requirements

Unless stated otherwise, the conformance terms defined in [`SPEC.md`](../SPEC.md) apply here, especially **target compatibility catalog**, **baseline compatibility milestone**, and **release compatibility profile**.

## Functional Requirements

### FR-1 Body Support
The system must support computation of at least:

- Sun
- Moon
- Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto
- true and mean lunar node
- mean and true apogee/perigee where supported and documented
- Ceres, Pallas, Juno, and Vesta in the baseline catalog
- an extensible identifier model for additional numbered or named asteroids and other supported points

### FR-2 Coordinate and Observer Support
The system must support:

- ecliptic longitude/latitude
- equatorial right ascension/declination
- distance and apparent angular speed where available
- tropical and sidereal modes
- geocentric calculations as the baseline
- topocentric adjustments where supported by the selected backend or domain model

### FR-3 Time and Reference Handling
The public API must model at least:

- UTC input convenience
- Julian day style internal time representation
- the distinction between UT-based and dynamical-time-sensitive calculations where needed
- a documented Delta T policy

### FR-4 House Systems
The system must provide a house computation module that is open to the full target compatibility catalog.

The baseline compatibility milestone must include at minimum:

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

If a house system has latitude or numerical failure constraints, those constraints must be explicit in both the API and the release compatibility profile.

### FR-5 Ayanamsa Support
The system must provide an extensible ayanamsa model that supports both built-in and user-defined variants.

The baseline compatibility milestone must include at minimum:

- Lahiri
- Raman
- Krishnamurti
- Fagan/Bradley
- True Chitra and documented near-equivalent variants exposed as distinct built-ins or explicit aliases
- custom user-defined ayanamsa formulas or offset tables

### FR-6 Release Compatibility Profiles
Each release must publish a versioned compatibility profile that states:

- the current target compatibility catalog for that release line
- the baseline compatibility milestone guaranteed by the project
- the exact house systems and ayanamsas shipped in the release
- aliases or naming differences versus other astrology software
- known gaps, constraints, and latitude-specific failure modes relevant to interoperability

A release must not claim full compatibility-catalog coverage unless the shipped built-ins match the published target catalog for that release line.

### FR-7 Backend Abstraction
The system must expose a common backend trait that:

- can compute one body at one instant
- can compute multiple bodies efficiently in batch
- exposes supported time range and capability metadata
- distinguishes between data-backed and purely algorithmic implementations
- reports uncertainty or accuracy class where known
- centers on raw astronomical outputs, with sidereal conversion remaining a domain-layer operation unless a backend explicitly documents equivalent native support

### FR-8 Multiple Backend Implementations
The workspace must include separate first-party crates for multiple backends, including examples of:

- a JPL-based data backend
- a formula-based planetary backend
- a lunar algorithm backend
- a compressed packaged-data backend optimized for 1500-2500 CE

### FR-9 Compression and Distribution
The system must define a compressed ephemeris representation that:

- is optimized for repeated astrology lookups
- covers 1500-2500 CE
- is versioned and reproducible from public source data
- supports efficient random access by body and date/time

### FR-10 Stable Domain API Surface
The public Rust API must present stable domain types for:

- time scales and Julian day values
- body identifiers
- observer location
- coordinate frames
- house systems
- ayanamsa definitions
- backend selection and configuration
- release compatibility profile metadata

## Non-Functional Requirements

### NFR-1 Purity
The project must compile and run without required C/C++ dependencies. CI and release validation must reject first-party crates that introduce mandatory native build or runtime requirements.

### NFR-2 Performance
Common chart computations should be optimized for low-latency use in interactive astrology software.

### NFR-3 Accuracy Documentation
Each backend must publish documented expected error bounds or empirical validation results.

### NFR-4 Reproducibility
Generated data products must be reproducible from documented inputs and deterministic build steps.

### NFR-5 Extensibility
New backend crates and new house or ayanamsa implementations must be addable without breaking the core traits or public API model.

### NFR-6 Portability
The workspace should target common Rust platforms including Linux, macOS, and Windows.
