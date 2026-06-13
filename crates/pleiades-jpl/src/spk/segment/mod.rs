//! SPK segment decoding: dispatch by data type to a state evaluator.

pub mod chebyshev;
pub mod mda;

use super::bytes::Endian;
use super::daf::SegmentDescriptor;
use super::{ReadAt, SpkError, SpkErrorKind};

/// Position (km) and velocity (km/s) of a target relative to its center,
/// in the segment's reference frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StateVector {
    pub position_km: [f64; 3],
    pub velocity_km_s: [f64; 3],
}

/// Evaluates `descriptor`'s state at ephemeris time `et` (TDB sec past J2000).
pub fn evaluate<R: ReadAt + ?Sized>(
    src: &R,
    endian: Endian,
    descriptor: &SegmentDescriptor,
    et: f64,
) -> Result<StateVector, SpkError> {
    match descriptor.data_type {
        2 => chebyshev::evaluate_type2(src, endian, descriptor, et),
        3 => chebyshev::evaluate_type3(src, endian, descriptor, et),
        1 => mda::evaluate_mda(src, endian, descriptor, et, 15),
        21 => mda::evaluate_type21(src, endian, descriptor, et),
        other => Err(SpkError::new(
            SpkErrorKind::UnsupportedSegmentType,
            format!("SPK data type {other} is not supported"),
        )),
    }
}
