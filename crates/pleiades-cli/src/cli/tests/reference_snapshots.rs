//! Tests for reference-snapshot alias commands.

use crate::cli::render_cli;
#[test]
fn reference_snapshot_2451545_major_body_boundary_aliases_render_the_same_reports() {
    let boundary_2451545 = render_cli(&["reference-snapshot-2451545-major-body-boundary-summary"])
        .expect("2451545 major-body boundary summary should render");
    assert!(boundary_2451545.contains("Reference 2451545 major-body boundary evidence:"));
    assert!(boundary_2451545.contains("JD 2451545.0 (TDB)"));
    let boundary_2451545_alias = render_cli(&["2451545-major-body-boundary-summary"])
        .expect("2451545 major-body boundary alias should render");
    assert_eq!(boundary_2451545_alias, boundary_2451545);
}

#[test]
fn reference_snapshot_2453000_major_body_boundary_aliases_render_the_same_reports() {
    let boundary_2453000 = render_cli(&["reference-snapshot-2453000-major-body-boundary-summary"])
        .expect("2453000 major-body boundary summary should render");
    assert!(boundary_2453000.contains("Reference 2453000 major-body boundary evidence:"));
    let boundary_2453000_alias = render_cli(&["2453000-major-body-boundary-summary"])
        .expect("2453000 major-body boundary alias should render");
    assert_eq!(boundary_2453000_alias, boundary_2453000);
}

#[test]
fn reference_snapshot_1900_selected_body_boundary_aliases_render_the_same_reports() {
    let boundary_1900 = render_cli(&["reference-snapshot-1900-selected-body-boundary-summary"])
        .expect("1900 selected-body boundary summary should render");
    assert!(boundary_1900.contains("Reference 1900 selected-body boundary evidence:"));
    assert!(boundary_1900.contains("JD 2415020.5 (TDB)"));
    assert_eq!(
        render_cli(&["1900-selected-body-boundary-summary"])
            .expect("1900 selected-body boundary alias should render"),
        boundary_1900
    );
    assert_eq!(
        render_cli(&["2415020-selected-body-boundary-summary"])
            .expect("2415020 selected-body boundary alias should render"),
        pleiades_validate::reference_snapshot_2415020_selected_body_boundary_summary_for_report()
    );
}

#[test]
fn reference_snapshot_2451916_major_body_interior_aliases_render_the_same_reports() {
    let interior_2451916 = render_cli(&["reference-snapshot-2451916-major-body-interior-summary"])
        .expect("2451916 major-body interior summary should render");
    assert!(interior_2451916.contains("Reference 2451916 major-body interior evidence:"));
    assert!(interior_2451916.contains("JD 2451916.0 (TDB)"));
    let interior_2451916_alias = render_cli(&["2451916-major-body-interior-summary"])
        .expect("2451916 major-body interior alias should render");
    assert_eq!(interior_2451916_alias, interior_2451916);
}

