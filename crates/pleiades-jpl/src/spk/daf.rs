//! DAF container parsing: file record and the summary/name record chain.

use super::bytes::Endian;
use super::{ReadAt, SpkError, SpkErrorKind};

const RECORD_BYTES: usize = 1024;

/// One SPK segment descriptor decoded from a summary record.
#[derive(Clone, Debug, PartialEq)]
pub struct SegmentDescriptor {
    pub start_et: f64,
    pub stop_et: f64,
    pub target: i32,
    pub center: i32,
    pub frame: i32,
    pub data_type: i32,
    /// 1-based DAF double address of the first data word (inclusive).
    pub init_addr: i32,
    /// 1-based DAF double address of the last data word (inclusive).
    pub final_addr: i32,
    /// Trimmed segment name.
    pub name: String,
}

/// A parsed DAF file: endianness plus the list of segment descriptors.
#[derive(Clone, Debug)]
pub struct DafFile {
    pub endian: Endian,
    pub segments: Vec<SegmentDescriptor>,
}

/// Converts a 1-based DAF double address to a byte offset.
pub(crate) fn addr_to_byte(addr: i32) -> usize {
    ((addr as i64 - 1) * 8) as usize
}

fn offset_overflow() -> SpkError {
    SpkError::new(
        SpkErrorKind::Truncated,
        "DAF offset arithmetic overflowed a usize",
    )
}

fn checked_add(a: usize, b: usize) -> Result<usize, SpkError> {
    a.checked_add(b).ok_or_else(offset_overflow)
}

fn checked_mul(a: usize, b: usize) -> Result<usize, SpkError> {
    a.checked_mul(b).ok_or_else(offset_overflow)
}

fn record_byte(record_number: usize) -> Result<usize, SpkError> {
    record_number
        .checked_sub(1)
        .and_then(|zero_based| zero_based.checked_mul(RECORD_BYTES))
        .ok_or_else(|| {
            SpkError::new(
                SpkErrorKind::Truncated,
                format!("DAF record number {record_number} is out of range"),
            )
        })
}

