use super::*;
use pleiades_apparent::fnv1a64;

#[test]
fn ayanamsa_apparent_gate_passes_within_ceilings() {
    let report = validate_ayanamsa_apparent_corpus().expect("gate");
    assert_eq!(report.rows_validated, 1040);
}

#[test]
fn tampered_corpus_fails_the_checksum() {
    let csv = CORPUS_CSV.replacen("TrueCitra,", "TrueCitra, ", 1);
    assert!(matches!(
        validate(&csv, MANIFEST),
        Err(AyanamsaApparentError::ChecksumMismatch { .. })
    ));
}

#[test]
fn manifest_row_count_drift_fails_closed() {
    let manifest = format!("slice ayanamsa-apparent file=ayanamsa-apparent.csv role=ayanamsa-apparent rows=1039 checksum={}", fnv1a64(CORPUS_CSV));
    assert!(matches!(
        validate(CORPUS_CSV, &manifest),
        Err(AyanamsaApparentError::ManifestDrift { .. })
    ));
}

#[test]
fn a_truncated_corpus_with_a_matching_manifest_fails_the_floor() {
    let csv: String = CORPUS_CSV
        .lines()
        .take(30)
        .map(|l| format!("{l}\n"))
        .collect();
    let manifest = format!(
        "slice x rows={} checksum={}",
        csv.lines()
            .filter(|l| !l.starts_with('#') && !l.starts_with("mode"))
            .count(),
        fnv1a64(&csv)
    );
    assert!(matches!(
        validate_with_floor(&csv, &manifest, MIN_ROWS_VALIDATED),
        Err(AyanamsaApparentError::TooFewRowsValidated { .. })
    ));
}

/// One row, its correction shifted by `shift_arcsec`.
fn one_row(mode: &str, class: &str, shift_arcsec: f64) -> (String, String) {
    let line = CORPUS_CSV
        .lines()
        .find(|l| l.starts_with(&format!("{mode},")) && l.contains(&format!(",{class},")))
        .expect("row");
    let mut fields: Vec<String> = line.split(',').map(str::to_string).collect();
    let value: f64 = fields[3].parse().unwrap();
    fields[3] = format!("{:.6}", value + shift_arcsec);
    let csv = format!(
        "mode,jd_tt,class,se_correction_arcsec\n{}\n",
        fields.join(",")
    );
    let manifest = format!("slice x rows=1 checksum={}", fnv1a64(&csv));
    (csv, manifest)
}

#[test]
fn an_unshifted_row_passes_alone() {
    let (csv, manifest) = one_row("TrueCitra", "uniform", 0.0);
    validate_with_floor(&csv, &manifest, 1).expect("passes");
}

// The gate would catch the composition dropping the aberration (≈20″) or
// the deflection near conjunction (≈2.8″ for δ Cnc).
#[test]
fn a_correction_off_by_a_ceiling_fails() {
    for (mode, class) in [("TrueCitra", "uniform"), ("TruePushya", "conjunction")] {
        let (csv, manifest) = one_row(mode, class, 1.0);
        assert!(
            matches!(
                validate_with_floor(&csv, &manifest, 1),
                Err(AyanamsaApparentError::CeilingExceeded { .. })
            ),
            "{mode} {class}"
        );
    }
}

// Issue #226: a mode Swiss Ephemeris does not aberrate must get no correction
// at all, not merely one under the ceiling; and an anchored mode must get one.
#[test]
fn a_disagreement_on_whether_a_mode_is_corrected_fails() {
    for (mode, swiss_ephemeris_corrects) in [
        ("GalacticEquatorTrue", true),
        ("GalacticCenterMardyks", true),
        ("TrueCitra", false),
    ] {
        let line = CORPUS_CSV
            .lines()
            .find(|l| l.starts_with(&format!("{mode},")))
            .expect("row");
        let mut fields: Vec<&str> = line.split(',').collect();
        // A nonzero value well under the ceiling for an unaberrated mode, or
        // an exact zero for an anchored one.
        fields[3] = if swiss_ephemeris_corrects {
            "0.001000"
        } else {
            "0.000000"
        };
        let csv = format!("{}\n", fields.join(","));
        let manifest = format!("slice x rows=1 checksum={}", fnv1a64(&csv));
        let error = validate_with_floor(&csv, &manifest, 1).expect_err(mode);
        assert!(
            matches!(
                error,
                AyanamsaApparentError::CorrectionPresence { swiss_ephemeris_corrects: got, .. }
                    if got == swiss_ephemeris_corrects
            ),
            "{mode}: {error}"
        );
    }
    // The unaberrated corpus rows themselves pass.
    let (csv, manifest) = one_row("GalacticCenterMardyks", "uniform", 0.0);
    validate_with_floor(&csv, &manifest, 1).expect("passes");
}

#[test]
fn malformed_rows_are_rejected() {
    for bad in [
        "TrueCitra,2451545.0,uniform",
        "TrueCitra,2451545.0,uniform,NaN",
        "Nonesuch,2451545.0,uniform,0.0",
        "TrueCitra,2451545.0,sideways,0.0",
    ] {
        let csv = format!("{bad}\n");
        let manifest = format!("slice x rows=1 checksum={}", fnv1a64(&csv));
        assert!(
            matches!(
                validate_with_floor(&csv, &manifest, 1),
                Err(AyanamsaApparentError::MalformedRow(_))
            ),
            "{bad}"
        );
    }
}

/// Measurement helper, not a gate: prints the maxima the ceilings are sized
/// from. `cargo test -p pleiades-validate --lib measure_ayanamsa_apparent -- --ignored --nocapture`
#[test]
#[ignore]
fn measure_ayanamsa_apparent_maxima() {
    let rows = parse_rows(CORPUS_CSV).unwrap();
    let (mut uniform, mut conjunction) = (0.0f64, 0.0f64);
    for row in rows {
        let r = residual_arcsec(&row).unwrap();
        match row.class {
            RowClass::Uniform => uniform = uniform.max(r),
            RowClass::Conjunction => conjunction = conjunction.max(r),
        }
    }
    println!("uniform max {uniform:.4}\"  conjunction max {conjunction:.4}\"");
}
