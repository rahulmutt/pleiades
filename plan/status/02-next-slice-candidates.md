# Status 2 — Next Slice Candidates

This document translates the stage plan into a shortlist of **good next changes**.

It is intentionally conservative: each slice is small enough to review, test, and release without turning Stage 6 into an unfocused backlog.

## Selection rules

Choose slices that satisfy all of the following:

- preserve a workable repository state on their own,
- improve either compatibility clarity, release confidence, or reproducibility,
- fit within existing crate boundaries,
- can be validated with focused tests and updated release-facing docs,
- do not require redesign of Stage 2 shared contracts.

## Best next-slice themes

### 1. Compatibility-profile tightening

Good slices:

- add one missing interoperability alias batch and cover it with resolver tests,
- improve wording that currently blurs aliases, caveats, validation reference points, or custom definitions,
- add one missing compact summary or profile cross-reference that helps release review.

Why this is high priority:

- `spec/requirements.md` makes the release compatibility profile a required public artifact,
- Stage 6 depends on keeping claims explicit as breadth grows,
- these changes are usually low-risk and immediately improve maintainability.

### 2. Remaining compatibility-catalog batches

Good slices:

- add one coherent family of house systems that share formulas or interoperability context,
- add one coherent ayanamsa batch with metadata, aliases, and profile coverage,
- close one documented gap where the release profile already announces future support.

Guardrails:

- catalog additions must include tests,
- update compatibility-profile output in the same slice,
- do not mix unrelated chart-helper work into the same change.

### 3. Packaged-data and artifact reproducibility refinement

Good slices:

- improve generation metadata recorded in artifacts or staged bundles,
- add one missing validation check around segment edges, unsupported channels, or fallback behavior,
- extend packaged coverage only where the validation story is already strong,
- harden the bundle-manifest sidecar or other release-surface checks when a canonical format or tamper-evidence rule is still implicit.

Guardrails:

- avoid changing the artifact format casually,
- prefer validation/reporting improvements before broad coverage expansion,
- keep public provenance explicit.

### 4. Validation and benchmarking hardening

Good slices:

- add one representative benchmark workload that is currently under-covered,
- preserve one newly discovered regression as a durable test/report fixture,
- improve one capability matrix field so backend limits are easier to audit.

Guardrails:

- benchmark additions should represent real chart or packaged-data workloads,
- validation output should remain reproducible from checked-in or documented inputs.

### 5. API stabilization and documentation polish

Good slices:

- clarify rustdoc around units, ranges, apparentness, or failure modes,
- narrow one unstable or ambiguous helper surface with explicit stability notes,
- add one example that demonstrates the intended facade/backend/domain split.

Guardrails:

- prefer documentation and stability posture improvements over convenience expansion,
- avoid breaking API changes unless they are clearly justified and documented.

## Slice sizing guidance

A strong slice usually changes one of these combinations:

- one resolver + one compatibility-profile update + one test batch,
- one validation command/report improvement + one documentation update,
- one packaged-artifact verification improvement + one release-bundle update,
- one coherent catalog batch + its release-facing metadata.

A weak slice usually tries to do too many of these at once.

## Suggested default order when multiple slices compete

If several candidate slices are available, prefer them in this order:

1. fixes to release-profile accuracy,
2. fixes to verification or reproducibility gaps,
3. compatibility breadth that is already well specified,
4. validation corpus improvements,
5. optional helper expansion.

## When to create a new stage instead of extending Stage 6

Do **not** create a Stage 7 just to hold miscellaneous backlog.

A new stage should exist only if the project develops a new class of work with a distinct workable-state promise, for example:

- a major new backend family that changes distribution strategy,
- a new data-product tier beyond the current packaged 1500-2500 artifact line,
- a substantially new consumer-facing API layer with its own stability posture.

Until then, Stage 6 should remain the umbrella for compatibility completion and release hardening, with this file used to keep the work sliced sensibly.
