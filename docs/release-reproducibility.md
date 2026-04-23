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

## Release smoke workflow

The repository also ships a release-style smoke check that exercises the validation bundle path end to end:

```bash
mise run release-smoke
```

That task generates a temporary release bundle, then verifies the staged manifest checksums using `pleiades-validate`.

## Manual bundle generation

To inspect the release artifacts directly, generate a bundle in a directory of your choice:

```bash
cargo run -q -p pleiades-validate -- bundle-release --out /tmp/pleiades-release
```

The bundle currently writes these text artifacts:

- `compatibility-profile.txt`
- `backend-matrix.txt`
- `api-stability.txt`
- `validation-report.txt`
- `bundle-manifest.txt`

Verify the staged bundle with:

```bash
cargo run -q -p pleiades-validate -- verify-release-bundle --out /tmp/pleiades-release
```

## What the bundle is for

The release bundle makes the current release posture easy to reproduce and audit:

- the compatibility profile captures shipped house systems, ayanamsas, aliases, and known gaps,
- the backend matrix records the implemented backend catalog and its declared coverage,
- the API stability posture records which surfaces are stable versus operational,
- the validation report preserves comparison and benchmark summaries,
- the manifest records deterministic checksums for the published text artifacts.

If any of those files change, regenerate the bundle from the repository and re-run the verification command.
