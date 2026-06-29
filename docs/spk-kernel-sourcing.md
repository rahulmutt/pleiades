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

Selected-asteroid coverage reads a JPL small-body perturber kernel,
**not** committed to this repository.

- File: `sb441-n373s.bsp`
- Source: NASA/JPL SSD/NAIF small-body perturber set, fitted consistently with
  DE441 (agrees with de440 over the overlap) —
  `https://ssd.jpl.nasa.gov/ftp/eph/small_bodies/asteroids_de441/sb441-n373s.bsp`
- License: public domain (U.S. Government work).
- SHA-256: `2143113282bfc2b2a0b0b4626125d4f84362339b5a8ae7eea40f4120ca8da10b`
- Size: ~937 MB (982,106,112 bytes).
- Bodies: 343 main-belt perturbers + 30 KBOs, DE441-consistent. Supersedes the
  retired 16-body `sb441-n16`. The curated Tier-A subset used here is 25 bodies:
  the original 9 (Ceres, Pallas, Juno, Vesta, Hygiea, Psyche, Iris, Eunomia,
  Cybele) plus 16 newly promoted bodies (see astrological usage below).
- Verified coverage window: 1900–2100 CE (confirmed for all 25 Tier-A bodies).
- Astrological usage (gate 2 — promoted bodies):
  - 5 Astraea: Greek goddess of justice/innocence (Astraea/Dike); listed by
    number and name in the Swiss Ephemeris asteroid name catalog (`seasnam.txt`,
    Astrodienst/astro.com) and used in the asteroid-astrology interpretive
    tradition (cf. Martha Lang-Wescott, *Mechanics of the Future: Asteroids*).
  - 6 Hebe: Greek goddess of youth, cupbearer to the gods; same catalog +
    tradition.
  - 8 Flora: Roman goddess of flowers and spring; same catalog + tradition.
  - 9 Metis: Titaness of wisdom/counsel, first wife of Zeus; same catalog +
    tradition.
  - 19 Fortuna: Roman goddess of fortune/luck (distinct from the Part-of-Fortune
    chart point); same catalog + tradition.
  - 80 Sappho: the archaic Greek poet of Lesbos (not a deity); themes of love,
    poetry, and friendship in the asteroid-astrology tradition; same catalog.
  - 433 Eros: Greek god of erotic love/desire; a core "personal" asteroid in
    the tradition; same catalog + tradition.
  - TNOs/dwarf planets used in modern outer-body astrology (all confirmed in
    `sb441-n373s`): 136199 Eris (Greek goddess of strife/discord), 90377 Sedna
    (Inuit sea goddess), 136108 Haumea (Hawaiian goddess of fertility/childbirth),
    136472 Makemake (Rapa Nui creator god), 50000 Quaoar (Tongva creation deity),
    90482 Orcus (Etruscan/Roman god of the underworld and broken oaths), 225088
    Gonggong (Chinese water deity), 20000 Varuna (Vedic god of cosmic order,
    waters, and the sky), 28978 Ixion (Greek mythological figure, bound to a
    fiery wheel); all listed in the Swiss Ephemeris asteroid name catalog
    (`seasnam.txt`, Astrodienst/astro.com) and used in modern TNO/outer-body
    astrology.
- Regen coverage: the regeneration recipe filters the corpus by the Tier-A
  roster, so all 25 Tier-A bodies are automatically included in subsequent
  regeneration runs — no recipe edit is required.
- Regenerate the committed slice with:
  `PLEIADES_DE_KERNEL=… PLEIADES_AST_KERNEL=… cargo run -p pleiades-jpl --bin regenerate-asteroid-corpus`
- Default asteroid window: 1900–2100 CE (the corpus samples only this window;
  the kernel covers the full DE441 interval).

Usage / reproduction:

```bash
PLEIADES_DE_KERNEL=/path/to/de440.bsp \
PLEIADES_AST_KERNEL=/path/to/sb441-n373s.bsp \
  cargo test -p pleiades-jpl --test corpus_regen -- --nocapture
```

## Asteroid slices (Tier B — Horizons-sourced, constrained)

Centaurs, personal asteroids, and TNOs are **not** in any fixed perturber
kernel. They are generated once via JPL Horizons over 1900–2100 using
`pleiades_jpl::ingest` (see the `horizons-fetch` feature) and committed as the
provenance-validated `asteroid_constrained` slice — never put behind the kernel
regen gate.

- Bodies: see `crates/pleiades-jpl/src/spk/asteroid_roster.rs` (Tier B entries) —
  5 centaurs, 6 personal/minor main-belt bodies (Amor, Lilith, Hidalgo, Icarus,
  Toro, Apollo); 11 bodies total. All 9 TNOs promoted to Tier A in slice 2.
- Generation date: 2026-06-17, from JPL Horizons (per-object JPL small-body
  solutions, consistent with the DE441 small-body framework).
- Recipe: `cargo run -p pleiades-jpl --features horizons-fetch --bin regenerate-asteroid-constrained`.
  Each body is fetched with `EPHEM_TYPE=VECTORS, VEC_TABLE=1, REF_PLANE=ECLIPTIC,
  CENTER='500@399', OUT_UNITS=KM-S, COMMAND='<IAU-number>;'`, sampled at its
  class cadence (main-belt 180 d, centaur 365 d, TNO 1825 d). Because Horizons
  solutions update over time, this slice is **not** byte-reproducible and is
  validated by window/schema/provenance, never the kernel regen gate.

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
