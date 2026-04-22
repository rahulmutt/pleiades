# Appendix 1 — Stage-to-Spec Map

This appendix connects the execution plan to the normative specification set.

Use it when:

- deciding which spec documents to reread before starting a stage,
- checking whether a stage plan has drifted from the project requirements,
- reviewing whether a proposed milestone covers the right acceptance criteria.

`SPEC.md` is always the top-level entry point, but each stage has a smaller subset of documents that should drive day-to-day decisions.

## Stage-to-spec traceability

| Stage | Primary spec drivers | Why these docs matter most in this stage |
| --- | --- | --- |
| 1. Workspace bootstrap | [SPEC.md](../../SPEC.md), [spec/architecture.md](../../spec/architecture.md), [spec/requirements.md](../../spec/requirements.md), [spec/validation-and-testing.md](../../spec/validation-and-testing.md) | Stage 1 establishes crate boundaries, purity constraints, tool reproducibility, and baseline quality gates. |
| 2. Domain types and backend contract | [SPEC.md](../../SPEC.md), [spec/backend-trait.md](../../spec/backend-trait.md), [spec/api-and-ergonomics.md](../../spec/api-and-ergonomics.md), [spec/astrology-domain.md](../../spec/astrology-domain.md) | Stage 2 fixes the shared semantics that all later code must obey. |
| 3. Chart MVP and algorithmic baseline | [SPEC.md](../../SPEC.md), [spec/astrology-domain.md](../../spec/astrology-domain.md), [spec/backends.md](../../spec/backends.md), [spec/api-and-ergonomics.md](../../spec/api-and-ergonomics.md), [spec/requirements.md](../../spec/requirements.md) | Stage 3 is where the first useful astrology workflow appears, so it must align tightly with domain and API requirements. |
| 4. Reference backend and validation | [SPEC.md](../../SPEC.md), [spec/backends.md](../../spec/backends.md), [spec/validation-and-testing.md](../../spec/validation-and-testing.md), [spec/requirements.md](../../spec/requirements.md) | Stage 4 turns correctness and provenance into reproducible evidence. |
| 5. Compression and packaged data | [SPEC.md](../../SPEC.md), [spec/data-compression.md](../../spec/data-compression.md), [spec/backends.md](../../spec/backends.md), [spec/validation-and-testing.md](../../spec/validation-and-testing.md) | Stage 5 defines the packaged 1500-2500 product and its quality envelope. |
| 6. Compatibility expansion and release hardening | [SPEC.md](../../SPEC.md), [spec/requirements.md](../../spec/requirements.md), [spec/astrology-domain.md](../../spec/astrology-domain.md), [spec/validation-and-testing.md](../../spec/validation-and-testing.md), [spec/roadmap.md](../../spec/roadmap.md) | Stage 6 closes remaining compatibility gaps and converts the project into a dependable release process. |

## Cross-cutting spec reminders

These spec constraints apply in every stage, even when they are not the main focus:

- [spec/architecture.md](../../spec/architecture.md): no layer violations, no dependency cycles, and no source-specific logic in generic domain crates.
- [spec/api-and-ergonomics.md](../../spec/api-and-ergonomics.md): public APIs should stay strongly typed, deterministic, batch-friendly, and explicit about assumptions.
- [spec/requirements.md](../../spec/requirements.md): the end-state compatibility target is broader than the first implementation milestone; do not design APIs around the baseline subset alone.
- [spec/validation-and-testing.md](../../spec/validation-and-testing.md): validation is a release gate, not a cleanup task deferred indefinitely.

## Review checklist for plan changes

When changing a stage document, quickly verify:

1. the stage still advances the repository toward a workable state,
2. the stage does not narrow the long-term compatibility target,
3. the stage still respects the crate layering in `spec/architecture.md`,
4. the stage's exit criteria are backed by the right spec documents,
5. the relevant checklist or track doc is updated if expectations changed.
