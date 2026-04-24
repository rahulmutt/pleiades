# Checklist 2 — Release Artifacts

Use this checklist when preparing a release-facing bundle.

## Required bundle contents

- [ ] Source revision identifier.
- [ ] Rust/toolchain version information.
- [ ] Workspace status summary from a clean checkout.
- [ ] Release compatibility profile.
- [ ] Compatibility-profile summary.
- [ ] Backend capability matrix.
- [ ] Backend capability matrix summary.
- [ ] API stability posture.
- [ ] API stability summary.
- [ ] Validation report and compact summary.
- [ ] Artifact validation/inspection report and summary for packaged data, if packaged data is claimed.
- [ ] Release notes and summary.
- [ ] Release checklist and summary.
- [ ] Manifest with canonical checksums.
- [ ] Manifest checksum sidecar.

## Required validation before publication

- [ ] `cargo fmt --all --check`.
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- [ ] `cargo test --workspace`.
- [ ] Compatibility-profile verification.
- [ ] Release-bundle verification.
- [ ] Pure-Rust/native-dependency audit.
- [ ] Artifact checksum and error-threshold validation for every shipped artifact.
- [ ] Benchmark/report generation using the release source revision.

## Claim audit

- [ ] Body coverage claims match backend metadata and validation reports.
- [ ] House-system claims match descriptor metadata, implementation status, and reference tests.
- [ ] Ayanamsa claims match descriptor metadata, implementation status, and reference tests.
- [ ] Compressed-data coverage claims match artifact metadata and validation summaries.
- [ ] Accuracy claims include measured tolerances and source/reference descriptions.
- [ ] Known gaps, aliases, and latitude/numerical constraints are listed.
- [ ] No profile claims full target-catalog coverage unless all target entries are implemented and validated.

## Reproducibility

- [ ] The bundle can be regenerated from documented commands.
- [ ] Generated compressed artifacts can be reproduced from documented public inputs and parameters.
- [ ] Reports include enough environment metadata for maintainers to compare reruns.
- [ ] Checksums verify after unpacking or copying the bundle.
