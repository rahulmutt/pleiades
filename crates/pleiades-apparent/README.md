# pleiades-apparent

Apparent-place corrections for the `pleiades` workspace: light-time (whose
geocentric re-query carries annual aberration), precession-to-date, and
nutation-in-longitude,
referred to the true equinox of date, with typed correction provenance. Pure
math; the chart layer supplies positions, and callers can take the Sun's true
longitude of date for the aberration estimate from the backend-free Meeus Sun,
`sun_true_longitude_of_date_deg`.

`gravitational_deflection` gives the Sun's light deflection of a direction (with Swiss Ephemeris's solar-disc taper), and `apparent_star_place` combines it with annual aberration to move a fixed star's mean place to its apparent place. The sidereal chart and event layers use it for the opt-in apparent-star ayanamsa.
