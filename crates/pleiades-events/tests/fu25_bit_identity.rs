//! Pins every longitude, latitude, distance and speed bit the event engine
//! reports on the algorithmic backends, so the FU-25 query-shape changes
//! (issue #128) can prove they move none of them.

use pleiades_apparent::fnv1a64;
use pleiades_backend::CompositeBackend;
use pleiades_elp::ElpBackend;
use pleiades_events::{CrossingFrame, EventEngine};
use pleiades_fict::FictitiousBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
use pleiades_vsop87::Vsop87Backend;

/// Value the checksum had on `main` at 91b13290d, before FU-25's second round.
const EVENTS_CHECKSUM: u64 = 0x0dfc_fbba_b887_e99b;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn bits(out: &mut String, value: Option<f64>) {
    match value {
        Some(v) => out.push_str(&format!("{:016x};", v.to_bits())),
        None => out.push_str("none;"),
    }
}

#[test]
fn event_engine_outputs_are_pinned() {
    let mut out = String::new();
    let engine = EventEngine::new(CompositeBackend::new(
        ElpBackend::new(),
        Vsop87Backend::new(),
    ));
    let bodies = [
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mercury,
        CelestialBody::Mars,
        CelestialBody::Saturn,
        CelestialBody::Pluto,
        CelestialBody::TrueNode,
    ];
    let frames = [
        CrossingFrame::GeocentricApparentOfDate,
        CrossingFrame::GeocentricMeanOfDate,
        CrossingFrame::Heliocentric,
    ];
    for jd in [2_460_763.5, 2_451_545.0, 2_433_282.5] {
        for body in &bodies {
            for frame in frames {
                out.push_str(&format!("{jd}:{body:?}:{frame:?}:"));
                match engine.longitude_at(body.clone(), frame, tdb(jd)) {
                    Ok(lon) => bits(&mut out, Some(lon.degrees())),
                    Err(error) => out.push_str(&format!("err({error});")),
                }
                match engine.position_at(body.clone(), frame, tdb(jd)) {
                    Ok(p) => {
                        bits(&mut out, Some(p.ecliptic.longitude.degrees()));
                        bits(&mut out, Some(p.ecliptic.latitude.degrees()));
                        bits(&mut out, p.ecliptic.distance_au);
                        bits(&mut out, p.motion.longitude_deg_per_day);
                        bits(&mut out, p.motion.latitude_deg_per_day);
                        bits(&mut out, p.motion.distance_au_per_day);
                    }
                    Err(error) => out.push_str(&format!("err({error});")),
                }
                out.push('\n');
            }
        }
    }

    // A fictitious body over the VSOP87 Sun source (Task 3 changes how that
    // source is read).
    let fict = EventEngine::new(FictitiousBackend::new(Vsop87Backend::new()));
    for jd in [2_460_763.5, 2_451_545.0] {
        let p = fict
            .position_at(
                CelestialBody::Cupido,
                CrossingFrame::GeocentricApparentOfDate,
                tdb(jd),
            )
            .expect("Cupido");
        out.push_str(&format!("fict {jd}:"));
        bits(&mut out, Some(p.ecliptic.longitude.degrees()));
        bits(&mut out, Some(p.ecliptic.latitude.degrees()));
        bits(&mut out, p.ecliptic.distance_au);
        bits(&mut out, p.motion.longitude_deg_per_day);
        out.push('\n');
    }

    let checksum = fnv1a64(&out);
    assert_eq!(
        checksum, EVENTS_CHECKSUM,
        "event outputs moved: got {checksum:#018x}\n{out}"
    );
}
