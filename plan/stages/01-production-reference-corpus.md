# Phase 1 — Production Reference/Source Corpus

## Goal

Provide public, documented, deterministic source inputs broad enough to support release body claims, backend validation, hold-out testing, and compressed-artifact generation.

## Starting point

The repository has checked-in JPL Horizons snapshots, comparison fixtures, selected-asteroid slices, and hold-out rows. They are valuable regression evidence, but are still described as repository-checked fixtures rather than a broad production corpus or general source reader.

## Implementation goals

- Choose the production source strategy: broader checked-in public fixtures, a pure-Rust reader/parser for public ephemeris files, or a documented hybrid.
- Record provenance, source revisions, license/redistribution posture, frame, time scale, schema, generation command, checksums, and evidence class for every corpus member.
- Cover every release-claimed body/channel/frame over the advertised date range with separate reference and hold-out sets.
- Keep boundary overlays, fixture-exactness samples, provenance-only rows, and validation rows separate in data and reports.
- Make corpus expansion reproducible from public inputs without network access during normal tests.

Progress update: the production-generation boundary-request corpus parity check now validates the ecliptic/equatorial request corpora field-by-field and fails closed on drift in request count, body count, bodies, epoch count, earliest/latest epoch, time scale, zodiac mode, or apparentness. The consolidated source-corpus summary now also carries the production-generation source provenance line, keeping the public-input provenance block aligned with the standalone source summary, and now also surfaces the production-generation date range explicitly alongside the source-window payload. The source-corpus summary validation now also exercises the comparison-corpus guard and the JPL contract/classification/provenance-only fields directly, so the consolidated provenance block fails closed on the remaining top-level corpus posture lines as well. The source-corpus construction now also rejects duplicated JPL label prefixes in nested payloads, keeping those embedded audit labels normalized rather than silently double-prefixed. The independent-holdout source revision checksum and the source-corpus alias expectations were resynced after the 84-row hold-out boundary expansion, so the staged provenance text now matches the current CSV payload. The body/date/channel claims summary now also carries the production-generation coverage posture explicitly, keeping release-facing claim summaries tied to the current corpus coverage line. The reference-snapshot mixed TT/TDB batch parity summary now also has a CLI alias, and the lunar reference mixed TT/TDB batch parity summary now also has a direct validation/CLI entrypoint, keeping the phase-1 batch-shape evidence user-facing in both reference corpora. The production-generation body-class coverage summary validation now also reports field-specific drift for the major-body and selected-asteroid slices instead of collapsing every mismatch to the row-count field.

## Completion criteria

- Validation commands can identify the corpus version and reproduce checksums from a clean checkout.
- Hold-out samples exercise the same release bodies and channels as the fitting corpus without being consumed by fitting.
- Release profiles and backend matrices cannot claim broader body/date/channel support than the corpus validates.
