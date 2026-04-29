use pleiades_validate::ComparisonAuditSummary;

#[test]
fn comparison_audit_summary_summary_line_is_public_and_matches_display() {
    let summary = ComparisonAuditSummary {
        body_count: 10,
        within_tolerance_body_count: 4,
        outside_tolerance_body_count: 6,
        regression_count: 12,
    };

    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(
        summary.summary_line(),
        "status=regressions found, bodies checked=10, within tolerance bodies=4, outside tolerance bodies=6, notable regressions=12"
    );
    assert_eq!(summary.validate(), Ok(()));
}
