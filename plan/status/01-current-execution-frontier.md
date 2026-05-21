# Status 1 — Current Execution Frontier

## Frontier

The active frontier is **Phase 1: Production Reference/Source Corpus**. Production compressed-data work depends on a broader source and hold-out corpus, so artifact changes must not broaden release claims until Phase 1 is complete.

Recent implementation note: the consolidated source-corpus posture now also has a standalone `source-corpus-summary` / `source-corpus` surface in the validation and CLI front-ends, so the release-grade guard, JPL source-corpus contract, and phase-2 corpus alignment can be inspected directly without going through the larger release summary. The source provenance surfaces now also spell out the shared schema label explicitly, so frame/time-scale/column posture is paired with a visible schema contract in the release-facing corpus blocks. Release bundle rendering and verification now also carry the standalone source-corpus summary file and checksum, so the bundle contract stays aligned with the top-level source-corpus surface.

## Why this frontier comes first

The specification requires compressed 1500-2500 CE data to be reproducible from public inputs and validated against measured error envelopes. The current artifact and reports are useful infrastructure, but current comparison output still shows production tolerance failures. A better generator alone is insufficient until the source corpus is broad, documented, and separated into fitting/reference/hold-out evidence.

Recent implementation note: production-generation source-summary validation now checks the exact rendered source-revision checksum payload, so checksum drift in the corpus provenance block is no longer only implied by nested struct equality. The new production-generation corpus-shape summary now ties the validated source summary to both the ecliptic and equatorial boundary request corpora, so body order, epochs, frame, time scale, columns, apparentness, and checksum posture are surfaced together in release-facing reports. Source-revision checksums are also exposed through a dedicated `production-generation-source-revision-summary` CLI/report surface, validated against the current fixture checksums, and bundled/verified as a first-class release artifact. The production-generation source summary now also spells out the evidence-class split (`reference`, `hold-out`, `boundary overlay`, `provenance-only`) and now makes the license posture explicit alongside redistribution posture, and its cadence fragment is now derived from the checked-in source-window and boundary-request corpus counts rather than a hardcoded prose pair, so the source-window payload stays partitioned after rendering changes. Reference and hold-out source summaries now also carry explicit frame/time-scale posture fields, and the release-facing body-class coverage summaries now reuse the shared celestial-body class taxonomy. JPL reference and hold-out provenance summaries now also carry explicit evidence-class labels and now record explicit frame-treatment and time-scale posture, the boundary-overlay render path preserves that label in release-facing output, the JPL source corpus contract now has a typed, fail-closed release-facing summary, and that contract now also has a direct CLI/report entrypoint for release-audit checks. The JPL evidence summary now renders a dedicated provenance-only posture line, the comparison snapshot manifest summary now fails closed on redistribution drift, the backend matrix summary now folds in the comparison corpus release-grade guard, reference/hold-out overlap, independent hold-out, release-grade body claims, Pluto fallback posture, and validated production-generation corpus claims, release bundles now carry independent-holdout body-class coverage alongside the existing source-window evidence, release bundles now also carry the production-generation corpus-shape summary alongside the source-window and manifest summaries while directory verification fails closed if it is missing, and the release summary now surfaces validated production-generation body-class coverage and corpus-shape lines alongside the release-grade body-claims line. The independent hold-out fixture was also broadened with selected-asteroid anchors at JD 2378498.5, JD 2451545, JD 2451917.5, JD 2453000.5, and JD 2500000, so hold-out validation now spans 65 rows across 16 bodies and 13 epochs while the major-body boundary slice remained unchanged. The compatibility inventory now surfaces ayanamsa alias-bearing entry counts so the alias/provenance audit remains visible in release-facing summaries. The production-generation source summary now also carries an explicit apparentness posture (`apparentness=Mean`) and schema posture (`schema=epoch_jd, body, x_km, y_km, z_km`) so the corpus provenance line and the boundary-request corpus contract stay aligned on request semantics. Selected-asteroid source evidence and source-window summaries now also expose explicit evidence-class, frame, and time-scale posture, keeping the selected-asteroid corpus slice aligned with the broader provenance contract. The release summary, validation report, and compact corpus views now also surface a consolidated source-corpus line combining the comparison corpus release-grade guard, the JPL source corpus contract, and the phase-2 corpus alignment, so provenance and coverage stay visible in the top-level release surfaces.

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
