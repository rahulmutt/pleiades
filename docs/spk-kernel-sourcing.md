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
  retired 16-body `sb441-n16`. The curated Tier-A subset sourced from this kernel
  is 25 bodies: the original 9 (Ceres, Pallas, Juno, Vesta, Hygiea, Psyche, Iris,
  Eunomia, Cybele) plus 16 promoted bodies (see astrological usage below). An
  additional 11 bodies absent from sb441-n373s are covered by per-object SPKs
  (see section below), bringing total Tier-A to 36.
- Verified coverage window: 1900–2100 CE (confirmed for all 25 kernel-sourced Tier-A bodies).
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
  roster, so all 36 Tier-A bodies are automatically included in subsequent
  regeneration runs — no recipe edit is required.
- Regenerate the committed slice with:
  `PLEIADES_DE_KERNEL=.kernels/de440.bsp PLEIADES_AST_KERNEL=.kernels/sb441-n373s.bsp PLEIADES_OBJECT_SPK_DIR=.kernels/objects cargo run -p pleiades-jpl --bin regenerate-asteroid-corpus`
- Frame: rows are on the J2000 ecliptic, like the other de440-derived slices
  (regenerated after the fixed-ε₀ J2000 ecliptic reduction fix, commit 375b54d39; earlier rows were ecliptic-of-date
  and off by up to about 47″ in latitude).
- Default asteroid window: 1900–2100 CE (the corpus samples only this window;
  the kernel covers the full DE441 interval).

Usage / reproduction:

```bash
PLEIADES_DE_KERNEL=/path/to/de440.bsp \
PLEIADES_AST_KERNEL=/path/to/sb441-n373s.bsp \
PLEIADES_OBJECT_SPK_DIR=/path/to/objects \
  cargo test -p pleiades-jpl --test corpus_regen -- --nocapture
```

## Asteroid per-object SPKs (Tier A — pinned, kernel-absent bodies)

The 11 bodies absent from `sb441-n373s` were promoted to Tier A in slice 3
(2026-06-29) by sourcing each from its own pinned JPL Horizons SPK file
(`EPHEM_TYPE=SPK, COMMAND='DES=<7-digit-naif>;', START_TIME=1899-12-01,
STOP_TIME=2100-02-01`). Each file is pinned by SHA-256 and committed provenance
is in `crates/pleiades-jpl/src/spk/object_spk.rs` (`object_spk_manifest`).

