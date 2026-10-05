# pleiades-time

Civil-time conversion for the `pleiades` workspace: Gregorian calendar to Julian
Day, leap seconds, Delta-T, and TT/TDB with typed conversion provenance.

- `to_terrestrial` converts a civil UTC or UT1 datetime to a TT or TDB
  `Instant`.
- `from_terrestrial` converts a TT or TDB `Instant` back to a civil UTC or UT1
  datetime, rounded to the millisecond. A UTC result inside an inserted leap
  second is returned as `23:59:60.x`.
- `civil_from_tt` and `civil_from_tdb` pick the scale for you: UTC from
  1972-01-01 on, UT1 before, with `CivilConversion::scale` reporting which.
  Use them for event instants, which can fall anywhere in the window.

Both cover 1900–2100 and report the same `ConversionProvenance`: `exact` for
UTC inside the leap-second table (from 1972), `observed` or `predicted` where
the Delta-T model is used. An instant converted to civil time and back returns
to within 1 ms. A datetime converted to an instant and back also returns to
within 1 ms, except UT1 in the 0.216 s before the 2020-01-01 Delta-T node
(which comes back as the post-node datetime) and the last 0.5 ms of 2100
(which rounds out of range).
