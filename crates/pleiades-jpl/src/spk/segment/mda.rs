//! SPK Type 1 / Type 21 modified-difference-array decoders (filled in Task 7).
use super::super::bytes::Endian;
use super::super::daf::SegmentDescriptor;
use super::super::{ReadAt, SpkError, SpkErrorKind};
use super::StateVector;

pub fn evaluate_mda<R: ReadAt + ?Sized>(
    _src: &R,
    _endian: Endian,
    _d: &SegmentDescriptor,
    _et: f64,
    _maxdim: usize,
) -> Result<StateVector, SpkError> {
    Err(SpkError::new(SpkErrorKind::UnsupportedSegmentType, "mda not yet implemented"))
}

pub fn evaluate_type21<R: ReadAt + ?Sized>(
    _src: &R,
    _endian: Endian,
    _d: &SegmentDescriptor,
    _et: f64,
) -> Result<StateVector, SpkError> {
    Err(SpkError::new(SpkErrorKind::UnsupportedSegmentType, "type 21 not yet implemented"))
}
