# Pleiades Specification

This document defines the top-level specification for **Pleiades**, a pure-Rust modular ephemeris workspace for astrology software.

## Bootstrap Summary

The specification must satisfy the bootstrap prompt in `prompts/bootstrap.md`:

1. modular ephemeris software similar in scope to Swiss Ephemeris for astrology-oriented workflows
2. pure Rust only, with no required C/C++ dependencies
3. a modular backend trait with separate backend crates for different public data sources and algorithm families
4. a compressed representation optimized for common use in **1500-2500 CE**
5. first-party crates named `pleiades-*`

## Goals

Pleiades must:

- provide astrology-oriented ephemeris functionality comparable in scope to Swiss-Ephemeris-class workflows
- remain fully implemented in **pure Rust** with **no required C/C++ dependencies**
- expose a modular backend abstraction so multiple data sources and algorithm families can coexist
- support the bodies, house systems, ayanamsas, zodiac modes, and derived values needed by real astrology software
- define a compressed data representation optimized for **1500-2500 CE**
- organize every first-party crate under the `pleiades-*` prefix

## Specification Index

### Normative Documents

- [spec/vision-and-scope.md](spec/vision-and-scope.md) — product scope, users, goals, and non-goals
- [spec/requirements.md](spec/requirements.md) — functional and non-functional requirements
- [spec/architecture.md](spec/architecture.md) — workspace layout, crate boundaries, and dependency rules
- [spec/backend-trait.md](spec/backend-trait.md) — backend contract and capability model
- [spec/astrology-domain.md](spec/astrology-domain.md) — bodies, houses, ayanamsas, coordinate handling, and derived values
- [spec/data-compression.md](spec/data-compression.md) — compressed artifact format and generation pipeline for 1500-2500 CE
- [spec/backends.md](spec/backends.md) — backend families and required first-party backend crates
- [spec/api-and-ergonomics.md](spec/api-and-ergonomics.md) — public Rust API shape, configuration, and errors
- [spec/validation-and-testing.md](spec/validation-and-testing.md) — correctness, benchmarking, and release gates

### Informative Document

- [spec/roadmap.md](spec/roadmap.md) — phased implementation plan

## Core Product Statement

Pleiades is a **workspace of pure-Rust crates** that separates:

1. shared domain types,
2. backend contracts,
3. source-specific backend implementations,
4. astrology-domain computations,
5. high-level façade APIs,
6. validation and packaging tooling.

## Normative Decisions

These decisions are binding unless a sub-spec explicitly refines them:

1. **Language/runtime**: Rust only.
2. **FFI policy**: no required C/C++ libraries, wrappers, or native toolchains.
3. **Crate naming**: every first-party crate must begin with `pleiades-`.
4. **Backend modularity**: each backend implementation lives in its own crate.
5. **Separation of concerns**: astrology-domain logic must not be hardwired to one backend.
6. **Compatibility scope**: Pleiades must define an end-state built-in catalog of house systems and ayanamsas sufficient for Swiss-Ephemeris-class interoperability.
7. **Layering rule**: backends provide astronomical results and capability metadata; house, ayanamsa, and chart logic live in domain-layer crates unless a backend explicitly exposes equivalent native support.
8. **Data-range optimization**: packaged compressed data is optimized for 1500-2500 CE, while some live or algorithmic backends may support broader ranges.
9. **Reproducibility**: generated artifacts must be versioned, documented, and reproducible from public inputs.

## Conformance Terms

The spec uses these terms consistently:

- **Target compatibility catalog**: the full end-state built-in house-system and ayanamsa catalog Pleiades intends to ship for Swiss-Ephemeris-class interoperability.
- **Baseline compatibility milestone**: the minimum built-in subset required before broader catalog completion.
- **Release compatibility profile**: a versioned manifest published with each release that lists the exact built-ins, aliases, constraints, and known gaps shipped in that release.

Phased delivery is allowed, but interim releases must not narrow the end-state scope. APIs and crate boundaries must remain open to the full target compatibility catalog without redesign.

## Initial Crate Family

The initial workspace is expected to include at least:

- `pleiades-types`
- `pleiades-backend`
- `pleiades-core`
- `pleiades-houses`
- `pleiades-ayanamsa`
- `pleiades-compression`
- `pleiades-jpl`
- `pleiades-vsop87`
- `pleiades-elp`
- `pleiades-data`
- `pleiades-cli`
- `pleiades-validate`

Additional backend or tooling crates may be added if they follow the same naming and layering rules.

## Acceptance Summary

The project is aligned with this specification when it can:

- compute Sun, Moon, planetary, and baseline asteroid positions through a backend-agnostic API
- compute the published house-system and ayanamsa catalogs for the release compatibility profile
- switch among at least two backend families without changing the consumer-facing query model
- distribute a compressed ephemeris dataset covering 1500-2500 CE
- build and test on supported targets using pure Rust dependencies only

## Bootstrap Verification

The derived specification set satisfies the bootstrap prompt as follows:

| Bootstrap requirement | Where it is specified |
| --- | --- |
| Modular astrology-oriented ephemeris platform comparable in scope to Swiss Ephemeris | [spec/vision-and-scope.md](spec/vision-and-scope.md), [spec/requirements.md](spec/requirements.md), [spec/astrology-domain.md](spec/astrology-domain.md) |
| Pure Rust only, with no required C/C++ dependencies | [spec/requirements.md](spec/requirements.md), [spec/architecture.md](spec/architecture.md), [spec/validation-and-testing.md](spec/validation-and-testing.md) |
| Modular backend trait with separate backend crates | [spec/backend-trait.md](spec/backend-trait.md), [spec/backends.md](spec/backends.md), [spec/architecture.md](spec/architecture.md) |
| Compressed representation optimized for common use in 1500-2500 CE | [spec/data-compression.md](spec/data-compression.md), [spec/backends.md](spec/backends.md), [spec/architecture.md](spec/architecture.md) |
| All first-party crates named `pleiades-*` | [spec/architecture.md](spec/architecture.md) |

The current spec set adheres to the bootstrap requirements. The main refinements in this revision are:

- making the end-state catalog policy explicit so baseline milestones are not mistaken for final scope
- tightening crate-layer rules so domain crates remain backend-agnostic
- separating artifact/data responsibilities from astrology-domain responsibilities to reduce architectural ambiguity
- clarifying alias and compatibility-profile expectations for house systems and ayanamsas

## Document Status

Status: Draft 6
Owner: Project maintainers
Last updated: 2026-04-23
