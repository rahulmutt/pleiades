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

The CI gate also runs a workspace-native dependency audit:

```bash
mise run audit
```

## Release smoke workflow

The repository also ships a release-style smoke check that exercises the validation bundle path end to end:

```bash
mise run release-smoke
```

That task first runs the workspace audit, validates the bundled compressed artifact, then generates a temporary release bundle and verifies the staged manifest checksums using `pleiades-validate`.

## Manual bundle generation

To inspect the release artifacts directly, generate a bundle in a directory of your choice:

```bash
cargo run -q -p pleiades-validate -- bundle-release --out /tmp/pleiades-release
```

The validation tool can also render the compact profile summary, release notes, and release checklist directly when you only need the individual maintainer-facing artifacts:

```bash
cargo run -q -p pleiades-validate -- compatibility-profile-summary
cargo run -q -p pleiades-validate -- release-notes
cargo run -q -p pleiades-validate -- release-checklist
```

The bundle currently writes these text artifacts:

- `compatibility-profile.txt`
- `compatibility-profile-summary.txt`
- `release-notes.txt`
- `release-checklist.txt`
- `backend-matrix.txt`
- `api-stability.txt`
- `validation-report.txt`
- `bundle-manifest.txt` (includes the recorded source revision, workspace status, Rust compiler version, profile/API identifiers, and validation-round count)

The generated `release-checklist.txt` now also embeds the canonical `bundle-release` and `verify-release-bundle` commands plus a pointer back to this guide, so the bundle stays self-describing for maintainers.

Verify the staged bundle with:

```bash
cargo run -q -p pleiades-validate -- verify-release-bundle --out /tmp/pleiades-release
```

## What the bundle is for

The release bundle makes the current release posture easy to reproduce and audit:

- the compatibility profile captures shipped house systems, ayanamsas, aliases, and known gaps,
- the compatibility profile summary gives a compact count-based view of the same release posture,
- the release notes file summarizes release-specific coverage, known limitations, and the current API stability / deprecation-policy snapshot,
- the release checklist captures the repository-managed release gates and the published bundle contents,
- the backend matrix records the implemented backend catalog and its declared coverage,
- the bundle manifest records the source revision, workspace status, Rust compiler version, profile/API identifiers, and validation-round count alongside deterministic checksums,
- the API stability posture records which surfaces are stable versus operational,
- the validation report preserves comparison and benchmark summaries,
- the manifest records deterministic checksums for the published text artifacts.

If any of those files change, regenerate the bundle from the repository and re-run the verification command.
