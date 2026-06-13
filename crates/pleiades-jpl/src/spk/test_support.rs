//! In-memory DAF/SPK byte-blob builder for deterministic decoder tests.
//!
//! Produces little-endian `DAF/SPK ` files with one summary record, one name
//! record, and contiguous segment data arrays. Only what the reader needs.

/// One segment to embed: descriptor doubles/ints plus its raw data doubles.
pub struct SegmentSpec {
    pub start_et: f64,
    pub stop_et: f64,
    pub target: i32,
    pub center: i32,
    pub frame: i32,
    pub data_type: i32,
    /// The segment's data array, as f64 doubles (trailer included by caller).
    pub data: Vec<f64>,
    /// 40-char name (space-padded/truncated by the writer).
    pub name: String,
}

const RECORD_BYTES: usize = 1024;

fn push_f64(buf: &mut Vec<u8>, v: f64) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn push_packed_ints(buf: &mut Vec<u8>, a: i32, b: i32) {
    buf.extend_from_slice(&a.to_le_bytes());
    buf.extend_from_slice(&b.to_le_bytes());
}

/// Builds a complete little-endian DAF/SPK byte blob from the given segments.
///
/// Layout: record 1 = file record; record 2 = summary record; record 3 = name
/// record; records 4.. = segment data arrays (each segment 8-byte aligned to a
/// record boundary for simplicity).
pub fn build_daf(segments: &[SegmentSpec]) -> Vec<u8> {
    // Segment data starts at record 4 (1-based). Compute each segment's
    // initial/final 1-based double addresses.
    let mut data_records: Vec<Vec<f64>> = Vec::new();
    let mut addresses: Vec<(i32, i32)> = Vec::new();
    let mut next_record = 4usize; // 1-based record number for next data block
    for seg in segments {
        let first_addr = ((next_record - 1) * 128 + 1) as i32; // 1-based double address
        let final_addr = first_addr + seg.data.len() as i32 - 1;
        addresses.push((first_addr, final_addr));
        // Pad this segment's data to a whole number of 128-double records.
        let mut block = seg.data.clone();
        let pad = (128 - (block.len() % 128)) % 128;
        block.extend(std::iter::repeat(0.0).take(pad));
        let records_used = block.len() / 128;
        data_records.push(block);
        next_record += records_used;
    }

    // ---- Record 1: file record ----
    let mut file_rec = Vec::with_capacity(RECORD_BYTES);
    file_rec.extend_from_slice(b"DAF/SPK "); // LOCIDW (8)
    file_rec.extend_from_slice(&2i32.to_le_bytes()); // ND
    file_rec.extend_from_slice(&6i32.to_le_bytes()); // NI
    file_rec.extend_from_slice(&[b' '; 60]); // LOCIFN
    file_rec.extend_from_slice(&2i32.to_le_bytes()); // FWARD (record 2)
    file_rec.extend_from_slice(&2i32.to_le_bytes()); // BWARD (record 2)
    let free_addr = ((next_record - 1) * 128 + 1) as i32;
    file_rec.extend_from_slice(&free_addr.to_le_bytes()); // FREE
    file_rec.extend_from_slice(b"LTL-IEEE"); // LOCFMT (8)
    file_rec.resize(RECORD_BYTES, 0); // PRENUL/FTPSTR/PSTNUL — zero-filled is fine for the reader

    // ---- Record 2: summary record ----
    let mut sum_rec: Vec<u8> = Vec::with_capacity(RECORD_BYTES);
    push_f64(&mut sum_rec, 0.0); // NEXT
    push_f64(&mut sum_rec, 0.0); // PREV
    push_f64(&mut sum_rec, segments.len() as f64); // NSUM
    for (seg, (init_addr, final_addr)) in segments.iter().zip(&addresses) {
        push_f64(&mut sum_rec, seg.start_et);
        push_f64(&mut sum_rec, seg.stop_et);
        push_packed_ints(&mut sum_rec, seg.target, seg.center);
        push_packed_ints(&mut sum_rec, seg.frame, seg.data_type);
        push_packed_ints(&mut sum_rec, *init_addr, *final_addr);
    }
    sum_rec.resize(RECORD_BYTES, 0);

    // ---- Record 3: name record ----
    let mut name_rec: Vec<u8> = Vec::with_capacity(RECORD_BYTES);
    for seg in segments {
        let mut name = seg.name.clone().into_bytes();
        name.resize(40, b' ');
        name_rec.extend_from_slice(&name[..40]);
    }
    name_rec.resize(RECORD_BYTES, 0);

    // ---- Assemble ----
    let mut out = Vec::new();
    out.extend_from_slice(&file_rec);
    out.extend_from_slice(&sum_rec);
    out.extend_from_slice(&name_rec);
    for block in data_records {
        for v in block {
            push_f64(&mut out, v);
        }
    }
    out
}

/// Builds a single Type 2 record's data: [MID, RADIUS, X.., Y.., Z..].
pub fn type2_record(mid: f64, radius: f64, x: &[f64], y: &[f64], z: &[f64]) -> Vec<f64> {
    let mut r = vec![mid, radius];
    r.extend_from_slice(x);
    r.extend_from_slice(y);
    r.extend_from_slice(z);
    r
}

/// Wraps Type 2 records with the trailer [INIT, INTLEN, RSIZE, N].
pub fn type2_segment_data(init: f64, intlen: f64, rsize: usize, records: &[Vec<f64>]) -> Vec<f64> {
    let mut data = Vec::new();
    for rec in records {
        assert_eq!(rec.len(), rsize, "record size mismatch");
        data.extend_from_slice(rec);
    }
    data.push(init);
    data.push(intlen);
    data.push(rsize as f64);
    data.push(records.len() as f64);
    data
}

/// Builds a Type 3 record: [MID, RADIUS, X.., Y.., Z.., dX.., dY.., dZ..].
pub fn type3_record(
    mid: f64,
    radius: f64,
    x: &[f64],
    y: &[f64],
    z: &[f64],
    dx: &[f64],
    dy: &[f64],
    dz: &[f64],
) -> Vec<f64> {
    let mut r = vec![mid, radius];
    for set in [x, y, z, dx, dy, dz] {
        r.extend_from_slice(set);
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_daf_is_record_aligned() {
        let rec = type2_record(0.0, 1.0, &[1.0, 0.0], &[2.0, 0.0], &[3.0, 0.0]);
        let data = type2_segment_data(-1.0, 2.0, rec.len(), &[rec]);
        let blob = build_daf(&[SegmentSpec {
            start_et: -1.0,
            stop_et: 1.0,
            target: 399,
            center: 0,
            frame: 1,
            data_type: 2,
            data,
            name: "TEST SEG".to_string(),
        }]);
        assert_eq!(blob.len() % 1024, 0);
        assert!(blob.len() >= 4 * 1024);
        assert_eq!(&blob[0..8], b"DAF/SPK ");
    }
}
