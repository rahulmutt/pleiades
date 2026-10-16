# README capability blocks generated from backend metadata (design)

**Status:** design approved, spec under review · **Opened:** 2026-10-16 ·
**Issue:** #166 (the remaining item: hand-written capability prose) ·
**Crates:** `pleiades-validate` (renderer, registry, audit rules, command);
the READMEs of `pleiades-data`, `pleiades-vsop87`, `pleiades-elp`,
`pleiades-jpl` and `pleiades-core`

## Context

#86 corrected the READMEs and added `workspace-audit` rules for the crate
lists and pinned release series (`release/readme_audit.rs`). #178 brought the
`package-check` list under the guard. One item of #166 remains:

> Capability prose in the READMEs is hand-written. The guard covers crate
> lists and version literals only, so a README can again deny or claim a
> capability the crate does not match (as the `pleiades-core` and
> `pleiades-data` READMEs did).

The prose that drifted states facts each backend already declares in its
`BackendMetadata`. Examples are "returns mean geometric coordinates and rejects
apparent requests", the bodies served, and the 1900–2100 window. Today nothing
compares the two.

Measured metadata of the release backends on 2026-10-16 (scratch probe, no
kernels loaded):

| Backend id | Capabilities | Frames / scales | Nominal range | Body claims |
|---|---|---|---|---|
| `pleiades-data` | geocentric, mean, batch | ecliptic, equatorial / TT, TDB | JD 2415020.5 → 2488069.5 (TT) | 21 release-grade (High) |
| `pleiades-vsop87` | same | same | unbounded | 9 constrained (Moderate) |
| `pleiades-elp` | same | same | unbounded | 5 constrained (Moderate), 2 unsupported |
| `jpl-snapshot` | same | same | JD 2378498.5 → 2453000.5 (TDB) | 16 constrained (Unknown) |
| `jpl-spk` | same | same | unbounded | none until kernels are loaded |

Everything except `jpl-spk`'s body claims is fixed by the code. `jpl-spk`'s
claims depend on which SPK kernels the environment loads
(`PLEIADES_DE_KERNEL`, `PLEIADES_AST_KERNEL`), so a README cannot carry them.

## Goal

The capability facts in each release-backend README are generated from that
backend's metadata. They sit inside a marked block, and the blocking-tier
`workspace-audit` fails when a block differs from what the metadata renders
today. Free prose stays free. The facts move into the block, and the duplicate
hand-written capability sentences are removed.

Non-goals:

- **The domain crates' READMEs** (`pleiades-events`, `-eclipse`, `-apparent`,
  `-ayanamsa`, `-houses`, `-types`, `-backend` and others). They describe
  algorithms and APIs, not a backend's capabilities.
- **`FictitiousBackend`** (`pleiades-fict`). Its metadata is derived from the
  backend it wraps, so it has no fixed capabilities of its own.
