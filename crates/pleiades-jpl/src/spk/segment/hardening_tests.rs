//! Regression tests for segment evaluation over untrusted kernel bytes (#29).
//!
//! Every malformed input must come back as an `SpkError`, never a panic, an
//! arithmetic overflow, or an allocation sized by an unchecked file value.

use super::evaluate;
use crate::spk::daf::{DafFile, SegmentDescriptor};
use crate::spk::test_support::{
    build_daf, type21_single_record_segment, type2_record, type2_segment_data, type3_record,
    SegmentSpec,
};
use crate::spk::{SpkError, SpkErrorKind};

/// A valid single-record Type 2 segment's data array (trailer included).
fn type2_data() -> Vec<f64> {
    let rec = type2_record(0.0, 100.0, &[11.0, 0.0], &[22.0, 0.0], &[33.0, 0.0]);
    type2_segment_data(-100.0, 200.0, rec.len(), &[rec])
}

/// A valid single-record Type 3 segment's data array (trailer included).
fn type3_data() -> Vec<f64> {
    let c = [1.0, 0.0];
    let rec = type3_record(0.0, 100.0, &c, &c, &c, &c, &c, &c);
    type2_segment_data(-100.0, 200.0, rec.len(), &[rec])
}

fn spec(data_type: i32, data: Vec<f64>) -> SegmentSpec {
    SegmentSpec {
        start_et: -100.0,
        stop_et: 100.0,
        target: 499,
        center: 0,
        frame: 1,
        data_type,
        data,
        name: "HARDENING".to_string(),
    }
}

/// Builds a one-segment kernel and evaluates it at `et`, with `edit` applied
/// to the parsed descriptor first (to model a crafted summary record).
fn eval_with(
    data_type: i32,
    data: Vec<f64>,
    et: f64,
    edit: impl FnOnce(&mut SegmentDescriptor),
) -> Result<(), SpkError> {
    let blob = build_daf(&[spec(data_type, data)]);
    let src: &[u8] = &blob;
    let daf = DafFile::parse(src).expect("fixture kernel parses");
    let mut descriptor = daf.segments[0].clone();
    edit(&mut descriptor);
    evaluate(src, daf.endian, &descriptor, et).map(|_| ())
}

/// Sets Type 2/3 trailer word `index` (0 INIT, 1 INTLEN, 2 RSIZE, 3 N).
fn with_trailer_word(mut data: Vec<f64>, index: usize, value: f64) -> Vec<f64> {
    let at = data.len() - 4 + index;
    data[at] = value;
    data
}

/// A named edit that models a crafted summary-record field.
type DescriptorEdit = (&'static str, fn(&mut SegmentDescriptor));

fn expect_truncated(result: Result<(), SpkError>, what: &str) {
    let err = result.expect_err(what);
    assert_eq!(err.kind, SpkErrorKind::Truncated, "{what}: {}", err.message);
}

#[test]
fn chebyshev_rejects_huge_rsize_before_allocating() {
    for rsize in [1.0e15, 1.0e19, f64::INFINITY, f64::MAX] {
        let data = with_trailer_word(type2_data(), 2, rsize);
        expect_truncated(
            eval_with(2, data, 0.0, |_| {}),
            &format!("RSIZE {rsize} must be rejected"),
        );
    }
}

#[test]
fn chebyshev_rejects_non_integral_or_negative_trailer_counts() {
    for (index, value) in [
        (2, f64::NAN),
        (2, -8.0),
        (2, 8.5),
        (3, f64::NAN),
        (3, -1.0),
        (3, 1.5),
        (3, f64::INFINITY),
    ] {
        let data = with_trailer_word(type2_data(), index, value);
        expect_truncated(
            eval_with(2, data, 0.0, |_| {}),
            &format!("trailer word {index} = {value} must be rejected"),
        );
    }
}

#[test]
fn chebyshev_rejects_record_count_larger_than_segment() {
    // One record is stored; claim a thousand. Evaluating near the end of the
    // claimed span would otherwise read records past the segment.
    let data = with_trailer_word(type2_data(), 3, 1000.0);
    let err = eval_with(2, data, 199_000.0, |_| {}).expect_err("N beyond the segment");
    assert_eq!(err.kind, SpkErrorKind::Truncated);
    assert!(
        err.message.contains("inconsistent with segment size"),
        "{}",
        err.message
    );
}

#[test]
fn chebyshev_rejects_rsize_not_split_into_coefficient_sets() {
    // Type 2: RSIZE - 2 must divide into 3 sets; Type 3 into 6.
    let rec = type2_record(0.0, 100.0, &[1.0, 0.0, 0.0], &[2.0, 0.0], &[3.0, 0.0]);
    let data = type2_segment_data(-100.0, 200.0, rec.len(), &[rec]);
    expect_truncated(eval_with(2, data, 0.0, |_| {}), "Type 2 with RSIZE 9");

    let data = with_trailer_word(type3_data(), 2, 11.0);
    expect_truncated(eval_with(3, data, 0.0, |_| {}), "Type 3 with RSIZE 11");
}

#[test]
fn chebyshev_rejects_crafted_segment_addresses() {
    let cases: [DescriptorEdit; 5] = [
        ("negative init_addr", |d| d.init_addr = -5),
        ("zero init_addr", |d| d.init_addr = 0),
        ("i32::MIN final_addr", |d| d.final_addr = i32::MIN),
        ("final before init", |d| d.final_addr = d.init_addr - 1),
        ("i32::MAX final_addr", |d| d.final_addr = i32::MAX),
    ];
    for (what, edit) in cases {
        expect_truncated(eval_with(2, type2_data(), 0.0, edit), what);
    }
}

#[test]
fn mda_rejects_crafted_segment_addresses() {
    let data = type21_single_record_segment(25, 100.0, [1.0; 3], [1.0; 3], 1000.0);
    let cases: [DescriptorEdit; 4] = [
        ("negative init_addr", |d| d.init_addr = -5),
        ("i32::MIN final_addr", |d| d.final_addr = i32::MIN),
        ("final before init", |d| d.final_addr = d.init_addr - 1),
        ("i32::MAX final_addr", |d| d.final_addr = i32::MAX),
    ];
    for (what, edit) in cases {
        expect_truncated(eval_with(21, data.clone(), 110.0, edit), what);
    }
}

#[test]
fn mda_rejects_record_count_whose_epoch_table_overruns_segment() {
    // Two valid 25-dim records (DLSIZE 111) and the [MAXDIM, NUMREC = 2]
    // trailer, but no room for the two-entry epoch table: the decoder would
    // read the trailer words as epochs.
    let one = type21_single_record_segment(25, 100.0, [1.0; 3], [1.0; 3], 1000.0);
    let record = &one[..111];
    let mut data = [record, record].concat();
    data.extend_from_slice(&[25.0, 2.0]);
    expect_truncated(
        eval_with(21, data, 110.0, |_| {}),
        "epoch table past the segment",
    );
}

#[test]
fn valid_segments_still_evaluate() {
    eval_with(2, type2_data(), 25.0, |_| {}).expect("valid Type 2");
    eval_with(3, type3_data(), 25.0, |_| {}).expect("valid Type 3");
    let data = type21_single_record_segment(25, 100.0, [1.0; 3], [1.0; 3], 1000.0);
    eval_with(21, data, 110.0, |_| {}).expect("valid Type 21");
}
