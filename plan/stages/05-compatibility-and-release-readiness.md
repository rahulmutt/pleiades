# Phase 5 — Compatibility and Release Gates

## Goal

Finish compatibility evidence and release gates so a production release cannot
ship stale artifacts, native-dependency drift, or overbroad claims.

## Current baseline

- Baseline house and ayanamsa support is present.
- Broader descriptor catalogs, aliases, compatibility-profile summaries,
  known-gap reporting, house validation, ayanamsa catalog validation, and release
  bundle rehearsal surfaces exist.
- Release-bundle verification already rechecks many generated sidecars against
  current renderers.

## Remaining implementation work

- Keep compatibility profiles exact about shipped built-ins, aliases,
  constraints, descriptor-only entries, custom-definition territory, and known
  gaps.
- Add release gates for any remaining generated artifacts whose stale output,
  missing input, unsupported-mode claim drift, or threshold failure is not yet
  checked.
- Keep pure-Rust/native-dependency audits and workspace tool-version provenance
  in the release process; the workspace audit now also checks the pinned
  `mise.toml` rust toolchain against the workspace `rust-version` and requires
  the `rustfmt` and `clippy` components.
- Document production-ready public workflows once claims and request modes are
  settled.

## Exit criteria

- A clean checkout can build, validate, benchmark, bundle, and verify a release.
- Release gates fail on stale generated outputs, native-dependency drift,
  artifact threshold failures, unsupported-mode claim drift, missing evidence, or
  compatibility-profile overclaims.
