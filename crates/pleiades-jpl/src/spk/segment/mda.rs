//! SPK Type 1 and Type 21 modified-difference-array decoders.
//!
//! Faithful port of the SPICELIB SPKE01/SPKE21 evaluation. Type 1 fixes the
//! difference-table dimension at 15; Type 21 stores it per segment as MAXDIM.

use super::super::bytes::Endian;
use super::super::daf::{addr_to_byte, SegmentDescriptor};
use super::super::{ReadAt, SpkError, SpkErrorKind};
use super::StateVector;

fn read_doubles<R: ReadAt + ?Sized>(
    src: &R,
    endian: Endian,
    addr: i32,
    n: usize,
) -> Result<Vec<f64>, SpkError> {
    let mut out = Vec::with_capacity(n);
    let base = addr_to_byte(addr);
    for i in 0..n {
        out.push(endian.f64_at(src, base + i * 8)?);
    }
    Ok(out)
}

/// Type 1: MAXDIM is fixed at 15 and not stored in the trailer.
pub fn evaluate_mda<R: ReadAt + ?Sized>(
    src: &R,
    endian: Endian,
    d: &SegmentDescriptor,
    et: f64,
    maxdim: usize,
) -> Result<StateVector, SpkError> {
    // Trailer: [...epoch table (N)][...directory (N/100)][N]. Last word = N.
    let numrec = read_doubles(src, endian, d.final_addr, 1)?[0] as usize;
    decode(src, endian, d, et, maxdim, numrec)
}

/// Type 21: MAXDIM stored just before NUMREC in the trailer.
pub fn evaluate_type21<R: ReadAt + ?Sized>(
    src: &R,
    endian: Endian,
    d: &SegmentDescriptor,
    et: f64,
) -> Result<StateVector, SpkError> {
    let tail = read_doubles(src, endian, d.final_addr - 1, 2)?; // [MAXDIM, NUMREC]
    let maxdim = tail[0] as usize;
    let numrec = tail[1] as usize;
    if maxdim == 0 || maxdim > 25 {
        return Err(SpkError::new(
            SpkErrorKind::Truncated,
            format!("bad MAXDIM {maxdim}"),
        ));
    }
    decode(src, endian, d, et, maxdim, numrec)
}

