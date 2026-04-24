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

The validation tool can also render the full compatibility profile, the compact profile summary, verify the release compatibility profile against the canonical catalogs, render the release notes, release-notes summary, release checklist, release-checklist summary, release summary, API stability summary, backend matrix summary, artifact summary, and validation summary directly when you only need the individual maintainer-facing artifacts, and the user-facing CLI mirrors the release-notes, release-notes-summary, release-checklist, release-checklist-summary / checklist-summary, release-summary, verify-compatibility-profile, api-stability-summary, backend-matrix-summary / matrix-summary, artifact-summary / artifact-posture-summary, and validation-report-summary / validation-summary / report-summary renderers too; the release notes now also point readers at the compact compatibility-profile-summary view for a quicker audit hop, the compact release-notes summary gives maintainers a lighter-weight release-notes view and a direct hop to verify-compatibility-profile, the release-checklist summary now also points back to the compact release-summary view for a quicker one-screen audit hop and the `verify-release-bundle` verifier for the staged bundle audit, and the compact release summary now also includes the custom-definition label counts that keep the release posture self-describing, a packaged-artifact cross-reference, and the compact validation-report-summary / validation-summary / report-summary alias in its summary-view list:

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
cargo run -q -p pleiades-cli -- release-notes
cargo run -q -p pleiades-cli -- release-notes-summary
cargo run -q -p pleiades-cli -- release-checklist
cargo run -q -p pleiades-cli -- release-checklist-summary
cargo run -q -p pleiades-cli -- checklist-summary
cargo run -q -p pleiades-cli -- release-summary
cargo run -q -p pleiades-cli -- verify-compatibility-profile
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
- `validation-report-summary.txt`
- `validation-report.txt`
- `bundle-manifest.txt` (includes the recorded source revision, workspace status, Rust compiler version, profile/API identifiers, and validation-round count)
- `bundle-manifest.checksum.txt` (records the checksum used to verify the staged manifest itself; the verifier expects the canonical single-line `0x...` format with no stray whitespace)

The generated `release-checklist.txt` now also embeds the canonical `bundle-release` and `verify-release-bundle` commands plus a pointer back to this guide, while `release-checklist-summary.txt` provides a compact audit view for quick release review, so the bundle stays self-describing for maintainers.

Verify the staged bundle with:

```bash
cargo run -q -p pleiades-validate -- verify-release-bundle --out /tmp/pleiades-release
```

The verifier expects exactly the staged bundle files listed above, so stray files or missing entries will cause verification to fail.

## What the bundle is for

The release bundle makes the current release posture easy to reproduce and audit:

- the compatibility profile captures shipped house systems, ayanamsas, aliases, validation reference points, and compatibility caveats, and `verify-compatibility-profile` provides a quick catalog-alignment audit for the same release surface,
- the compatibility profile summary gives a compact count-based view of the same release posture, including validation reference points,
- the release notes file summarizes release-specific coverage, validation reference points, known limitations, and the current API stability / deprecation-policy snapshot, the release-notes summary gives a compact release-notes view, the release summary gives a compact one-screen overview of the same release posture, and the release checklist captures the repository-managed release gates and the published bundle contents,
- the backend matrix records the implemented backend catalog and its declared coverage, and the backend-matrix summary provides a compact count-based audit view for maintainers,
- the bundle manifest records the source revision, workspace status, Rust compiler version, profile/API identifiers, and validation-round count alongside deterministic checksums, and the manifest checksum sidecar keeps the manifest itself tamper-evident,
- the API stability posture records which surfaces are stable versus operational, and the API stability summary provides a compact count-based audit view tagged with the current compatibility-profile identifier,
- the validation report summary provides a compact cross-check of the comparison, house-validation, and benchmark corpus coverage before you open the full validation report,
- the validation report preserves comparison, benchmark, and packaged-data benchmark summaries,
- the manifest records deterministic checksums for the published text artifacts.

If any of those files change, regenerate the bundle from the repository and re-run the verification command.
