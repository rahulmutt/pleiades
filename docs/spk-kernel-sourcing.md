# SPK Kernel Sourcing

The `jpl-spk` backend and the reference-corpus generator read a public-domain
JPL DE SPK kernel that is **not** committed to this repository (it is ~114 MB).

## Kernel

- File: `de440.bsp`
- Source: NASA/JPL NAIF generic kernels —
  `https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440.bsp`
- License: public domain (U.S. Government work).
- SHA-256: `a4ce9bf9b3282becc9f4b2ac3cebe03a2ae7599981aabd7265fd8482fff7c4b5`

## Coverage

`de440.bsp` covers approximately **1550-01-01 to 2650-01-01**. The project's
target packaged range is **1600-2600 CE**, which sits entirely within de440's
coverage, so there is no floor gap for the target range. The backend advertises
the kernel's *actual* coverage (read from its segment descriptors), and release
profiles record that advertised window. The full-historic `de441` kernel
(~3 GB) is not required for the target range.

## Asteroid kernel (Tier A — pinned)

Selected-asteroid main-belt coverage reads a JPL small-body perturber kernel,
**not** committed to this repository.

- File: `sb441-n16.bsp`
- Source: NASA/JPL SSD/NAIF small-body perturber set, fitted consistently with
  DE441 (agrees with de440 over the overlap) —
  `https://ssd.jpl.nasa.gov/ftp/eph/small_bodies/asteroids_de441/sb441-n16.bsp`
- License: public domain (U.S. Government work).
- SHA-256: `919d612ce3c72a78fc7158f9120156542d0f21e6b8b052e4c1339c759747fd90`
- Size: 645,727,232 bytes (full DE441 interval).
- Bodies: the 16 most-massive perturbers (Ceres, Pallas, Juno, Vesta, Hygiea,
  Psyche, Iris, …); the curated subset used here is the Tier A roster (Ceres,
  Pallas, Juno, Vesta, Hygiea, Psyche, Iris — all confirmed present).
- Regenerate the committed slice with:
  `PLEIADES_DE_KERNEL=… PLEIADES_AST_KERNEL=… cargo run -p pleiades-jpl --bin regenerate-asteroid-corpus`
- Default asteroid window: 1900–2100 CE (the corpus samples only this window;
  the kernel itself covers the full DE441 interval).

Usage / reproduction:

```bash
PLEIADES_DE_KERNEL=/path/to/de440.bsp \
PLEIADES_AST_KERNEL=/path/to/sb441-n16.bsp \
  cargo test -p pleiades-jpl --test corpus_regen -- --nocapture
```

## Asteroid slices (Tier B — Horizons-sourced, constrained)

Centaurs, personal asteroids, and TNOs are **not** in any fixed perturber
kernel. They are generated once via JPL Horizons over 1900–2100 using
`pleiades_jpl::ingest` (see the `horizons-fetch` feature) and committed as the
provenance-validated `asteroid_constrained` slice — never put behind the kernel
regen gate.

- Bodies: see `crates/pleiades-jpl/src/spk/asteroid_roster.rs` (Tier B entries).
- Solution epoch / generation date: `<recorded-in-Task-11>`
- Recipe: `<exact Horizons request recorded in Task 11>`

## Usage

```bash
PLEIADES_DE_KERNEL=/path/to/de440.bsp \
  cargo test -p pleiades-jpl --test spk_full_kernel -- --nocapture
```

## Corpus reproduction check

To verify that the checked-in reference corpus is reproducible from the real
kernel, run the gated integration test:

```bash
PLEIADES_DE_KERNEL=/path/to/de440.bsp \
  cargo test -p pleiades-jpl --test corpus_regen -- --nocapture
```

Without the env var the test compiles and passes immediately via early return
(skip). With the kernel, it regenerates each boundary-slice row and asserts
that values match the checked-in CSV within 1 km.
