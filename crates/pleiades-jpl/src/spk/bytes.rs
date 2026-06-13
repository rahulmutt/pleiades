//! Endian-aware primitive reads over a [`ReadAt`] source.

use super::{ReadAt, SpkError, SpkErrorKind};

/// Byte order indicated by a DAF `LOCFMT` marker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endian {
    /// `LTL-IEEE`
    Little,
    /// `BIG-IEEE`
    Big,
}

impl Endian {
    /// Reads an `f64` at byte `offset`.
    pub fn f64_at<R: ReadAt + ?Sized>(self, src: &R, offset: usize) -> Result<f64, SpkError> {
        let b: [u8; 8] = src
            .read_at(offset, 8)?
            .try_into()
            .map_err(|_| SpkError::new(SpkErrorKind::Truncated, "f64 read"))?;
        Ok(match self {
            Endian::Little => f64::from_le_bytes(b),
            Endian::Big => f64::from_be_bytes(b),
        })
    }

    /// Reads an `i32` at byte `offset`.
    pub fn i32_at<R: ReadAt + ?Sized>(self, src: &R, offset: usize) -> Result<i32, SpkError> {
        let b: [u8; 4] = src
            .read_at(offset, 4)?
            .try_into()
            .map_err(|_| SpkError::new(SpkErrorKind::Truncated, "i32 read"))?;
        Ok(match self {
            Endian::Little => i32::from_le_bytes(b),
            Endian::Big => i32::from_be_bytes(b),
        })
    }

    /// Reads the two `i32` integers packed into the 8 bytes at `offset`
    /// (low half first under little-endian, high half second).
    pub fn packed_i32_pair_at<R: ReadAt + ?Sized>(self, src: &R, offset: usize) -> Result<(i32, i32), SpkError> {
        let first = self.i32_at(src, offset)?;
        let second = self.i32_at(src, offset + 4)?;
        Ok((first, second))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_le_and_be_doubles_and_packed_pairs() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1.5f64.to_le_bytes());
        bytes.extend_from_slice(&7i32.to_le_bytes());
        bytes.extend_from_slice(&(-3i32).to_le_bytes());
        let src: &[u8] = &bytes;

        assert_eq!(Endian::Little.f64_at(src, 0).unwrap(), 1.5);
        assert_eq!(Endian::Little.packed_i32_pair_at(src, 8).unwrap(), (7, -3));

        let be = 2.25f64.to_be_bytes();
        let be_src: &[u8] = &be;
        assert_eq!(Endian::Big.f64_at(be_src, 0).unwrap(), 2.25);
    }
}