The pins were refreshed on 2026-10-08 (issue #201), when all 11 files were
re-fetched with the same request to regenerate `asteroid_reference.csv` on the
J2000 ecliptic frame. Horizons SPKs are not byte-reproducible by re-fetch: each
file embeds its generation time, and Horizons serves the current orbit
solution (the refresh moved Apollo by up to 81 km, Amor 45 km, Icarus 15 km,
Toro 0.2 km; the other seven by under 1e-5 km). The pins identify the files
the committed corpus rows were generated from.

Verified coverage window for all 11: JD 2415020.5–2488069.5
(actual segment span 2414989.5–2488100.5, 0 gaps).

| Body | NAIF (7-digit) | SHA-256 | Class | Astrological usage |
| --- | --- | --- | --- | --- |
| 2060 Chiron | 2002060 | `92eb11beda1a809a0bf89ef3591aaa5ea63da0330f8b72675e9d341ad9e82e82` | Centaur | centaur astrology (Reinhart; von Heeren/Koch) — the wounded healer |
| 5145 Pholus | 2005145 | `da3d5addaa0aac7d5b21564bbda20dd71d1904aeb87eedbd98454e9f10a921ae` | Centaur | centaur astrology (Reinhart; von Heeren/Koch) |
| 7066 Nessus | 2007066 | `048b4d3b9f37101d81d9a374ff52c1afd1db1bf11215a3acaa5ef1d0d7c084fb` | Centaur | centaur astrology (Reinhart; von Heeren/Koch) |
| 10199 Chariklo | 2010199 | `3f3a41d7bfd57f9d13ec4c9a2a0c02f2002f076374e6f2e881575fd0fe172931` | Centaur | centaur astrology (Reinhart; von Heeren/Koch) |
| 8405 Asbolus | 2008405 | `bce77731972233a0b71bdc7e7174040f879e992f774be2a63dd65d4cc4cf88be` | Centaur | centaur astrology (Reinhart; von Heeren/Koch) |
| 1221 Amor | 2001221 | `039cd11324d48f0ddf0ef46090e8eea2207f765fb5d8f3797f6864f4d3d4d096` | MainBelt/NEA | asteroid astrology (Lang-Wescott; Demetra George) — love/compassion |
| 1181 Lilith | 2001181 | `7c86c230a4a037c4a7292e982cc431b7506dfda9facf086c2c129e4d46fdf06c` | MainBelt | Lang-Wescott — the catalogued numbered asteroid 1181, distinct from Black Moon Lilith |
| 944 Hidalgo | 2000944 | `40d6e5c13b182654a7c5dfd681133c0489c3ada6c1c5b5863b2ec6a1babdf575` | MainBelt | Lang-Wescott; Demetra George — advocacy/authority |
| 1566 Icarus | 2001566 | `266fbb4eb5521c469866c5f28078c522a21f50e0af704dfaa85ac925191f12dc` | MainBelt/NEA | Lang-Wescott — recklessness/risk |
| 1685 Toro | 2001685 | `2a4fc1d3550a5af57d36f1a9cdfa3b4cbbb93152ba0d5b9ee2fa537a743cfc61` | MainBelt/NEA | Lang-Wescott — force/power |
| 1862 Apollo | 2001862 | `c3b9cb7a0da9ac97b31f16c9c48e3a94f9559cbe5cf30cf5b21de529769f55ad` | MainBelt/NEA | asteroid astrology (Lang-Wescott; Demetra George) — ambition |

Regen recipe (all 36 Tier-A bodies, kernel + per-object SPKs):
```bash
PLEIADES_DE_KERNEL=.kernels/de440.bsp \
PLEIADES_AST_KERNEL=.kernels/sb441-n373s.bsp \
PLEIADES_OBJECT_SPK_DIR=.kernels/objects \
  cargo run -p pleiades-jpl --bin regenerate-asteroid-corpus
```
The `.bsp` files stay uncommitted; committed provenance is the SHA-256 table above
and `crates/pleiades-jpl/src/spk/object_spk.rs`.

## Asteroid slices (Tier B — Horizons-sourced, constrained)

The Horizons-sourced constrained asteroid slice (`asteroid_constrained.csv`) is
**now empty** (header-only). All 11 former Tier-B bodies (5 centaurs: Chiron,
Pholus, Nessus, Chariklo, Asbolus; 6 personal/minor/NEA: Amor, Lilith, Hidalgo,
Icarus, Toro, Apollo) were promoted to Tier A in slice 3 via per-object pinned
SPKs (see section above). All 9 TNOs were already promoted to Tier A in slice 2.

Historical note: bodies were originally generated once via JPL Horizons over
1900–2100 using `pleiades_jpl::ingest` (see the `horizons-fetch` feature) and
committed as the provenance-validated `asteroid_constrained` slice. The recipe
(`cargo run -p pleiades-jpl --features horizons-fetch --bin regenerate-asteroid-constrained`)
remains available but the slice no longer contains any bodies.

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

## Regenerating the packaged artifact

The packaged compressed artifact (`crates/pleiades-data/tests/fixtures/packaged-artifact.bin`)
is fit densely from two kernels over the shipped 1900-2100 window: the de440
planetary kernel (Sun, Moon, Mercury through Pluto) and the
`sb441-n373s` small-body kernel (Ceres, Pallas, Juno, Vesta and
`asteroid:433-Eros`, stored heliocentric like the planets). Both kernels are
required; the command refuses to write with either missing. Run it in release
mode:

```bash
PLEIADES_DE_KERNEL=/path/to/de440.bsp PLEIADES_AST_KERNEL=/path/to/sb441-n373s.bsp \
  cargo run --release -p pleiades-cli -- generate-packaged-artifact \
  --out crates/pleiades-data/tests/fixtures/packaged-artifact.bin
```

Then verify the result (save the previous artifact first, to compare against):

```bash
# Regeneration is deterministic: the committed bytes are reproduced exactly.
PLEIADES_DE_KERNEL=/path/to/de440.bsp PLEIADES_AST_KERNEL=/path/to/sb441-n373s.bsp \
  cargo test --release -p pleiades-data --test artifact_regen -- --nocapture

# The ten base bodies keep their segments bit-for-bit.
PLEIADES_BASELINE_ARTIFACT=/path/to/packaged-artifact.before.bin \
  cargo test --release -p pleiades-data --lib base_bodies_match_the_baseline_artifact -- --ignored --nocapture

# The encoded artifact stays within its size budget.
cargo test -q -p pleiades-data --lib encoded_artifact_within_size_budget
```

`pleiades-cli generate-artifact <de440.bsp> --asteroid-kernel <sb441-n373s.bsp> --out <path>`
does the same over a custom coverage window.
