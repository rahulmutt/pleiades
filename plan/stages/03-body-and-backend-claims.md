# Phase 3 — Body and Backend Claim Closure

## Goal

Ensure every public body/backend claim is source-backed, artifact-backed,
constrained, approximate, or unsupported with no ambiguous status.

## Current baseline

- First-party backend traits and capability metadata exist.
- VSOP87-style major-planet paths, a compact lunar baseline, checked-in JPL
  snapshots, and a draft packaged-data backend exist.
- Current reports explicitly separate release-grade major-body claims from Pluto
  fallback posture and selected-asteroid fixture evidence.

## Remaining implementation work

- Resolve Pluto status by either implementing a validated source-backed path,
  keeping it explicitly approximate, or excluding it from release-grade claims.
- Decide whether to implement a fuller lunar theory/ELP-backed path or constrain
  lunar/lunar-point claims to the compact Meeus-style baseline.
- Promote Ceres, Pallas, Juno, Vesta, and any custom/numbered asteroid support
  only where source evidence and backend metadata are broad enough.
- Keep extensible body identifiers without implying unsupported bodies are
  available from all backends.
- Audit backend capability metadata against actual supported bodies, dates,
  frames, time scales, coordinate channels, observer policy, and apparentness.
- Preserve metadata preflight so unsupported request shapes fail before backend
  computation.

## Exit criteria

- Backend matrices, compatibility profiles, release summaries, CLI output, and
  rustdoc agree on body/backend support and limitations.
- No release-facing surface advertises unsupported or approximate bodies as
  production-grade.
