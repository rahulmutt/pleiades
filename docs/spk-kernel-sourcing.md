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

## Asteroid kernel (optional)

Selected-asteroid coverage requires a small-body SPK kernel (e.g. an
`astNNN_de440.bsp` distribution). Record its file name, source URL, license, and
SHA-256 here when adopted.

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
