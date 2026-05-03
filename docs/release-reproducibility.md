# Release reproducibility

This repository keeps the release workflow reproducible with repository-managed tooling and a small set of checked-in commands.

## Standard local workflow

Install the pinned toolchain and run the normal quality gates:

```bash
mise install
mise run fmt
mise run lint
mise run test
```

These are the same commands expected by the workspace CI and by the stage-gate checklists.

The CI gate also runs a workspace-native dependency audit that checks manifests, lockfile entries, and crate-root `build.rs` files:

```bash
mise run audit
```

## Release smoke workflow

The repository also ships a release-style smoke check that exercises the validation bundle path end to end:

```bash
mise run release-smoke
```

That task first runs the workspace audit, validates the bundled compressed artifact, then generates a temporary release bundle and verifies the staged manifest checksums and manifest checksum sidecar using `pleiades-validate`.

## Manual bundle generation

To inspect the release artifacts directly, generate a bundle in a directory of your choice:

```bash
cargo run -q -p pleiades-validate -- bundle-release --out /tmp/pleiades-release
```

The validation tool can also render the full compatibility profile, the compact profile summary, verify the release compatibility profile against the canonical catalogs, render the release notes, release-notes summary, release checklist, release-checklist summary, release summary, production-generation-boundary summary, frame-policy summary, observer-policy summary, apparentness-policy summary, time-scale-policy summary, delta-t-policy summary, request-surface-summary, source-documentation-summary, source-documentation-health-summary / source-documentation-health, request-policy summary, interpolation-posture summary, packaged-lookup-epoch-policy summary, API stability summary, backend matrix summary, artifact summary, validate-artifact, workspace-audit, and validation summary directly when you only need the individual maintainer-facing artifacts, and the user-facing CLI mirrors the same maintainer-facing renderers, including the workspace audit and the full packaged-artifact inspection command. The compact compatibility-profile-summary view now also cross-references `verify-release-bundle` and `release-summary`, the release notes artifact now also names `release-summary`, the release-notes summary now also names `Packaged-artifact summary`, `Artifact summary`, and `Artifact validation` alongside `artifact-summary / artifact-posture-summary`, the release summary now also names `backend-matrix-summary` and `validation-report-summary`, the frame-policy summary exposes the shared request-frame posture directly, the request-policy summary exposes the shared request-semantics posture directly, the backend-matrix summary now also points to `validation-report-summary` and `verify-release-bundle`, the API stability summary now also points to `verify-release-bundle`, the validation-report summary now also points to `verify-release-bundle` and `release-notes-summary`, the artifact summary now also points to `release-checklist-summary`, `verify-release-bundle`, and `workspace-audit`, and the release-checklist summary now also points directly to `validation-report-summary`, `artifact-summary`, and `api-stability-summary` in addition to the compact release-summary view and the staged-bundle verifier, so maintainers can jump between the compact release surfaces without changing the underlying compatibility profile:

```bash
cargo run -q -p pleiades-validate -- compatibility-profile
cargo run -q -p pleiades-validate -- compatibility-profile-summary
cargo run -q -p pleiades-validate -- verify-compatibility-profile
cargo run -q -p pleiades-validate -- release-notes
cargo run -q -p pleiades-validate -- release-notes-summary
cargo run -q -p pleiades-validate -- release-checklist
cargo run -q -p pleiades-validate -- release-checklist-summary
cargo run -q -p pleiades-validate -- checklist-summary
cargo run -q -p pleiades-validate -- release-summary
cargo run -q -p pleiades-validate -- production-generation-boundary-summary
cargo run -q -p pleiades-validate -- observer-policy-summary
cargo run -q -p pleiades-validate -- apparentness-policy-summary
cargo run -q -p pleiades-validate -- request-policy-summary
cargo run -q -p pleiades-validate -- interpolation-posture-summary
cargo run -q -p pleiades-validate -- packaged-lookup-epoch-policy-summary
cargo run -q -p pleiades-cli -- release-notes
cargo run -q -p pleiades-cli -- release-notes-summary
cargo run -q -p pleiades-cli -- release-checklist
cargo run -q -p pleiades-cli -- release-checklist-summary
cargo run -q -p pleiades-cli -- checklist-summary
cargo run -q -p pleiades-cli -- release-summary
cargo run -q -p pleiades-cli -- request-policy-summary
cargo run -q -p pleiades-cli -- validate-artifact
cargo run -q -p pleiades-cli -- verify-compatibility-profile
cargo run -q -p pleiades-cli -- verify-release-bundle --out /tmp/pleiades-release
cargo run -q -p pleiades-cli -- report --rounds 100
cargo run -q -p pleiades-cli -- generate-report --rounds 100
cargo run -q -p pleiades-cli -- validation-report-summary
cargo run -q -p pleiades-cli -- validation-summary
cargo run -q -p pleiades-validate -- backend-matrix-summary
cargo run -q -p pleiades-validate -- matrix-summary
cargo run -q -p pleiades-validate -- api-stability-summary
cargo run -q -p pleiades-validate -- artifact-summary
cargo run -q -p pleiades-validate -- artifact-posture-summary
cargo run -q -p pleiades-validate -- validation-report-summary
cargo run -q -p pleiades-validate -- validation-summary
```

