//! White-box checks of the speed assembly in [`super::motion`].

use super::{motion, sample};
use crate::crossings::CrossingFrame;
use crate::reference::CrossingReference;
use pleiades_data::packaged_backend;
use pleiades_types::CelestialBody;

#[test]
fn of_date_speed_exceeds_the_j2000_speed_by_the_precession_rate() {
    // General precession is 3.82e-5 deg/day; the nutation rate adds an
    // oscillation bounded by 6.0e-5 deg/day that averages out over a year.
    const PRECESSION_DEG_PER_DAY: f64 = 3.82e-5;
    let backend = packaged_backend();
    let body = CelestialBody::Neptune;
    let helio: CrossingReference = CrossingFrame::Heliocentric.into();
    let epochs = 366;
    let mut excesses = Vec::with_capacity(epochs);
    for day in 0..epochs {
        let jd = 2_451_545.0 + day as f64;
        let centre = sample(&backend, &body, &helio, jd).expect("sample");
        let base = centre
            .base_motion
            .and_then(|m| m.longitude_deg_per_day)
            .expect("J2000 base speed");
        let of_date = motion(&backend, &body, &helio, jd, &centre)
            .longitude_deg_per_day
            .expect("of-date speed");
        excesses.push(of_date - base);
    }
    let min = excesses.iter().copied().fold(f64::INFINITY, f64::min);
    let max = excesses.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mean = excesses.iter().sum::<f64>() / excesses.len() as f64;
    for (day, excess) in excesses.iter().enumerate() {
        assert!(
            (excess - PRECESSION_DEG_PER_DAY).abs() < 6.0e-5,
            "day {day}: excess {excess:e} (min {min:e}, max {max:e}, mean {mean:e})"
        );
    }
    assert!(
        (mean - PRECESSION_DEG_PER_DAY).abs() < 1.0e-5,
        "mean excess {mean:e} (min {min:e}, max {max:e})"
    );
}
