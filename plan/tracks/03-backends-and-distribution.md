# Track 3 — Backends and Distribution

## Role

Advance backend implementations, source corpora, and compressed distribution
without coupling the public API to one source family.

## Standards

- Source-specific backend crates may depend only on lower-layer crates permitted
  by `spec/architecture.md`.
- Backend metadata must reflect actual body, time-range, frame, apparentness,
  observer, channel, and accuracy support.
- Packaged artifacts must be reproducible from documented public inputs.

## Remaining backend goals

- Production-grade 1600-2600 CE compressed artifact (the 1900-2100 release
  artifact is shipped; the wider span is opt-in via `generate-artifact`, not yet
  gated).
- Pluto, fuller lunar theory/lunar points, and selected asteroid claim closure.
- Empirical accuracy documentation for each advertised backend path.
