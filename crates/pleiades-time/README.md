# pleiades-time

Civil-time conversion for the `pleiades` workspace: Gregorian calendar to Julian
Day, leap seconds, Delta-T, and TT/TDB with typed conversion provenance.

- `to_terrestrial` converts a civil UTC or UT1 datetime to a TT or TDB
  `Instant`.
- `from_terrestrial` converts a TT or TDB `Instant` back to a civil UTC or UT1
  datetime, rounded to the millisecond. A UTC result inside an inserted leap
  second is returned as `23:59:60.x`.

Both cover 1900–2100 and report the same `ConversionProvenance`: `exact` for
UTC inside the leap-second table (from 1972), `observed` or `predicted` where
the Delta-T model is used. A round trip returns the starting datetime to
within 1 ms.
