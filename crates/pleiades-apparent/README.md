# pleiades-apparent

Apparent-place corrections for the `pleiades` workspace: light-time (whose
geocentric re-query carries annual aberration), precession-to-date, and
nutation-in-longitude,
referred to the true equinox of date, with typed correction provenance. Pure
math; the chart layer supplies positions, and callers can take the Sun's true
longitude of date for the aberration estimate from the backend-free Meeus Sun,
`sun_true_longitude_of_date_deg`.