fn decode<R: ReadAt + ?Sized>(
    src: &R,
    endian: Endian,
    d: &SegmentDescriptor,
    et: f64,
    maxdim: usize,
    numrec: usize,
) -> Result<StateVector, SpkError> {
    if numrec == 0 {
        return Err(SpkError::new(SpkErrorKind::Truncated, "empty mda segment"));
    }
    let dlsize = 4 * maxdim + 11;
    // Epoch table begins right after the NUMREC records.
    let epoch_table_addr = d.init_addr + (numrec * dlsize) as i32;
    let epochs = read_doubles(src, endian, epoch_table_addr, numrec)?;

    // Find first record whose epoch >= et (linear scan; directory skip omitted
    // for the first cut — correct, just O(numrec) worst case).
    let mut recno = numrec - 1;
    for (i, &e) in epochs.iter().enumerate() {
        if et <= e {
            recno = i;
            break;
        }
    }
    let rec_addr = d.init_addr + (recno * dlsize) as i32;
    let rec = read_doubles(src, endian, rec_addr, dlsize)?;

    // Unpack record.
    let tl = rec[0];
    let g = &rec[1..1 + maxdim];
    let r = maxdim + 1;
    let refpos = [rec[r], rec[r + 2], rec[r + 4]];
    let refvel = [rec[r + 1], rec[r + 3], rec[r + 5]];
    let dt_base = maxdim + 7;
    // DT(maxdim, 3) column-major: component c, order j -> rec[dt_base + c*maxdim + j].
    let dt = |c: usize, j: usize| rec[dt_base + c * maxdim + j];
    let kqmax1 = rec[4 * maxdim + 7] as usize;
    let kq = [
        rec[4 * maxdim + 8] as usize,
        rec[4 * maxdim + 9] as usize,
        rec[4 * maxdim + 10] as usize,
    ];

    // --- SPKE01/SPKE21 interpolation ---
    let delta = et - tl;
    let mut fc = vec![0.0f64; maxdim + 1];
    let mut wc = vec![0.0f64; maxdim + 1];
    fc[0] = 1.0;
    let mut tp = delta;
    for j in 0..(kqmax1.saturating_sub(2)) {
        if g[j] == 0.0 {
            return Err(SpkError::new(
                SpkErrorKind::NumericalFailure,
                "zero stepsize",
            ));
        }
        fc[j + 1] = tp / g[j];
        wc[j] = delta / g[j];
        tp = delta + g[j];
    }

    // W(j) = 1/j initialisation.
    let mut w = vec![0.0f64; maxdim + 3];
    for (j, slot) in w.iter_mut().enumerate().take(kqmax1) {
        *slot = 1.0 / (j as f64 + 1.0);
    }

    let mut ks = kqmax1.saturating_sub(1);
    let mut jx = 0usize;
    // Position W recurrence.
    while ks >= 2 {
        jx += 1;
        let ks1 = ks - 1;
        for j in 0..jx {
            w[j + ks] = fc[j + 1] * w[j + ks1] - wc[j] * w[j + ks];
        }
        ks -= 1;
    }

    let mut position_km = [0.0f64; 3];
    for c in 0..3 {
        let mut sum = 0.0;
        for j in (0..kq[c]).rev() {
            sum += dt(c, j) * w[j + ks];
        }
        position_km[c] = refpos[c] + delta * (refvel[c] + delta * sum);
    }

    // Velocity: one more W update, then ks -= 1.
    jx += 1;
    let ks1 = ks - 1;
    for j in 0..jx {
        w[j + ks] = fc[j + 1] * w[j + ks1] - wc[j] * w[j + ks];
    }
    ks -= 1;
    let mut velocity_km_s = [0.0f64; 3];
    for c in 0..3 {
        let mut sum = 0.0;
        for j in (0..kq[c]).rev() {
            sum += dt(c, j) * w[j + ks];
        }
        velocity_km_s[c] = refvel[c] + delta * sum;
    }

    Ok(StateVector {
        position_km,
        velocity_km_s,
    })
}

#[cfg(test)]
mod tests {
    use crate::spk::daf::DafFile;
    use crate::spk::segment::evaluate;
    use crate::spk::test_support::{
        build_daf, type1_single_record_segment, type21_single_record_segment, SegmentSpec,
    };

    #[test]
    fn type1_zero_differences_is_linear_state() {
        // With zero DT, pos = refpos + delta*refvel, vel = refvel.
        let data = type1_single_record_segment(100.0, [10.0, 20.0, 30.0], [1.0, 2.0, 3.0], 1000.0);
        let blob = build_daf(&[SegmentSpec {
            start_et: 0.0,
            stop_et: 1000.0,
            target: 2099942,
            center: 10,
            frame: 1,
            data_type: 1,
            data,
            name: "AST1".to_string(),
        }]);
        let src: &[u8] = &blob;
        let daf = DafFile::parse(src).unwrap();
        let st = evaluate(src, daf.endian, &daf.segments[0], 105.0).unwrap();
        // delta = 105 - 100 = 5.
        assert!((st.position_km[0] - (10.0 + 5.0 * 1.0)).abs() < 1e-7);
        assert!((st.position_km[1] - (20.0 + 5.0 * 2.0)).abs() < 1e-7);
        assert!((st.velocity_km_s[2] - 3.0).abs() < 1e-7);
    }

    #[test]
    fn type21_zero_differences_is_linear_state() {
        let data =
            type21_single_record_segment(25, 100.0, [10.0, 20.0, 30.0], [1.0, 2.0, 3.0], 1000.0);
        let blob = build_daf(&[SegmentSpec {
            start_et: 0.0,
            stop_et: 1000.0,
            target: 20099942,
            center: 10,
            frame: 1,
            data_type: 21,
            data,
            name: "AST21".to_string(),
        }]);
        let src: &[u8] = &blob;
        let daf = DafFile::parse(src).unwrap();
        let st = evaluate(src, daf.endian, &daf.segments[0], 110.0).unwrap();
        assert!((st.position_km[0] - (10.0 + 10.0 * 1.0)).abs() < 1e-7);
        assert!((st.velocity_km_s[1] - 2.0).abs() < 1e-7);
    }
}
