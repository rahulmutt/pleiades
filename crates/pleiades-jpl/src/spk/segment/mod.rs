//! SPK segment decoding: dispatch by data type to a state evaluator.

pub mod chebyshev;
pub mod mda;

#[cfg(test)]
mod hardening_tests;

use super::bytes::Endian;
use super::daf::{addr_to_byte, SegmentDescriptor};
use super::{ReadAt, SpkError, SpkErrorKind};

// Segment layouts are read from untrusted kernel bytes. The helpers below
// bound every count and address by the segment's own declared size and by the
// input's length, so a crafted kernel yields an `SpkError` instead of an
// overflow, a wrapped offset, or an allocation sized by a file value.

/// A `Truncated` error for a segment whose layout is inconsistent.
fn malformed(message: impl Into<String>) -> SpkError {
    SpkError::new(SpkErrorKind::Truncated, message)
}

/// Reads `n` consecutive doubles starting at 1-based DAF address `addr`.
///
/// The whole range is bounds-checked against the input before anything is
/// allocated.
fn read_doubles<R: ReadAt + ?Sized>(
    src: &R,
    endian: Endian,
    addr: i32,
    n: usize,
) -> Result<Vec<f64>, SpkError> {
    let byte_len = n
        .checked_mul(8)
        .ok_or_else(|| malformed(format!("read of {n} doubles overflowed a usize")))?;
    let bytes = src.read_at(addr_to_byte(addr)?, byte_len)?;
    bytes
        .as_chunks::<8>()
        .0
        .iter()
        .map(|chunk| endian.f64_at(chunk.as_slice(), 0))
        .collect()
}

/// Number of doubles in the segment's inclusive address range.
fn segment_doubles(d: &SegmentDescriptor) -> Result<usize, SpkError> {
    if d.init_addr < 1 || d.final_addr < d.init_addr {
        return Err(malformed(format!(
            "SPK segment address range {}..={} is invalid",
            d.init_addr, d.final_addr
        )));
    }
    // Both ends are positive i32 values, so the span fits in a usize.
    Ok((d.final_addr - d.init_addr) as usize + 1)
}

/// Address of the first of the segment's last `words` doubles.
fn trailer_addr(d: &SegmentDescriptor, words: usize) -> Result<i32, SpkError> {
    if segment_doubles(d)? < words {
        return Err(malformed(format!(
            "SPK segment is shorter than its {words}-word trailer"
        )));
    }
    // `words` <= the segment length, so this stays >= init_addr >= 1.
    Ok(d.final_addr - (words as i32 - 1))
}

/// The address `offset` doubles past `base`.
fn addr_plus(base: i32, offset: usize) -> Result<i32, SpkError> {
    i32::try_from(offset)
        .ok()
        .and_then(|offset| base.checked_add(offset))
        .ok_or_else(|| malformed(format!("SPK address {base} + {offset} overflowed")))
}

/// A record count or size stored as a double: finite, non-negative, whole.
fn count(value: f64, what: &str) -> Result<usize, SpkError> {
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 {
        return Err(malformed(format!(
            "SPK segment {what} {value} is not a count"
        )));
    }
    // Saturates above usize::MAX; callers bound the result by the segment size.
    Ok(value as usize)
}

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