#[test]
fn reference_snapshot_2451912_2451913_2451914_and_2451918_major_body_boundary_aliases_render_the_same_reports(
) {
    let boundary_2451912 = render_cli(&["reference-snapshot-2451912-major-body-boundary-summary"])
        .expect("2451912 major-body boundary summary should render");
    assert!(boundary_2451912.contains("Reference 2451912 major-body boundary evidence:"));
    assert!(boundary_2451912.contains("JD 2451912.5 (TDB)"));
    assert_eq!(
        render_cli(&["2451912-major-body-boundary-summary"])
            .expect("2451912 major-body boundary alias should render"),
        boundary_2451912
    );

    let boundary_2451913 = render_cli(&["reference-snapshot-2451913-major-body-boundary-summary"])
        .expect("2451913 major-body boundary summary should render");
    assert!(boundary_2451913.contains("Reference 2451913 major-body boundary evidence:"));
    assert!(boundary_2451913.contains("JD 2451913.5 (TDB)"));
    assert_eq!(
        render_cli(&["2451913-major-body-boundary-summary"])
            .expect("2451913 major-body boundary alias should render"),
        boundary_2451913
    );

    let boundary_2451914 = render_cli(&["reference-snapshot-2451914-major-body-boundary-summary"])
        .expect("2451914 major-body boundary summary should render");
    assert!(boundary_2451914.contains("Reference 2451914 major-body boundary evidence:"));
    assert!(boundary_2451914.contains("JD 2451914.5 (TDB)"));
    assert_eq!(
        render_cli(&["2451914-major-body-boundary-summary"])
            .expect("2451914 major-body boundary alias should render"),
        boundary_2451914
    );

    let boundary_2451918 = render_cli(&["reference-snapshot-2451918-major-body-boundary-summary"])
        .expect("2451918 major-body boundary summary should render");
    assert!(boundary_2451918.contains("Reference 2451918 major-body boundary evidence:"));
    assert!(boundary_2451918.contains("JD 2451918.5 (TDB)"));
    assert_eq!(
        render_cli(&["2451918-major-body-boundary-summary"])
            .expect("2451918 major-body boundary alias should render"),
        boundary_2451918
    );
}

#[test]
fn reference_snapshot_2451914_pre_bridge_2451914_bridge_2451915_bridge_and_2451916_dense_boundary_aliases_render_the_same_reports(
) {
    let pre_bridge = render_cli(&["reference-snapshot-2451914-major-body-pre-bridge-summary"])
        .expect("2451914 pre-bridge summary should render");
    assert!(pre_bridge.contains("Reference snapshot pre-bridge boundary day:"));
    assert!(pre_bridge.contains("JD 2451914.5 (TDB)"));
    assert_eq!(
        render_cli(&["2451914-major-body-pre-bridge-summary"])
            .expect("2451914 pre-bridge alias should render"),
        pre_bridge
    );

    let bridge_day = render_cli(&["reference-snapshot-2451914-major-body-bridge-summary"])
        .expect("2451914 bridge summary should render");
    assert!(bridge_day.contains("Reference snapshot bridge day:"));
    assert!(bridge_day.contains("JD 2451914.0 (TDB)"));
    assert_eq!(
        render_cli(&["2451914-major-body-bridge-summary"])
            .expect("2451914 bridge alias should render"),
        bridge_day
    );
    assert_eq!(
        render_cli(&["2451914-bridge-day-summary"])
            .expect("2451914 bridge-day alias should render"),
        bridge_day
    );
    assert_eq!(
        render_cli(&["2451914-major-body-bridge"])
            .expect("2451914 concise bridge alias should render"),
        bridge_day
    );

    let bridge_2451915 = render_cli(&["reference-snapshot-2451915-major-body-bridge-summary"])
        .expect("2451915 bridge summary should render");
    assert!(bridge_2451915.contains("Reference 2451915 major-body bridge evidence:"));
    assert!(bridge_2451915.contains("JD 2451915.0 (TDB)"));
    assert_eq!(
        render_cli(&["2451915-major-body-bridge-summary"])
            .expect("2451915 bridge alias should render"),
        pleiades_validate::reference_snapshot_2451915_major_body_bridge_summary_for_report()
    );
    assert_eq!(
        render_cli(&["2451915-major-body-bridge"])
            .expect("2451915 concise bridge alias should render"),
        bridge_2451915
    );
    assert_eq!(
        render_cli(&["bridge-summary"]).expect("bridge alias should render"),
        render_cli(&["major-body-bridge-summary"]).expect("major body bridge alias should render")
    );
    assert_eq!(
        render_cli(&["bridge-summary", "extra"])
            .expect_err("bridge alias should reject extra arguments"),
        "bridge-summary does not accept extra arguments"
    );

    let dense_boundary =
        render_cli(&["reference-snapshot-2451916-major-body-dense-boundary-summary"])
            .expect("2451916 dense boundary summary should render");
    assert!(dense_boundary.contains("Reference 2451916 major-body dense boundary evidence:"));
    assert!(dense_boundary.contains("JD 2451916.5 (TDB)"));
    assert_eq!(
        render_cli(&["2451916-major-body-dense-boundary-summary"])
            .expect("2451916 dense boundary alias should render"),
        dense_boundary
    );

    let boundary = render_cli(&["reference-snapshot-2451916-major-body-boundary-summary"])
        .expect("2451916 boundary alias should render");
    assert!(boundary.contains("Reference 2451916 major-body boundary evidence:"));
    assert!(boundary.contains("JD 2451916.5 (TDB)"));
    assert_eq!(
        render_cli(&["2451916-major-body-boundary-summary"])
            .expect("2451916 boundary alias should render"),
        boundary
    );
}