- **Accuracy figures and maturity notes** (arcseconds against Swiss Ephemeris,
  Pluto's fallback window, "pre-1.0"). They stay prose. The validation gates
  measure them; metadata does not hold them.
- **Chart-layer facts in `pleiades-core`**, for example that `ChartRequest::new`
  defaults to apparent place. They are API defaults, not backend metadata.

## Block format

A block is delimited by HTML comments carrying the backend id:

```markdown
<!-- audit:capabilities pleiades-vsop87 -->
- Coordinates: mean geometric; apparent requests are rejected
- Observer: geocentric; topocentric requests are rejected
- Frames: ecliptic, equatorial · Time scales: TT, TDB
- Window: no nominal limit
- Release-grade bodies: none
- Constrained bodies (Moderate): Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto
<!-- /audit:capabilities -->
```

The text between the markers is exactly the renderer's output for that id,
with one blank line neither before nor after. Rendering is deterministic
(order follows the metadata), and each line comes from one field:

| Line | Source | Rendering |
|---|---|---|
| Coordinates | `capabilities.mean`, `.apparent` | `mean geometric` and/or `apparent`, joined by " and "; a missing one adds "; apparent requests are rejected" or "; mean requests are rejected" |
| Observer | `.geocentric`, `.topocentric` | `geocentric` and/or `topocentric`; a missing topocentric adds "; topocentric requests are rejected" |
| Frames · Time scales | `supported_frames`, `supported_time_scales` | the `Display` of each, lowercased for frames, comma-joined |
| Window | `nominal_range` | `TimeRange::summary_line()`, or "no nominal limit" when both ends are open |
| Bodies, one line per tier | `body_claims` grouped by `tier`, in the order Release-grade, Constrained, Approximate, Unsupported; the label carries the accuracy class when every claim in the tier shares one | `Release-grade bodies: none` is always written; other empty tiers are omitted |
| Sidereal | `capabilities.native_sidereal` | written only when true: "Sidereal: native"; otherwise omitted (no first-party backend has it, and the chart layer applies the ayanamsa) |

`batch`, `deterministic` and `offline` are true for every first-party backend.
They render as one line only when any of them is false ("Not batch-capable",
"Not deterministic", "Needs network access"), so the common case stays short.

**The SPK block's body line.** The registry marks `jpl-spk` as having runtime
bodies. Its body lines are replaced by the fixed sentence `Bodies: those of the
loaded SPK kernels (none without one)`. The block is then the same on every
machine.

## Registry

In `pleiades-validate`, one list names each block:

```rust
pub(crate) struct CapabilityBlock {
    /// Workspace-relative README path.
    pub readme: &'static str,
    /// The backend id, which is also the block key.
    pub backend_id: &'static str,
    /// Builds the metadata without loading kernels or touching the network.
    pub metadata: fn() -> BackendMetadata,
    pub bodies: BodySource,
}

pub(crate) enum BodySource {
    /// Render `body_claims`.
    Metadata,
    /// The bodies are fixed at runtime; render this sentence instead.
    Runtime(&'static str),
}
```

Entries:

- `crates/pleiades-data/README.md`: `pleiades-data`
- `crates/pleiades-vsop87/README.md`: `pleiades-vsop87`
- `crates/pleiades-elp/README.md`: `pleiades-elp`
- `crates/pleiades-jpl/README.md`: `jpl-snapshot` and `jpl-spk` (runtime)

Each `metadata` fn constructs the backend the way the probe did (`default()`,
the unit struct, or `SpkBackend::builder().build()` with no kernels). It never
reads the environment, so the rendering cannot depend on it. A test pins that
each `metadata()` returns the id its entry names.

## Audit rules (`workspace-audit`, blocking tier)

A new module, `release/readme_capabilities.rs`, holds the parser, renderer
and rules. `workspace_audit.rs` calls it once per registered README:

- `readme.capabilities-block-missing`: a registered backend has no block in
  its README.
- `readme.capabilities-block-stale`: the block's text differs from the
  rendering. The detail names the backend, the first differing line, and the
  regeneration command.
- `readme.capabilities-block-unknown`: any README in the workspace has a
  block whose id is not registered, or is registered for a different README.
- `readme.capabilities-block-malformed`: an opening marker without a closing
  one, or a block nested inside another.

All four are ordinary `WorkspaceAuditViolation`s, reported with the existing
ones by `mise run audit`.

## Regeneration command

`pleiades-validate readme-capabilities` rewrites each registered block in
place from the metadata, as `crossings-golden` regenerates its column.
`--check` writes nothing and exits non-zero with the same violations as the
audit. A missing block is not created, because placement is an editorial
choice; the command reports it and the author adds the two markers. Text
outside the blocks is never touched.

It is exposed as `mise run readme-capabilities`, and the stale rule's
message names that task. The command is listed in the `pleiades-validate`
CLI help text beside `workspace-audit`, and the `pleiades-validate` command
block in `docs/cli.md` gains it under a "README capability blocks" comment.

## One-time README cleanup

- **`pleiades-data`:** the intro's hand-written body list ("the Sun, the Moon,
  Mercury through Pluto and the asteroid 433 Eros") and the Status sentence
  "returns mean geometric coordinates and rejects apparent requests" are
  replaced by a `## Capabilities` section holding the block. The intro keeps a
  one-line description, and the Status keeps the maturity note and the
  pointer to the chart layer and event engines.
- **`pleiades-vsop87`, `pleiades-elp`:** the Status sentence "returns mean
  geometric ... rejects apparent and topocentric requests" moves into a
  `## Capabilities` block. The accuracy prose stays.
- **`pleiades-jpl`:** a `## Capabilities` section with the two blocks, each
  under a one-line heading naming its backend.
- **`pleiades-core`:** the sentence "The first-party backends themselves
  return mean geometric coordinates and reject apparent requests" becomes a
  pointer: "Each backend README lists what it returns; the chart layer applies
  apparent place and the topocentric correction on top." The chart-layer
  claims stay as they are.

No README outside these five changes.

## Testing

- **Renderer unit tests** (`readme_capabilities/tests.rs`), on hand-built
  metadata:
  - a mean-only, geocentric-only backend with two tiers;
  - an apparent-and-mean, topocentric backend;
  - an open-ended range;
  - a runtime-bodies entry;
  - a tier with mixed accuracy classes (no class in the label);
  - all of `batch`/`deterministic`/`offline` false.
  
  Expected blocks are written out by hand in the test, not captured from the
  renderer.
- **Rule tests:** missing, stale (the detail names the first differing line),
  unknown id, a block in the wrong README, malformed markers, and a clean pass.
- **Regeneration:** rewriting a stale block gives a README the audit accepts.
  A second run changes nothing (idempotent). Text before and after the block
  is byte-identical. `--check` on a stale README exits non-zero and writes
  nothing.
- **Registry pin:** each entry's `metadata()` id equals its `backend_id`, and
  no two entries share an id.
- **The real tree:** `mise run audit` passes on the cleaned-up READMEs, and
  `pleiades-validate readme-capabilities --check` passes.
- **Environment independence:** with `PLEIADES_DE_KERNEL` set, the rendered
  `jpl-spk` block is unchanged, because the registry never loads kernels. A test
  builds the entry's metadata and checks the runtime sentence.

## Documentation

- `docs/development.md`, beside `mise run audit`: one sentence saying the
  audit also checks the README capability blocks, and that
  `mise run readme-capabilities` regenerates them after a metadata change.
- `docs/cli.md`: the command, as above.
- `AGENTS.md` needs no change. The audit failure names the command.
- #166: the remaining item is checked off.
