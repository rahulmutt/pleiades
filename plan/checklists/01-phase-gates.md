# Checklist 1 — Phase Gates

## Common gate for every phase

- [ ] The implemented behavior maps to one or more requirements in `SPEC.md` or `spec/*.md`.
- [ ] Public APIs include units, frames, time scales, normalization, and failure-mode documentation where relevant.
- [ ] Errors distinguish invalid input, unsupported features, out-of-range data, missing data/configuration, and numerical failures as applicable.
- [ ] Tests cover success paths, edge cases, and regression scenarios introduced by the phase.
- [ ] `cargo fmt --all --check` passes.
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes or any exception is narrowly justified.
- [ ] `cargo test --workspace` passes.
- [ ] New tools are declared in `mise.toml` unless they must live in `devenv.nix` with justification.
- [ ] Plan/status docs no longer list tasks completed by the change.

## Phase 1: Reference accuracy and request semantics

- [ ] Release-claimed bodies have source-backed or explicitly documented accuracy evidence.
- [ ] Pluto and other approximate/fallback paths are either validated within release thresholds or downgraded in release-profile claims.
- [ ] Reference/source coverage is sufficient for validation and artifact generation inputs.
- [ ] Delta T, UTC/UT1 conversion, apparentness, topocentric, native sidereal, and frame behavior is implemented or rejected explicitly through metadata and structured errors.
- [ ] Validation reports distinguish release-tolerance evidence from provenance-only or interpolation-transparency evidence.

## Phase 2: Production compressed artifacts

- [ ] Artifact headers/profiles record provenance, source versions, body coverage, channels, checksums, and generation parameters.
- [ ] Artifact generation is deterministic from documented public inputs.
- [ ] Decode/lookup tests include segment boundaries, interval interiors, unsupported bodies, unsupported outputs, and checksum failures.
- [ ] Artifact validation reports include measured errors inside published thresholds and benchmark data.
- [ ] Runtime packaged-data metadata no longer labels production claims as prototype.

## Phase 3: Compatibility evidence and catalog truthfulness

- [ ] Every release-profile catalog entry has descriptor metadata, aliases, implementation status, constraints, and tests.
- [ ] House systems document formulas, assumptions, latitude/numerical failure modes, and reference/golden scenarios.
- [ ] Ayanamsas document reference epochs, offsets/formulas, aliases, provenance, sidereal metadata, and custom-definition posture.
- [ ] Profile verification fails on unsupported, descriptor-only, approximate, or constrained entries advertised as fully implemented.

## Phase 4: Release hardening and publication

- [ ] Compatibility profile, backend matrix, validation report, benchmark report, artifact summary, release notes, release checklist, and release summary are generated from current code/data.
- [ ] Release bundle verification passes from a clean checkout.
- [ ] Native-dependency/pure-Rust audit passes.
- [ ] Public documentation and rustdoc examples cover main workflows and known limitations.
- [ ] Archived artifacts include source revision, profile identifiers, tool versions, checksums, validation parameters, and artifact-generation parameters.
