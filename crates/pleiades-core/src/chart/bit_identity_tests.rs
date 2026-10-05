//! Pins every position and speed bit an apparent chart reports, so the
//! FU-25 query-shape changes (issue #128) can prove they move none of them.
//! Provenance is deliberately excluded: its aberration estimate moves by
//! design when the chart switches to the Meeus Sun.

use pleiades_apparent::fnv1a64;
use pleiades_backend::{Apparentness, CompositeBackend, EphemerisBackend, EphemerisRequest};
use pleiades_data::packaged_backend;
use pleiades_elp::ElpBackend;
use pleiades_types::{
    Ayanamsa, CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
    ZodiacMode,
};
use pleiades_vsop87::Vsop87Backend;

use crate::chart::{ChartEngine, ChartRequest, ChartSnapshot};

/// Value the checksum had on `main` at 91b13290d, before FU-25's second round,
/// was `0x29ba_dd07_d29e_6380`. Issue #140 re-pinned it: the VSOP87 and ELP
/// backends' shorter speed step moved the speed bits of the 44 composite rows
/// and no position bit.
///
/// Re-pin it only in a change that intentionally moves positions or speeds,
/// like the crossings golden: run `cargo test -p pleiades-core --lib apparent_chart_outputs_are_pinned`,
/// copy the `got 0x...` value from the failure message (it also prints every
/// pinned bit, for a diff against the old run), and state the re-pin and its
/// reason in the commit message. The value hashes `libm` output and is pinned
/// on Linux CI; another platform's libm may differ in the last bit.
const CHART_CHECKSUM: u64 = 0xa646_c0c4_0af4_0e91;

/// First instant the packaged artifact covers for Mars and the Moon, found by
/// bisecting `OutOfRangeInstant` on `position`. `nominal_range.start`
/// (JD 2378498.5) is wider than the packed coverage, so it cannot be used.
const PACKAGED_MARS_FIRST_JD: f64 = 2_415_020.5;

fn eleven_bodies() -> Vec<CelestialBody> {
    vec![
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mercury,
        CelestialBody::Venus,
        CelestialBody::Mars,
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
        CelestialBody::Uranus,
        CelestialBody::Neptune,
        CelestialBody::Pluto,
        CelestialBody::TrueNode,
    ]
}

fn tt(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tt)
}

fn push_bits(out: &mut String, value: Option<f64>) {
    match value {
        Some(v) => out.push_str(&format!("{:016x};", v.to_bits())),
        None => out.push_str("none;"),
    }
}

fn render(out: &mut String, label: &str, snapshot: &ChartSnapshot) {
    out.push_str(label);
    out.push('\n');
    for placement in &snapshot.placements {
        out.push_str(&format!(
            "{:?}:{:?}:",
            placement.body, placement.position.apparent
        ));
        let ecliptic = placement.position.ecliptic;
        push_bits(out, ecliptic.map(|e| e.longitude.degrees()));
        push_bits(out, ecliptic.map(|e| e.latitude.degrees()));
        push_bits(out, ecliptic.and_then(|e| e.distance_au));
        let motion = placement.position.motion;
        push_bits(out, motion.and_then(|m| m.longitude_deg_per_day));
        push_bits(out, motion.and_then(|m| m.latitude_deg_per_day));
        push_bits(out, motion.and_then(|m| m.distance_au_per_day));
        out.push('\n');
    }
}

fn chart<B: EphemerisBackend>(backend: B, request: &ChartRequest) -> ChartSnapshot {
    ChartEngine::new(backend).chart(request).expect("chart")
}

#[test]
fn apparent_chart_outputs_are_pinned() {
    let composite = || CompositeBackend::new(ElpBackend::new(), Vsop87Backend::new());
    let mut out = String::new();

    // Issue #128's epoch and two more spread over the composite's range.
    for jd in [2_460_763.5, 2_451_545.0, 2_433_282.5] {
        let request = ChartRequest::new(tt(jd))
            .with_bodies(eleven_bodies())
            .with_apparentness(Apparentness::Apparent);
        render(
            &mut out,
            &format!("composite {jd}"),
            &chart(composite(), &request),
        );
    }

    // Topocentric sidereal (Review Focus 5).
    let observer = ObserverLocation::new(
        Latitude::from_degrees(51.4779),
        Longitude::from_degrees(-0.0015),
        None,
    );
    let request = ChartRequest::new(tt(2_460_763.5))
        .with_bodies(eleven_bodies())
        .with_apparentness(Apparentness::Apparent)
        .with_observer(observer)
        .with_zodiac_mode(ZodiacMode::Sidereal {
            ayanamsa: Ayanamsa::Lahiri,
        })
        .with_topocentric(true);
    render(
        &mut out,
        "composite topo lahiri",
        &chart(composite(), &request),
    );

    // A bounded backend at its window start (Review Focus 4). An apparent
    // place needs the light-time-retarded instant, which lies before the
    // packed coverage at the very first covered instant, so that edge falls
    // back to the mean place (pinned as-is by the second row). A little later
    // (+0.05 day, past Mars's retardation) the place is apparent while the
    // earlier speed neighbour is still out of range, so the speed is
    // one-sided.
    let start_jd = PACKAGED_MARS_FIRST_JD + 0.05;
    let outside = packaged_backend().position(&EphemerisRequest::new(
        CelestialBody::Mars,
        tt(start_jd - 0.5),
    ));
    assert!(
        outside.is_err(),
        "the earlier speed neighbour must be out of range"
    );
    for (label, jd) in [
        ("packaged window start", start_jd),
        ("packaged first covered instant", PACKAGED_MARS_FIRST_JD),
    ] {
        let request = ChartRequest::new(tt(jd))
            .with_bodies(vec![CelestialBody::Mars, CelestialBody::Moon])
            .with_apparentness(Apparentness::Apparent);
        let snapshot = chart(packaged_backend(), &request);
        if jd == start_jd {
            for placement in &snapshot.placements {
                assert_eq!(placement.position.apparent, Apparentness::Apparent);
                let speed = placement
                    .position
                    .motion
                    .and_then(|m| m.longitude_deg_per_day);
                assert!(
                    speed.is_some(),
                    "{:?} has no longitude speed",
                    placement.body
                );
            }
        }
        render(&mut out, label, &snapshot);
    }

    let checksum = fnv1a64(&out);
    assert_eq!(
        checksum, CHART_CHECKSUM,
        "chart outputs moved: got {checksum:#018x}\n{out}"
    );
}