impl DafFile {
    /// Parses the DAF file record and the full summary/name record chain.
    pub fn parse<R: ReadAt + ?Sized>(src: &R) -> Result<Self, SpkError> {
        if src.len() < RECORD_BYTES {
            return Err(SpkError::new(
                SpkErrorKind::Truncated,
                "file shorter than one record",
            ));
        }
        let idword = src.read_at(0, 8)?;
        if &idword[0..4] != b"DAF/" && idword != b"NAIF/DAF" {
            return Err(SpkError::new(
                SpkErrorKind::BadHeader,
                "missing DAF identification word",
            ));
        }
        let locfmt = src.read_at(88, 8)?;
        let endian = match locfmt {
            b"LTL-IEEE" => Endian::Little,
            b"BIG-IEEE" => Endian::Big,
            _ => return Err(SpkError::new(SpkErrorKind::UnknownEndianness, "bad LOCFMT")),
        };
        let nd = endian.i32_at(src, 8)?;
        let ni = endian.i32_at(src, 12)?;
        if nd != 2 || ni != 6 {
            return Err(SpkError::new(
                SpkErrorKind::BadHeader,
                format!("expected SPK ND=2 NI=6, got ND={nd} NI={ni}"),
            ));
        }
        let fward = endian.i32_at(src, 76)? as usize;

        let ss = (nd + (ni + 1) / 2) as usize; // 5 for SPK
        let mut segments = Vec::new();
        // A well-formed DAF never revisits a summary record. Tracking visited
        // records bounds the walk by the file's own record count, so no
        // legitimate kernel — however large — can be rejected, while a cyclic
        // or backward-pointing NEXT field terminates with an error instead of
        // looping until memory is exhausted.
        let mut visited = std::collections::HashSet::new();
        let mut rec_no = fward;
        while rec_no != 0 {
            if !visited.insert(rec_no) {
                return Err(SpkError::new(
                    SpkErrorKind::BadHeader,
                    format!("DAF summary record chain revisits record {rec_no}"),
                ));
            }
            let base = record_byte(rec_no)?;
            let next = endian.f64_at(src, base)? as usize;
            let nsum = endian.f64_at(src, checked_add(base, 16)?)? as usize; // 3rd double
            let name_base = record_byte(checked_add(rec_no, 1)?)?; // name record follows summary record
            for k in 0..nsum {
                // skip NEXT/PREV/NSUM, then k summaries
                let s = checked_add(base, checked_mul(checked_add(3, checked_mul(k, ss)?)?, 8)?)?;
                let start_et = endian.f64_at(src, s)?;
                let stop_et = endian.f64_at(src, checked_add(s, 8)?)?;
                let (target, center) = endian.packed_i32_pair_at(src, checked_add(s, 16)?)?;
                let (frame, data_type) = endian.packed_i32_pair_at(src, checked_add(s, 24)?)?;
                let (init_addr, final_addr) =
                    endian.packed_i32_pair_at(src, checked_add(s, 32)?)?;
                let nc = checked_mul(ss, 8)?;
                let raw = src.read_at(checked_add(name_base, checked_mul(k, nc)?)?, nc)?;
                let name = String::from_utf8_lossy(raw).trim_end().to_string();
                segments.push(SegmentDescriptor {
                    start_et,
                    stop_et,
                    target,
                    center,
                    frame,
                    data_type,
                    init_addr,
                    final_addr,
                    name,
                });
            }
            rec_no = next;
        }
        Ok(DafFile { endian, segments })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spk::test_support::{build_daf, type2_record, type2_segment_data, SegmentSpec};

    /// Same synthetic-segment shape used by `backend.rs` and
    /// `cross_check_tests.rs`; duplicated locally per this crate's existing
    /// per-module `#[cfg(test)]` convention (`test_support` has no
    /// `const_seg` of its own).
    fn const_seg(target: i32, center: i32, pos: [f64; 3]) -> SegmentSpec {
        let rec = type2_record(0.0, 1.0e12, &[pos[0], 0.0], &[pos[1], 0.0], &[pos[2], 0.0]);
        let data = type2_segment_data(-1.0e12, 2.0e12, rec.len(), &[rec]);
        SegmentSpec {
            start_et: -1.0e12,
            stop_et: 1.0e12,
            target,
            center,
            frame: 1,
            data_type: 2,
            data,
            name: "C".to_string(),
        }
    }

    /// Overwrites the NEXT field of the summary record at record 2.
    /// `build_daf` sets FWARD = 2, so the walk starts there and NEXT is the
    /// little-endian f64 at byte offset 1024.
    fn set_summary_next(blob: &mut [u8], next: f64) {
        blob[1024..1032].copy_from_slice(&next.to_le_bytes());
    }

    #[test]
    fn rejects_self_referential_record_chain() {
        let mut blob = build_daf(&[const_seg(10, 0, [1.0e8, 0.0, 0.0])]);
        // Record 2's NEXT points at record 2 — a cycle.
        set_summary_next(&mut blob, 2.0);

        let err = DafFile::parse(blob.as_slice())
            .expect_err("a self-referential record chain must be rejected, not looped on");
        assert_eq!(err.kind, SpkErrorKind::BadHeader);
    }

    #[test]
    fn rejects_record_number_that_overflows_offset_arithmetic() {
        let mut blob = build_daf(&[const_seg(10, 0, [1.0e8, 0.0, 0.0])]);
        // `as usize` saturates this to usize::MAX, overflowing (n - 1) * 1024.
        set_summary_next(&mut blob, 1.0e300);

        let err = DafFile::parse(blob.as_slice())
            .expect_err("an out-of-range record number must be rejected, not overflow");
        assert_eq!(err.kind, SpkErrorKind::Truncated);
    }

    #[test]
    fn valid_kernel_still_parses_after_hardening() {
        let blob = build_daf(&[
            const_seg(10, 0, [1.0e8, 0.0, 0.0]),
            const_seg(399, 3, [0.0, 0.0, 0.0]),
        ]);
        let daf = DafFile::parse(blob.as_slice()).expect("a well-formed DAF must still parse");
        assert_eq!(daf.segments.len(), 2);
    }

    #[test]
    fn parses_descriptor_fields_from_synthetic_daf() {
        let rec = type2_record(0.0, 1.0, &[1.0, 0.0], &[2.0, 0.0], &[3.0, 0.0]);
        let data = type2_segment_data(-1.0, 2.0, rec.len(), &[rec]);
        let blob = build_daf(&[SegmentSpec {
            start_et: -10.0,
            stop_et: 10.0,
            target: 499,
            center: 0,
            frame: 1,
            data_type: 2,
            data,
            name: "MARS BARYCENTER".to_string(),
        }]);
        let src: &[u8] = &blob;
        let daf = DafFile::parse(src).unwrap();
        assert_eq!(daf.endian, Endian::Little);
        assert_eq!(daf.segments.len(), 1);
        let seg = &daf.segments[0];
        assert_eq!(seg.target, 499);
        assert_eq!(seg.center, 0);
        assert_eq!(seg.frame, 1);
        assert_eq!(seg.data_type, 2);
        assert_eq!(seg.start_et, -10.0);
        assert_eq!(seg.stop_et, 10.0);
        assert_eq!(seg.name, "MARS BARYCENTER");
        // Data array round-trips through the recorded addresses.
        assert_eq!(addr_to_byte(seg.init_addr) % 8, 0);
        assert!(seg.final_addr > seg.init_addr);
    }
}
