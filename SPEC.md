# Pleiades Specification

This document defines the high-level specification for **Pleiades**, a pure-Rust modular ephemeris platform intended for astrology software.

## Goals

Pleiades must:

- provide ephemeris functionality comparable in scope to Swiss Ephemeris for astrology-oriented use cases
- be implemented in **pure Rust**, with **no C/C++ dependencies**
- expose a modular backend abstraction so multiple data sources and algorithm families can coexist
- support astrology features including the Sun, Moon, planets, selected asteroids, **the full project house-system catalog**, ayanamsas, sidereal/tropical modes, and derived chart quantities
- define a compressed data representation optimized for the common historical/future window **1500-2500 CE**
- organize **every first-party sub-crate** under names of the form **`pleiades-*`**

## Specification Index

- [spec/vision-and-scope.md](spec/vision-and-scope.md) — product scope, goals, non-goals, user targets
- [spec/requirements.md](spec/requirements.md) — functional and non-functional requirements
- [spec/architecture.md](spec/architecture.md) — workspace layout, crate decomposition, dependency rules
- [spec/backend-trait.md](spec/backend-trait.md) — modular ephemeris backend trait and backend contract
- [spec/astrology-domain.md](spec/astrology-domain.md) — supported bodies, houses, ayanamsa, coordinate systems, derived values
- [spec/data-compression.md](spec/data-compression.md) — compressed ephemeris representation for 1500-2500 and distribution strategy
- [spec/backends.md](spec/backends.md) — backend implementations, source families, crate inventory
- [spec/api-and-ergonomics.md](spec/api-and-ergonomics.md) — Rust API shape, error handling, configuration model
- [spec/validation-and-testing.md](spec/validation-and-testing.md) — correctness, benchmarks, compatibility, release criteria
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
3. **Crate naming**: every first-party crate must begin with `pleiades-`.
4. **Backend modularity**: every ephemeris source/algorithm implementation lives in its own crate.
5. **Separation of concerns**: astrology-domain calculations must not be hardwired to one backend.
6. **Data range optimization**: compressed packaged data is optimized for 1500-2500, while some live/computational backends may support broader ranges.
7. **Reproducibility**: packaged data artifacts must be versioned, documented, and regenerable from public inputs.

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
- compute the project’s full astrological house-system catalog and ayanamsa conversions
- switch among at least two backend families with no API break for consumers
- distribute a compressed ephemeris dataset covering 1500-2500
- build and test on supported targets using pure Rust dependencies only

## Bootstrap Compliance Checklist

This spec set satisfies the bootstrap prompt by making the following requirements normative:

| Bootstrap requirement | Where it is specified |
| --- | --- |
| Modular astrology-oriented ephemeris platform similar in scope to Swiss Ephemeris | [spec/vision-and-scope.md](spec/vision-and-scope.md), [spec/requirements.md](spec/requirements.md), [spec/astrology-domain.md](spec/astrology-domain.md) |
| Pure Rust only, with no C/C++ dependencies | [spec/requirements.md](spec/requirements.md), [spec/architecture.md](spec/architecture.md) |
| Modular ephemeris backend trait with distinct backend crates for each implementation | [spec/backend-trait.md](spec/backend-trait.md), [spec/backends.md](spec/backends.md), [spec/architecture.md](spec/architecture.md) |
| Compressed representation optimized for common use in 1500-2500 | [spec/data-compression.md](spec/data-compression.md), [spec/backends.md](spec/backends.md) |
| All first-party sub-crates named `pleiades-*` | [spec/architecture.md](spec/architecture.md) |

The main gap in the earlier draft was wording around house systems and ayanamsas: some sections implied only an initial subset was required. The revised sub-specs now make the **end-state requirement** explicit while still allowing phased implementation.

## Document Status

Status: Draft 2
Owner: Project maintainers
Last updated: 2026-04-22
