# Checklist 1 — Phase Gates

Use this checklist before marking any remaining phase complete.

## Common gate for every phase

- [ ] The implemented behavior maps to one or more requirements in `SPEC.md` or `spec/*.md`.
- [ ] Public APIs include units, frames, time scales, normalization, and failure-mode documentation where relevant.
- [ ] Errors distinguish invalid input, unsupported features, out-of-range data, missing data/configuration, and numerical failures as applicable.
- [ ] Tests cover success paths, edge cases, and regression scenarios introduced by the phase.
- [ ] `cargo fmt --all --check` passes.
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes or any exception is narrowly justified.
- [ ] `cargo test --workspace` passes.
- [ ] New tools are declared in `mise.toml` unless they must live in `devenv.nix` with justification.
- [ ] The plan/status docs no longer list tasks completed by the change.

## Phase 1: Production ephemeris accuracy

- [ ] Backend metadata identifies real source material and accuracy expectations.
- [ ] VSOP87, lunar, and JPL/reference implementations have golden or reference-backed tests for claimed bodies.
- [ ] Time-scale, Delta T, apparent/mean, frame, and topocentric semantics are implemented or rejected explicitly.
- [ ] Validation reports include measured errors and tolerances for claimed support.

## Phase 2: Reproducible compressed artifacts

- [ ] Artifact headers/profiles record provenance, source versions, body coverage, channels, checksums, and generated parameters.
- [ ] Artifact generation is deterministic from documented public inputs.
- [ ] Decode/lookup tests include segment boundaries, unsupported bodies, and checksum failures.
- [ ] Artifact validation reports include measured errors and benchmark data.

## Phase 3: Compatibility catalog completion

- [ ] Every release-profile catalog entry has descriptor metadata, aliases, implementation status, and tests.
- [ ] House systems document formulas, assumptions, and latitude/numerical failure modes.
- [ ] Ayanamsas document reference epochs, offsets/formulas, aliases, and provenance.
- [ ] Profile verification fails on unsupported entries advertised as implemented.

## Phase 4: Release stabilization and hardening

- [ ] Compatibility profile, backend matrix, validation report, benchmark report, artifact summary, release notes, and release checklist are generated from current code/data.
- [ ] Release bundle verification passes from a clean checkout.
- [ ] Native-dependency/pure-Rust audit passes.
- [ ] Public documentation and rustdoc examples cover main workflows.
- [ ] Archived artifacts include source revision, profile identifiers, tool versions, checksums, and validation parameters.
