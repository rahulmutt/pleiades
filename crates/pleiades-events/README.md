# pleiades-events

Ephemeris event-finding for the `pleiades` astrology workspace: longitude crossings of the Sun, Moon, and planets, their ecliptic positions and speeds, and their stations.

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
trial instant of the search. The heliocentric frame is tropical only, and an
ayanamsa without offset data is an error, never a silent tropical result. For
the star-anchored ayanamsa classes (`TrueStar` and `Galactic`) the mean
ayanamsa is used in every frame, whereas Swiss Ephemeris's apparent sidereal
positions add the anchoring star's annual aberration (up to about 20″) to the
ayanamsa. That is about 8 minutes of crossing time for the Sun and hours for a
slow planet such as Saturn, more near a station. True Citra and Galactic Center
were measured; the other ayanamsas in those classes follow from the same
mechanism and were not.
Mean-of-date and sidereal crossings are gated by `validate-crossings`.

## Stations

`EventEngine::stations_in_range(body, reference, start, end)` and
`EventEngine::next_station(body, reference, after)` find the instants a body's
longitude speed changes sign. A `Station` carries the TDB instant, the
longitude there, and a `StationKind` (`TurnsRetrograde` or `TurnsDirect`).

A station is a sign change of the speed `position_at` reports in the same
frame and zodiac, so `position_at` just before and at a returned instant
always disagree in direction. A returned instant can be handed back to
`next_station`, which then returns the following station. The zodiac matters:
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