#[test]
fn reference_snapshot_2451917_major_body_boundary_aliases_render_the_same_reports() {
    let boundary_2451917 = render_cli(&["reference-snapshot-2451917-major-body-boundary-summary"])
        .expect("2451917 major-body boundary summary should render");
    assert!(boundary_2451917.contains("Reference 2451917 major-body boundary evidence:"));
    assert!(boundary_2451917.contains("JD 2451917.5 (TDB)"));
    let boundary_2451917_alias = render_cli(&["2451917-major-body-boundary-summary"])
        .expect("2451917 major-body boundary alias should render");
    assert_eq!(boundary_2451917_alias, boundary_2451917);

    let bridge_2451917 = render_cli(&["reference-snapshot-2451917-major-body-bridge-summary"])
        .expect("2451917 major-body bridge summary should render");
    assert!(bridge_2451917.contains("Reference 2451917 major-body bridge evidence:"));
    assert!(bridge_2451917.contains("JD 2451917.0 (TDB)"));
    assert_eq!(
        render_cli(&["2451917-major-body-bridge-summary"])
            .expect("2451917 major-body bridge alias should render"),
        bridge_2451917
    );
    assert_eq!(
        render_cli(&["2451917-major-body-bridge"])
            .expect("2451917 concise major-body bridge alias should render"),
        bridge_2451917
    );
}

#[test]
fn reference_snapshot_2451919_major_body_boundary_aliases_render_the_same_reports() {
    let boundary_2451919 = render_cli(&["reference-snapshot-2451919-major-body-boundary-summary"])
        .expect("2451919 major-body boundary summary should render");
    assert!(boundary_2451919.contains("Reference 2451919 major-body boundary evidence:"));
    assert!(boundary_2451919.contains("JD 2451919.5 (TDB)"));
    let boundary_2451919_alias = render_cli(&["2451919-major-body-boundary-summary"])
        .expect("2451919 major-body boundary alias should render");
    assert_eq!(boundary_2451919_alias, boundary_2451919);
    assert_eq!(
        render_cli(&[
            "reference-snapshot-2451919-major-body-boundary-summary",
            "extra"
        ])
        .expect_err("2451919 major-body boundary summary should reject extra arguments"),
        "reference-snapshot-2451919-major-body-boundary-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["2451919-major-body-boundary-summary", "extra"])
            .expect_err("2451919 major-body boundary alias should reject extra arguments"),
        "reference-snapshot-2451919-major-body-boundary-summary does not accept extra arguments"
    );
}

#[test]
fn reference_snapshot_2451920_major_body_interior_aliases_render_the_same_reports() {
    let interior_2451920 = render_cli(&["reference-snapshot-2451920-major-body-interior-summary"])
        .expect("2451920 major-body interior summary should render");
    assert!(interior_2451920.contains("Reference 2451920 major-body interior evidence:"));
    assert!(interior_2451920.contains("JD 2451920.5 (TDB)"));
    let interior_2451920_alias = render_cli(&["2451920-major-body-interior-summary"])
        .expect("2451920 major-body interior alias should render");
    assert_eq!(interior_2451920_alias, interior_2451920);
}
