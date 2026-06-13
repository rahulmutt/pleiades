//! SPK Type 2 (position) and Type 3 (position+velocity) Chebyshev decoders.

use super::super::bytes::Endian;
use super::super::daf::{addr_to_byte, SegmentDescriptor};
use super::super::{ReadAt, SpkError, SpkErrorKind};
use super::StateVector;

/// Reads `n` consecutive doubles starting at 1-based DAF address `addr`.
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

/// Evaluates a Chebyshev series and its derivative at `s` in [-1, 1].
/// Returns (value, d value / d s).
fn cheb_eval(coeffs: &[f64], s: f64) -> (f64, f64) {
    let n = coeffs.len();
    if n == 0 {
        return (0.0, 0.0);
    }
    // Clenshaw for value; derivative via the standard T'_k recurrence.
    let t = [1.0, s];
    let dt = [0.0, 1.0];
    let mut value = coeffs[0] * t[0];
    let mut deriv = coeffs[0] * dt[0];
    if n > 1 {
        value += coeffs[1] * t[1];
        deriv += coeffs[1] * dt[1];
    }
    let mut tkm2 = t[0];
    let mut tkm1 = t[1];
    let mut dkm2 = dt[0];
    let mut dkm1 = dt[1];
    for k in 2..n {
        let tk = 2.0 * s * tkm1 - tkm2;
        let dk = 2.0 * tkm1 + 2.0 * s * dkm1 - dkm2;
        value += coeffs[k] * tk;
        deriv += coeffs[k] * dk;
        tkm2 = tkm1;
        tkm1 = tk;
        dkm2 = dkm1;
        dkm1 = dk;
    }
    (value, deriv)
}

struct Trailer {
    init: f64,
    intlen: f64,
    rsize: usize,
    n: usize,
}

fn read_trailer<R: ReadAt + ?Sized>(
    src: &R,
    endian: Endian,
    d: &SegmentDescriptor,
) -> Result<Trailer, SpkError> {
    // Last 4 doubles: INIT, INTLEN, RSIZE, N at final_addr-3 .. final_addr.
    let t = read_doubles(src, endian, d.final_addr - 3, 4)?;
    Ok(Trailer { init: t[0], intlen: t[1], rsize: t[2] as usize, n: t[3] as usize })
}

fn select_record(tr: &Trailer, et: f64) -> usize {
    if tr.intlen <= 0.0 {
        return 0;
    }
    let idx = ((et - tr.init) / tr.intlen).floor();
    (idx.max(0.0) as usize).min(tr.n.saturating_sub(1))
}

/// SPK Type 2: position from Chebyshev, velocity by analytic differentiation.
pub fn evaluate_type2<R: ReadAt + ?Sized>(
    src: &R,
    endian: Endian,
    d: &SegmentDescriptor,
    et: f64,
) -> Result<StateVector, SpkError> {
    evaluate_chebyshev(src, endian, d, et, 3)
}

/// SPK Type 3: position and velocity each from their own Chebyshev sets.
pub fn evaluate_type3<R: ReadAt + ?Sized>(
    src: &R,
    endian: Endian,
    d: &SegmentDescriptor,
    et: f64,
) -> Result<StateVector, SpkError> {
    evaluate_chebyshev(src, endian, d, et, 6)
}

fn evaluate_chebyshev<R: ReadAt + ?Sized>(
    src: &R,
    endian: Endian,
    d: &SegmentDescriptor,
    et: f64,
    sets: usize,
) -> Result<StateVector, SpkError> {
    let tr = read_trailer(src, endian, d)?;
    if tr.n == 0 || tr.rsize < 2 {
        return Err(SpkError::new(SpkErrorKind::Truncated, "empty chebyshev segment"));
    }
    let recno = select_record(&tr, et);
    let rec_addr = d.init_addr + (recno * tr.rsize) as i32;
    let rec = read_doubles(src, endian, rec_addr, tr.rsize)?;
    let mid = rec[0];
    let radius = rec[1];
    if radius == 0.0 {
        return Err(SpkError::new(SpkErrorKind::Truncated, "zero record radius"));
    }
    let s = (et - mid) / radius;
    let per = (tr.rsize - 2) / sets; // coeffs per component
    let coeff = |set: usize| &rec[2 + set * per..2 + (set + 1) * per];

    let mut position_km = [0.0; 3];
    let mut velocity_km_s = [0.0; 3];
    for axis in 0..3 {
        let (p, dp_ds) = cheb_eval(coeff(axis), s);
        position_km[axis] = p;
        if sets == 3 {
            // ds/dt = 1/radius; velocity in km/s.
            velocity_km_s[axis] = dp_ds / radius;
        }
    }
    if sets == 6 {
        for axis in 0..3 {
            let (v, _) = cheb_eval(coeff(3 + axis), s);
            velocity_km_s[axis] = v;
        }
    }
    Ok(StateVector { position_km, velocity_km_s })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spk::daf::DafFile;
    use crate::spk::segment::evaluate;
    use crate::spk::test_support::{build_daf, type2_record, type2_segment_data, SegmentSpec};

    #[test]
    fn type2_constant_position_matches_coefficients() {
        // Single record, degree-1 (2 coeffs/axis). Constant term only -> position
        // equals the c0 coefficient (T0 = 1), independent of s.
        let rec = type2_record(0.0, 100.0, &[11.0, 0.0], &[22.0, 0.0], &[33.0, 0.0]);
        let data = type2_segment_data(-100.0, 200.0, rec.len(), &[rec]);
        let blob = build_daf(&[SegmentSpec {
            start_et: -100.0,
            stop_et: 100.0,
            target: 499,
            center: 0,
            frame: 1,
            data_type: 2,
            data,
            name: "T2".to_string(),
        }]);
        let src: &[u8] = &blob;
        let daf = DafFile::parse(src).unwrap();
        let st = evaluate(src, daf.endian, &daf.segments[0], 25.0).unwrap();
        assert!((st.position_km[0] - 11.0).abs() < 1e-9);
        assert!((st.position_km[1] - 22.0).abs() < 1e-9);
        assert!((st.position_km[2] - 33.0).abs() < 1e-9);
        // Linear term zero -> velocity zero.
        assert!(st.velocity_km_s[0].abs() < 1e-9);
    }

    #[test]
    fn type2_linear_term_produces_velocity() {
        // c1 = 5 on X: value = c0 + 5*s, ds/dt = 1/radius -> v = 5/radius.
        let radius = 10.0;
        let rec = type2_record(0.0, radius, &[1.0, 5.0], &[0.0, 0.0], &[0.0, 0.0]);
        let data = type2_segment_data(-10.0, 20.0, rec.len(), &[rec]);
        let blob = build_daf(&[SegmentSpec {
            start_et: -10.0,
            stop_et: 10.0,
            target: 499,
            center: 0,
            frame: 1,
            data_type: 2,
            data,
            name: "T2".to_string(),
        }]);
        let src: &[u8] = &blob;
        let daf = DafFile::parse(src).unwrap();
        let st = evaluate(src, daf.endian, &daf.segments[0], 0.0).unwrap();
        assert!((st.position_km[0] - 1.0).abs() < 1e-9);
        assert!((st.velocity_km_s[0] - 0.5).abs() < 1e-9);
    }
}
