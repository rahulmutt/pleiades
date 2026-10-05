//! `position_without_motion` returns `position`'s place bit for bit and no
//! motion (issue #128: the light-time re-queries discard the motion, which
//! cost two of every three series evaluations).

use pleiades_backend::{EphemerisBackend, EphemerisRequest, EphemerisResult};
use pleiades_types::{CoordinateFrame, Instant, JulianDay, TimeScale};

use crate::Vsop87Backend;

#[test]
fn position_without_motion_is_position_minus_motion() {
    let backend = Vsop87Backend::new();
    // Pluto at 1885.0 / 2099.9 crosses its fit-window edges.
    for jd in [
        2_460_763.5,
        2_451_545.0,
        2_309_103.5,
        2_488_069.5,
        2_381_000.5,
    ] {
        for body in Vsop87Backend::supported_bodies() {
            for frame in [CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial] {
                let mut req = EphemerisRequest::new(
                    body.clone(),
                    Instant::new(JulianDay::from_days(jd), TimeScale::Tt),
                );
                req.frame = frame;
                let full = backend.position(&req).expect("position");
                let free = backend.position_without_motion(&req).expect("motion-free");
                assert!(full.motion.is_some(), "{body:?} {jd}");
                assert_eq!(
                    EphemerisResult {
                        motion: None,
                        ..full
                    },
                    free,
                    "{body:?} {jd} {frame:?}"
                );
            }
        }
    }
}
