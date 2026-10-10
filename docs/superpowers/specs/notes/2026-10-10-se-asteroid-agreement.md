# Swiss Ephemeris seas_18 vs sb441 asteroid agreement

Date: 2026-10-10. Swiss Ephemeris version (`swe_version`): 2.10.03 (libswisseph-sys 0.1.2).

Spike comparing SE (SWIEPH, `seas_18`) geocentric J2000 ecliptic asteroid positions with the 407 rows per body of
`crates/pleiades-jpl/data/corpus/asteroid_reference.csv` (JPL sb441-n373s, header: "geocentric ecliptic (mean geometric), TDB").
The probe asserted `ret & 2` on every call, so SE used the files rather than falling back to Moshier.

## File digests (SHA-256)

- `seas_18.se1`: a2cd8fc33807c78ca9a700c91c2e042258b12fc4796519e00781440b5ad8b2e2
- `sepl_18.se1`: ca1393ceab3a44fbc895887cf789c68819ae6a1cbc9b22225872dbe4ccd99a66 (matches the existing nod-aps pin)
- `semo_18.se1`: 1ca07bd67c24374d77226180c20a4f9996cba013697894810518e7eb582ca4f7 (matches the existing nod-aps pin)

## Run 1: geometric convention (matches the CSV)

`IFLAG` = SEFLG_SWIEPH | TRUEPOS | J2000 | NONUT | NOGDEFL | NOABERR = 2|16|32|64|512|1024 = 1650.

```
Ceres: 407 rows, max |dlon*cos(lat)| 1.3961", max |dlat| 0.2367"
Pallas: 407 rows, max |dlon*cos(lat)| 1.4366", max |dlat| 0.7822"
Juno: 407 rows, max |dlon*cos(lat)| 0.7092", max |dlat| 0.4556"
Vesta: 407 rows, max |dlon*cos(lat)| 2.2584", max |dlat| 0.5993"
```

## Run 2: light-time corrected (TRUEPOS cleared), for contrast

`IFLAG` = 1634. Not the CSV's convention; differences are 14-17" in longitude, confirming the CSV is geometric.

```
Ceres: 407 rows, max |dlon*cos(lat)| 14.5009", max |dlat| 2.8248"
Pallas: 407 rows, max |dlon*cos(lat)| 16.4690", max |dlat| 10.8227"
Juno: 407 rows, max |dlon*cos(lat)| 16.6198", max |dlat| 4.5461"
Vesta: 407 rows, max |dlon*cos(lat)| 16.2861", max |dlat| 2.2553"
```

## Conclusion

Under the matching (geometric) convention every body is within 5".
Floor for asteroid angle ceilings: 2.2584" (Vesta, longitude).
