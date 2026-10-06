# pleiades-events

Ephemeris event-finding for the `pleiades` astrology workspace: longitude crossings of the Sun, Moon, and planets, their ecliptic positions and speeds, their stations, and the exact moments of the aspects between them.

`EventEngine::position_at(body, frame, instant)` returns the full ecliptic
position in either `CrossingFrame`: longitude, latitude and distance, with
their speeds per day. Its longitude is exactly the one `longitude_at` returns,
so a position is consistent with the crossings found in the same frame.

- `GeocentricApparentOfDate`: the apparent place in the true ecliptic of date,
  with the same speed a `pleiades-core` apparent chart reports.
- `Heliocentric`: the geometric place from the Sun (no light-time, no
  aberration) in the true ecliptic and equinox of date — Swiss Ephemeris
  `SEFLG_HELCTR | SEFLG_TRUEPOS` — gated by `validate-helio-position`. Plain
  `SEFLG_HELCTR` output is retarded by the heliocentric light-time and differs
  by up to ≈ 41″ (Mercury). The Sun and Moon are an error in this frame.
- `GeocentricMeanOfDate`: the geometric place from the Earth's centre (no
  light-time, no aberration, no nutation) in the mean ecliptic and equinox of
  date. This is not the J2000 longitude a `pleiades-core` mean chart reports.

A speed channel is `None` when the backend reports no speed to derive it from.

Every crossing and position method takes a `CrossingFrame` (tropical zodiac) or
a `CrossingReference`, which adds a zodiac:

    let lahiri = CrossingReference::sidereal(
        CrossingFrame::GeocentricApparentOfDate,
        Ayanamsa::Lahiri,
    );
    let ingress = engine.next_longitude_crossing(
        CelestialBody::Sun,
        Longitude::from_degrees(0.0),
        lahiri,
        after,
    )?;

A sidereal longitude is the longitude on the mean equinox of date minus the
mean ayanamsa, the Swiss Ephemeris `SEFLG_SIDEREAL` convention: nutation does
not move a body through a sidereal zodiac. The ayanamsa is evaluated at every
trial instant of the search. The heliocentric frame takes a sidereal zodiac by
the same rule (Swiss Ephemeris `SEFLG_HELCTR | SEFLG_TRUEPOS | SEFLG_SIDEREAL`).
An ayanamsa without offset data is an error, never a silent tropical result. For
the star-anchored ayanamsa classes (`TrueStar` and `Galactic`) the mean
ayanamsa is used in every frame, whereas Swiss Ephemeris's apparent sidereal
positions add the anchoring star's annual aberration (up to about 20″) to the
ayanamsa. That is about 8 minutes of crossing time for the Sun and hours for a
slow planet such as Saturn, more near a station. True Citra and Galactic Center
were measured; the other ayanamsas in those classes follow from the same
mechanism and were not.
Mean-of-date and sidereal crossings, the heliocentric sidereal ones included,
are gated by `validate-crossings`.

## Civil times

Every event instant is TDB. A `Crossing`, `Station`, `AspectEvent` or
`RiseSet` gives its calendar date and clock time with `civil()`: UTC from
1972 on and UT1 before, to the millisecond, as `pleiades-time`'s
`civil_from_tdb` returns it.

    let ingress = engine.next_sun_crossing(Longitude::from_degrees(0.0), after)?.unwrap();
    let when = ingress.civil()?; // when.civil, when.scale

## Stations

`EventEngine::stations_in_range(body, reference, start, end)`,
`EventEngine::next_station(body, reference, after)` and
`EventEngine::previous_station(body, reference, before)` find the instants a
body's longitude speed changes sign. A `Station` carries the TDB instant, the
longitude there, and a `StationKind` (`TurnsRetrograde` or `TurnsDirect`).

A station is a sign change of the speed `position_at` reports in the same
frame and zodiac, so `position_at` just before and at a returned instant
always disagree in direction. A returned instant can be handed back to
`next_station`, which then returns the following station, or to
`previous_station`, which returns that same station (step back a second for
the one before it). The zodiac matters:
a sidereal speed is lower by the ayanamsa's rate, which moves a slow planet's
station by minutes to hours.

The Sun, the Moon, the mean node and every body in the heliocentric frame
never station and return nothing. A backend that reports no longitude speed is
`EventError::MissingSpeed`, never an empty list.

A station's instant is soft: near a station the speed changes slowly, so a
small speed difference between two ephemerides is a large time difference.
`validate-stations` compares the engine with the sign changes of Swiss
Ephemeris's own longitude speed (Moshier), 1900–2100 for the planets:

| Body | Largest time difference | Largest longitude difference |
|---|---|---|
| Mercury | 9.3 s | 0.381″ |
| Venus | 21.5 s | 0.542″ |
| Mars | 54.9 s | 1.119″ |
| Jupiter | 101.0 s | 0.695″ |
| Saturn | 165.1 s | 0.679″ |
| Uranus | 318.7 s | 0.505″ |
| Neptune | 495.4 s | 2.314″ |
| Pluto | 1292.7 s | 1.255″ |
| True node (1990–2030, separated stations) | 117467.6 s | 51.338″ |

These figures are for the packaged backend, which the gate runs on. On the
artifact-free VSOP87/ELP composite, Mercury's stations over 2000–2010 fall
within 0.33 s of the packaged backend's; a blocking test holds the year 2000
at 5 s.

For the true node the gate is a coarse existence-and-kind check, not a timing
check: it only confirms that each station at least 3 days from its neighbours
has a counterpart of the same kind within about three days. The node's speed
hovers near zero for days, so a station instant is ill-conditioned.

The search steps by 0.25 day for the Moon and the lunar points, 1 day for the
Sun, Mercury and Venus, and 2 days otherwise; two stations closer together
than the step are not reported. That happens only for the osculating lunar
points. The true node is retrograde on average and its speed touches zero
about every two weeks; whether a touch crosses zero for a few hours depends on
the ephemeris, so the gate compares only true-node stations at least 3 days
from their neighbours. Stations of asteroids, fictitious bodies and the
osculating apogee are found but not gated.

## Aspects

`EventEngine::aspects_in_range(first, second, angle, reference, start, end)`,
`EventEngine::next_aspect(first, second, angle, reference, after)` and
`EventEngine::previous_aspect(first, second, angle, reference, before)` find
the instants the ecliptic separation of two bodies equals an angle. An
`AspectEvent` carries the TDB instant and both longitudes.

The angle is an unsigned separation from 0° to 180°. An angle strictly
between the two is found on both sides: asking for 90° returns the moments
the first body is 90° ahead of the second and the moments it is 90° behind,
and the two longitudes say which. The moment a pair enters a 3° orb of a
square is the exact moment of the 87° or 93° separation. A returned instant
can be handed back to `next_aspect`, which then returns the following event.
`previous_aspect` returns the last event at or before an instant. Handed a
returned instant it gives that same event back, so step back a second to
reach the event before it. Each end of a range is decided by the separation
there, so two ranges that share an end hold each event exactly once.

A pair that approaches an angle and turns back before reaching it returns
nothing. A sidereal zodiac changes the reported longitudes, not the instants:
the ayanamsa cancels in the separation.

The search steps by the smaller of the two bodies' steps (0.25 day for the
Moon and the lunar points, 1 day for the Sun, Mercury and Venus, 2 days
otherwise) and splits each step where the separation turns, so two exact
moments inside one step, around a station, are both found. Two turning points
within two steps of each other may go unseen, and an event within two steps
of either end of the 1900–2100 window is not reported.

An aspect's instant is firm for a fast pair and soft for a slow pair near a
station, where the pair's relative speed is close to zero and a small
difference between two ephemerides is a large time difference.
`validate-aspects` compares the engine, event for event at 0°, 60°, 90°, 120°
and 180°, with the instants at which the difference of two Swiss Ephemeris
(Moshier) longitudes equals the angle, 1900–2100 for the planet pairs. The
separation difference is the time difference multiplied by the pair's
relative speed:

| Pair | Events | Largest separation difference | Largest time difference | Largest longitude difference |
|---|---|---|---|---|
| Sun–Moon (1990–2030) | 3959 | 2.894″ | 5.2 s | 0.394″ |
| Sun–Mercury | 1261 | 0.146″ | 2.4 s | 0.400″ |
| Mercury–Venus | 972 | 0.473″ | 101.7 s | 2.324″ |
| Venus–Mars | 1046 | 0.958″ | 44.5 s | 1.489″ |
| Mars–Jupiter | 842 | 2.211″ | 177.7 s | 0.801″ |
| Mars–Saturn | 872 | 1.114″ | 70.6 s | 1.075″ |
| Jupiter–Saturn | 210 | 1.119″ | 533.3 s | 2.231″ |
| Saturn–Pluto | 109 | 1.565″ | 744.8 s | 1.662″ |

The gate also covers Mercury–Venus and Mars–Saturn in the mean place and
Mars–Jupiter from the Sun. Aspects of the lunar points, asteroids and
fictitious bodies are found but not gated.
