# Pleiades Specification

This document defines the high-level specification for **Pleiades**, a pure-Rust modular ephemeris platform intended for astrology software.

## Goals

Pleiades must:

- provide ephemeris functionality comparable in scope to Swiss Ephemeris for astrology-oriented use cases
- be implemented in **pure Rust**, with **no C/C++ dependencies**
- expose a modular backend abstraction so multiple data sources and algorithm families can coexist
- support astrology features including the Sun, Moon, planets, an extensible asteroid/body catalog, **the full house-system catalog and the full ayanamsa catalog required for Swiss-Ephemeris-class astrology workflows**, sidereal/tropical modes, and derived chart quantities
- define a compressed data representation optimized for the common historical/future window **1500-2500 CE**
- organize **every first-party sub-crate** under names of the form **`pleiades-*`**

## Specification Index

### Normative Documents

- [spec/vision-and-scope.md](spec/vision-and-scope.md) — product scope, goals, non-goals, user targets
- [spec/requirements.md](spec/requirements.md) — functional and non-functional requirements
- [spec/architecture.md](spec/architecture.md) — workspace layout, crate decomposition, dependency rules
- [spec/backend-trait.md](spec/backend-trait.md) — modular ephemeris backend trait and backend contract
- [spec/astrology-domain.md](spec/astrology-domain.md) — supported bodies, houses, ayanamsa, coordinate systems, derived values
- [spec/data-compression.md](spec/data-compression.md) — compressed ephemeris representation for 1500-2500 and distribution strategy
- [spec/backends.md](spec/backends.md) — backend implementations, source families, crate inventory
- [spec/api-and-ergonomics.md](spec/api-and-ergonomics.md) — Rust API shape, error handling, configuration model
- [spec/validation-and-testing.md](spec/validation-and-testing.md) — correctness, benchmarks, compatibility, release criteria

### Informative Document

- [spec/roadmap.md](spec/roadmap.md) — phased implementation plan

## Core Product Statement

Pleiades is a **workspace of pure-Rust crates** that separates:

1. **domain API** for astrology and astronomy consumers,
2. **backend implementations** based on public datasets and/or closed-form algorithms,
3. **compressed data products** for fast practical use across 1500-2500,
4. **validation tooling** to compare results against authoritative references.

## Normative Decisions

The following decisions are binding unless a sub-spec explicitly supersedes them:

1. **Language/runtime**: Rust only.
2. **FFI policy**: no required C/C++ libraries, wrappers, or build-time native toolchains.
3. **Crate naming**: every first-party crate, including tooling and data crates, must begin with `pleiades-`.
4. **Backend modularity**: every ephemeris source/algorithm implementation lives in its own crate.
5. **Separation of concerns**: astrology-domain calculations must not be hardwired to one backend.
6. **Compatibility catalog**: the project must define and eventually ship the full built-in house-system and ayanamsa catalogs needed for Swiss-Ephemeris-class astrology compatibility. For this spec set, that means the complete built-in compatibility surface Pleiades intends to expose for interoperability with Swiss-Ephemeris-style workflows, including documented aliases and operational constraints. Each release must publish a versioned compatibility profile describing the exact built-ins, aliases, and remaining gaps.
7. **Layering rule**: low-level backends provide raw astronomical results and capability metadata; domain-layer crates are responsible for astrology-specific transforms such as sidereal conversion, house placement, and chart assembly unless a backend explicitly documents equivalent native support.
8. **Data range optimization**: compressed packaged data is optimized for 1500-2500, while some live/computational backends may support broader ranges.
9. **Reproducibility**: packaged data artifacts must be versioned, documented, and regenerable from public inputs.

## Initial Crate Family

The expected initial workspace includes at least:

- `pleiades-core`
- `pleiades-types`
- `pleiades-backend`
- `pleiades-jpl`
- `pleiades-vsop87`
- `pleiades-elp`
- `pleiades-houses`
- `pleiades-ayanamsa`
- `pleiades-compression`
- `pleiades-data`
- `pleiades-cli`
- `pleiades-validate`

Additional backend crates may be added as needed, provided they follow the same naming and interface rules.

## Acceptance Summary

The project is considered aligned with this specification when it can:

- compute Sun, Moon, planetary, and selected asteroid positions through a backend-agnostic API
- compute the target compatibility house-system catalog and the target compatibility ayanamsa catalog
- switch among at least two backend families with no API break for consumers
- distribute a compressed ephemeris dataset covering 1500-2500
- build and test on supported targets using pure Rust dependencies only

## Conformance Terminology

To keep phased delivery compatible with the long-term scope, the spec uses the following terms consistently:

- **Target compatibility catalog**: the full end-state built-in catalog of house systems and ayanamsas required for Swiss-Ephemeris-class astrology interoperability.
- **Baseline compatibility milestone**: the minimum built-in subset required before broader catalog completion; this is a delivery milestone, not the final scope boundary.
- **Release compatibility profile**: a versioned manifest published for each release that enumerates the exact built-ins, aliases, constraints, and known gaps shipped in that release.

Interim releases may implement only the baseline milestone plus incremental additions, but the architecture and public APIs must remain open to the full target compatibility catalog without redesign.

## Bootstrap Compliance Checklist

Verdict: the derived specification set adheres to the bootstrap prompt. The required constraints are stated normatively and decomposed across the linked sub-specs as follows:

| Bootstrap requirement | Where it is specified |
| --- | --- |
| Modular astrology-oriented ephemeris platform similar in scope to Swiss Ephemeris | [spec/vision-and-scope.md](spec/vision-and-scope.md), [spec/requirements.md](spec/requirements.md), [spec/astrology-domain.md](spec/astrology-domain.md) |
| Pure Rust only, with no C/C++ dependencies | [spec/requirements.md](spec/requirements.md), [spec/architecture.md](spec/architecture.md) |
| Modular ephemeris backend trait with distinct backend crates for each implementation | [spec/backend-trait.md](spec/backend-trait.md), [spec/backends.md](spec/backends.md), [spec/architecture.md](spec/architecture.md) |
| Compressed representation optimized for common use in 1500-2500 | [spec/data-compression.md](spec/data-compression.md), [spec/backends.md](spec/backends.md) |
| All first-party sub-crates named `pleiades-*` | [spec/architecture.md](spec/architecture.md) |

The main design caution is that phased implementation must never be mistaken for a reduced end-state scope. The revised sub-specs therefore treat the **target compatibility catalog** as the full compatibility surface and require every release to publish a compatibility profile that makes current coverage and gaps explicit.

## Document Status

Status: Draft 4
Owner: Project maintainers
Last updated: 2026-04-23
