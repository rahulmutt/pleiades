# Requirements

## Functional Requirements

### FR-1 Body Support
The system must support computation of at least:

- Sun
- Moon
- Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto
- true and mean lunar node
- true and mean apogee/perigee where supported and documented
- selected major asteroids, with initial priority on Ceres, Pallas, Juno, and Vesta

### FR-2 Coordinate and Reference Support
The system must support:

- ecliptic longitude/latitude
- equatorial right ascension/declination
- distance and apparent angular speed where available
- tropical and sidereal modes
- geocentric calculations as baseline
- topocentric adjustments where supported by the chosen backend/model

### FR-3 House Systems
The system must provide a house computation module supporting a complete and extensible catalog of astrological house systems. The initial mandatory catalog must include at minimum:

- Placidus
- Koch
- Porphyry
- Regiomontanus
- Campanus
- Equal
- Whole Sign
- Alcabitius
- Meridian / Axial variants where documented
- Topocentric / Polich-Page
- Morinus

Where a system has latitude/pathology constraints, those constraints must be explicit in the API. The specification goal remains support for the broader house-system catalog beyond the initial mandatory list.

### FR-4 Ayanamsa
The system must support a pluggable ayanamsa catalog, including at minimum:

- Lahiri
- Raman
- Krishnamurti
- Fagan/Bradley
- True Chitra or equivalent documented variants
- custom user-defined ayanamsa formulas or offset tables

### FR-5 Backend Abstraction
The system must expose a common backend trait that:

- can compute one body at one instant
- can compute multiple bodies efficiently in batch
- exposes supported time range and capability metadata
- distinguishes between data-backed and purely algorithmic implementations
- reports uncertainty/accuracy class where known

### FR-6 Multiple Backend Implementations
The workspace must include separate crates for multiple backends, including examples of:

- a JPL-based data backend
- a formula-based planetary backend
- a lunar algorithm backend
- a compressed packaged-data backend optimized for 1500-2500

### FR-7 Compression and Distribution
The system must define a compressed ephemeris representation that:

- is optimized for repeated astrology application lookups
- covers 1500-2500 CE
- is versioned and regenerable from public source data
- supports efficient random access by body and date/time

### FR-8 API Stability
The public Rust API must present stable domain types for:

- time scales and Julian day values
- body identifiers
- observer location
- coordinate frames
- house systems
- ayanamsa definitions
- backend selection/configuration

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
New backend crates and new house/ayanamsa implementations must be addable without breaking the core traits.

### NFR-6 Portability
The workspace should target common Rust platforms including Linux, macOS, and Windows.
