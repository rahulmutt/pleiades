# Vision and Scope

## Vision

Pleiades is a pure-Rust ephemeris platform for astrology software developers who need reliable astronomical positions and astrology-specific derived calculations without depending on Swiss Ephemeris or any native C/C++ toolchain.

## Primary Users

- astrology desktop/mobile/web application developers
- CLI and batch-processing tool authors
- chart calculation service developers
- researchers comparing astrological techniques across large date ranges

## Product Objectives

1. Deliver accurate positions for major bodies used in astrology.
2. Support standard Western and sidereal astrological workflows.
3. Allow interchangeable ephemeris backends with different tradeoffs.
4. Provide a practical precompressed data format for fast offline use.
5. Remain auditable, reproducible, and fully Rust-native.

## In Scope

- geocentric astrology-oriented ephemeris calculations
- Sun, Moon, planets, nodes, selected asteroids, and derived points where algorithmically justified
- house systems used in mainstream and advanced astrology software
- ayanamsa support and tropical/sidereal conversions
- backend abstraction for JPL-style data-backed and formula-based approaches
- compressed ephemeris products for the 1500-2500 range
- validation against public references and accepted astronomical standards

## Out of Scope

- full observatory-grade scientific mission analysis
- dependency on Swiss Ephemeris data files or native libraries
- mandatory support for every small body in existence
- GUI applications as part of the core workspace
- undocumented black-box approximations with unknown provenance

## Design Principles

- **Pure Rust first**: all required runtime/build dependencies must be Rust.
- **Modular by default**: bodies, house systems, ayanamsas, and data backends are separable.
- **Transparent accuracy**: every algorithm and dataset must document provenance and expected error.
- **Practical astrology fit**: prioritize workflows actual astrology applications require.
- **Offline capable**: compressed datasets must support local use without network access.
