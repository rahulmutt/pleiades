# Status 1 — Current Execution Frontier

## Frontier

The active frontier is **Phase 1: Production Reference/Source Corpus**. Production compressed-data work depends on a broader source and hold-out corpus, so artifact changes must not broaden release claims until Phase 1 is complete.

## Why this frontier comes first

The specification requires compressed 1500-2500 CE data to be reproducible from public inputs and validated against measured error envelopes. The current artifact and reports are useful infrastructure, but current comparison output still shows production tolerance failures. A better generator alone is insufficient until the source corpus is broad, documented, and separated into fitting/reference/hold-out evidence.

Recent implementation note: production-generation source-summary validation now checks the exact rendered source-revision checksum payload, so checksum drift in the corpus provenance block is no longer only implied by nested struct equality. The new production-generation corpus-shape summary also ties the validated source summary to the ecliptic boundary request corpus, so body order, epochs, frame, time scale, columns, apparentness, and checksum posture are surfaced together in release-facing reports. Source-revision checksums are also exposed through a dedicated `production-generation-source-revision-summary` CLI/report surface, validated against the current fixture checksums, and bundled/verified as a first-class release artifact. The production-generation source summary now also spells out the evidence-class split (`reference`, `hold-out`, `boundary overlay`, `provenance-only`) and now makes the license posture explicit alongside redistribution posture, so the source-window payload stays partitioned after rendering changes. JPL reference and hold-out provenance summaries now also carry explicit evidence-class labels, the boundary-overlay render path preserves that label in release-facing output, the JPL source corpus contract now has a typed, fail-closed release-facing summary, the JPL evidence summary now renders a dedicated provenance-only posture line, the comparison snapshot manifest summary now fails closed on redistribution drift, the backend matrix summary now folds in the comparison corpus release-grade guard, reference/hold-out overlap, independent hold-out, release-grade body claims, Pluto fallback posture, and validated production-generation corpus claims, release bundles now carry independent-holdout body-class coverage alongside the existing source-window evidence, and release bundles now also carry the production-generation corpus-shape summary alongside the source-window and manifest summaries while directory verification fails closed if it is missing. The compatibility inventory now surfaces ayanamsa alias-bearing entry counts so the alias/provenance audit remains visible in release-facing summaries.

## Immediate blockers

1. **Production source coverage** — checked-in snapshots do not yet form a broad production corpus for all release bodies, channels, and epochs.
2. **Claim boundaries** — Pluto, full lunar theory/lunar points, and selected asteroids need source-backed validation or explicit constrained/excluded status.
3. **Artifact promotion** — packaged data must remain draft-grade until it passes thresholds against production reference and hold-out corpora.
4. **Compatibility evidence** — broad house and ayanamsa catalogs need formula/provenance audits before stronger compatibility claims.

## Recommended next slice

Broaden the production corpus around the new contract surface:

- add or generate representative source coverage for every release-claimed body/channel/frame;
- separate fitting, reference, boundary, hold-out, and provenance-only evidence in the source corpus itself;
- derive backend matrices and release profiles from validated corpus evidence;
- wire any remaining corpus validation gaps into release-facing reports without manually curated summary prose.

## Parallel safe work

- Improve packaged-artifact fitting and reconstruction behind draft labels.
- Audit body release status for Pluto, lunar channels, and selected asteroids.
- Audit house/ayanamsa entries whose release status is stronger than their evidence.
- Keep unsupported request modes documented and structurally rejected.
- Harden release gates where they check current evidence rather than expanding claims.
