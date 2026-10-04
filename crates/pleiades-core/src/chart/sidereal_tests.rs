//! Sidereal placements of an apparent chart (issue #120).
//!
//! A sidereal longitude is the longitude on the **mean** equinox of date minus
//! the mean ayanamsa: the Swiss Ephemeris `SEFLG_SIDEREAL` convention, and the
//! one `pleiades-events` reads crossings in. An apparent longitude is on the
//! true equinox, so nutation in longitude (Δψ, up to about 17″) must come off
//! before the ayanamsa. Before this fix the chart subtracted the ayanamsa from
//! the true-equinox longitude and every sidereal placement carried Δψ.

use pleiades_apparent::nutation::nutation;
use pleiades_ayanamsa::sidereal_offset;
use pleiades_backend::{CompositeBackend, ZodiacMode};
use pleiades_elp::ElpBackend;
use pleiades_types::{Ayanamsa, CelestialBody, Instant, JulianDay, TimeScale};
use pleiades_vsop87::Vsop87Backend;

use crate::chart::{ChartEngine, ChartRequest, ChartSnapshot};

/// 2000-01-01 12:00 UTC as TT, the instant of the issue's reproduction.
const ISSUE_120_JD_TT: f64 = 2_451_545.000_739;

/// Swiss Ephemeris 2.10 (Moshier) `SEFLG_SIDEREAL` Lahiri longitudes at
/// [`ISSUE_120_JD_TT`], quoted in issue #120.
const SWISS_EPHEMERIS_LAHIRI_DEG: [(CelestialBody, f64); 4] = [
    (CelestialBody::Sun, 256.515_697),
    (CelestialBody::Moon, 199.470_553),
    (CelestialBody::Mars, 304.110_091),
    (CelestialBody::Saturn, 16.542_416),
];

fn composite_backend() -> CompositeBackend<Vsop87Backend, ElpBackend> {
    CompositeBackend::new(Vsop87Backend::new(), ElpBackend::new())
}

fn issue_instant() -> Instant {
    Instant::new(JulianDay::from_days(ISSUE_120_JD_TT), TimeScale::Tt)
}

fn lahiri() -> ZodiacMode {
    ZodiacMode::Sidereal {
        ayanamsa: Ayanamsa::Lahiri,
    }
}

fn bodies() -> Vec<CelestialBody> {
    SWISS_EPHEMERIS_LAHIRI_DEG
        .iter()
        .map(|(body, _)| body.clone())
        .collect()
}

fn longitude_deg(snapshot: &ChartSnapshot, body: &CelestialBody) -> f64 {
    snapshot
        .placement_for(body)
        .expect("body is placed")
        .position
        .ecliptic
        .expect("ecliptic coordinates")
        .longitude
        .degrees()
}

fn wrap_arcsec(deg: f64) -> f64 {
    ((deg + 180.0).rem_euclid(360.0) - 180.0) * 3600.0
}

#[test]
fn issue_120_sidereal_apparent_chart_matches_swiss_ephemeris_lahiri() {
    // The issue measured every chart placement 13.7″ below Swiss Ephemeris,
    // which is Δψ at the instant (−13.92″), while the events engine agreed
    // with Swiss Ephemeris to 0.1-0.7″.
    let snapshot = ChartEngine::new(composite_backend())
        .chart(
            &ChartRequest::new(issue_instant())
                .with_bodies(bodies())
                .with_zodiac_mode(lahiri()),
        )
        .expect("sidereal apparent chart succeeds");
    for (body, expected) in &SWISS_EPHEMERIS_LAHIRI_DEG {
        let residual = wrap_arcsec(longitude_deg(&snapshot, body) - expected);
        assert!(
            residual.abs() < 1.0,
            "{body:?} sidereal longitude is {residual:+.3}\" from Swiss Ephemeris Lahiri"
        );
    }
}

#[test]
fn sidereal_apparent_shift_is_the_ayanamsa_plus_nutation_in_longitude() {
    // Independent of any reference ephemeris: tropical apparent minus sidereal
    // apparent must be the mean ayanamsa plus Δψ, not the ayanamsa alone.
    let engine = ChartEngine::new(composite_backend());
    let tropical = engine
        .chart(&ChartRequest::new(issue_instant()).with_bodies(bodies()))
        .expect("tropical chart succeeds");
    let sidereal = engine
        .chart(
            &ChartRequest::new(issue_instant())
                .with_bodies(bodies())
                .with_zodiac_mode(lahiri()),
        )
        .expect("sidereal chart succeeds");
    let ayanamsa_deg = sidereal_offset(&Ayanamsa::Lahiri, issue_instant())
        .expect("Lahiri offset")
        .degrees();
    let delta_psi_deg = nutation(ISSUE_120_JD_TT)
        .expect("nutation")
        .delta_psi_arcsec
        / 3600.0;
    for body in bodies() {
        let shift = longitude_deg(&tropical, &body) - longitude_deg(&sidereal, &body);
        let residual = wrap_arcsec(shift - (ayanamsa_deg + delta_psi_deg));
        assert!(
            residual.abs() < 1e-6,
            "{body:?}: tropical − sidereal differs from ayanamsa + Δψ by {residual:+.6}\""
        );
    }
}