The bundle currently writes these text artifacts:

- `compatibility-profile.txt`
- `compatibility-profile-summary.txt`
- `release-notes.txt`
- `release-notes-summary.txt`
- `release-summary.txt`
- `release-checklist.txt`
- `release-checklist-summary.txt`
- `backend-matrix.txt`
- `backend-matrix-summary.txt`
- `api-stability.txt`
- `api-stability-summary.txt`
- `artifact-summary.txt`
- `release-profile-identifiers.txt`
- `benchmark-report.txt`
- `validation-report-summary.txt`
- `validation-report.txt`
- `bundle-manifest.txt` (includes the recorded source revision, workspace status, Rust compiler version, profile/API identifiers, and validation-round count)
- `bundle-manifest.checksum.txt` (records the checksum used to verify the staged manifest itself; the verifier expects the canonical single-line `0x...` format with no stray whitespace)

The generated `release-checklist.txt` now also embeds the canonical `bundle-release` and `verify-release-bundle` commands plus a pointer back to this guide, while `release-checklist-summary.txt` provides a compact audit view for quick release review, so the bundle stays self-describing for maintainers.

The VSOP87 source-backed coefficient blobs are also reproducible from the vendored public source text via the maintainer helper binary:

```bash
cargo run -q -p pleiades-vsop87 --bin regenerate-vsop87b-tables -- --out /tmp/pleiades-vsop87b
cargo run -q -p pleiades-vsop87 --bin regenerate-vsop87b-tables -- --check
```

That command rewrites the checked-in `VSOP87B.*.bin` artifacts from the matching vendored `VSOP87B.*` source files. The companion `--check` mode regenerates the blobs in memory and compares them against the committed artifacts without touching the working tree, and the crate tests assert the regenerated bytes match the committed blobs.

Verify the staged bundle with:

```bash
cargo run -q -p pleiades-validate -- verify-release-bundle --out /tmp/pleiades-release
```

The verifier expects exactly the staged bundle files listed above, so stray files or missing entries will cause verification to fail. Each expected path must also be a regular file; symlinks and other non-regular entries are rejected so staged bundles cannot smuggle in external contents.

## What the bundle is for

The release bundle makes the current release posture easy to reproduce and audit:

- the compatibility profile captures shipped house systems, ayanamsas, aliases, validation reference points, and compatibility caveats, and `verify-compatibility-profile` provides a quick catalog-alignment audit for the same release surface,
- the compatibility profile summary gives a compact count-based view of the same release posture, including validation reference points and ayanamsa sidereal-metadata coverage,
- the release notes file summarizes release-specific coverage, validation reference points, known limitations, and the current API stability / deprecation-policy snapshot, the release-notes summary gives a compact release-notes view, the release summary gives a compact one-screen overview of the same release posture, and the release checklist captures the repository-managed release gates and the published bundle contents,
- the backend matrix records the implemented backend catalog and its declared coverage, and the backend-matrix summary provides a compact count-based audit view for maintainers,
- the bundle manifest records the source revision, workspace status, Rust compiler version, profile/API identifiers, and validation-round count alongside deterministic checksums, the verifier treats those manifest values as canonical so leading or trailing whitespace in provenance entries is rejected, and the manifest checksum sidecar keeps the manifest itself tamper-evident,
- the API stability posture records which surfaces are stable versus operational, and the API stability summary provides a compact count-based audit view tagged with the current compatibility-profile identifier,
- the validation report summary provides a compact cross-check of the comparison, house-validation, and benchmark corpus coverage before you open the full validation report, and now repeats the benchmark provenance line so the source revision, workspace status, and Rust compiler version stay visible in the compact audit view,
- the validation report preserves comparison, benchmark, and packaged-data benchmark summaries, and the benchmark sections now include the same provenance line for reproducibility,
- the backend matrix summary and artifact summary give compact audit views and now point straight to `verify-compatibility-profile` and `verify-release-bundle`, so the coverage and artifact audits can hop back to the catalog check as well as the staged-bundle integrity check,
- the manifest records deterministic checksums for the published text artifacts.

If any of those files change, regenerate the bundle from the repository and re-run the verification command.
