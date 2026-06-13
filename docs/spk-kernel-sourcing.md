# SPK Kernel Sourcing

The `jpl-spk` backend and the reference-corpus generator read a public-domain
JPL DE SPK kernel that is **not** committed to this repository (it is ~114 MB).

## Kernel

- File: `de440.bsp`
- Source: NASA/JPL NAIF generic kernels —
  `https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440.bsp`
- License: public domain (U.S. Government work).
- SHA-256: `<fill in after downloading: shasum -a 256 de440.bsp>`

## Coverage and the 1500 CE known gap

`de440.bsp` covers approximately **1550-01-01 to 2650-01-01**. The project's
target packaged range is **1500-2500 CE**, so the **1500-1550 CE window is a
known gap** in this slice. The backend advertises the kernel's *actual* coverage
(read from its segment descriptors), and release profiles must record the gap
rather than claim 1500. Closing it requires `de441` (full historic range,
~3 GB) and is deferred to a later slice.

## Asteroid kernel (optional)

Selected-asteroid coverage requires a small-body SPK kernel (e.g. an
`astNNN_de440.bsp` distribution). Record its file name, source URL, license, and
SHA-256 here when adopted.

## Usage

```bash
PLEIADES_DE_KERNEL=/path/to/de440.bsp \
  cargo test -p pleiades-jpl --test spk_full_kernel -- --nocapture
```
