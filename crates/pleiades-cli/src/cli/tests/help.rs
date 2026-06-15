//! Tests for help/banner CLI output.

use crate::cli::{banner, render_cli};
#[test]
fn banner_mentions_package() {
    assert!(banner().contains("pleiades-cli"));
}

#[test]
fn help_text_mentions_tdb_to_tt_retagging_flag() {
    let rendered = render_cli(&["help"]).expect("help should render");
    assert!(rendered.contains("--tt-from-utc-offset-seconds"));
    assert!(rendered.contains("--tt-from-ut1-offset-seconds"));
    assert!(rendered.contains("--tdb-from-utc-offset-seconds"));
    assert!(rendered.contains("--tdb-from-ut1-offset-seconds"));
    assert!(rendered.contains("--tdb-from-tt-offset-seconds"));
    assert!(rendered.contains("--tt-from-tdb-offset-seconds"));
    assert!(rendered.contains("UTC convenience policy: built-in UTC convenience conversion remains out of scope; callers must supply TT/TDB offsets explicitly"));
    assert!(rendered.contains("reference-high-curvature-summary"));
    assert!(rendered.contains("high-curvature-summary"));
    assert!(rendered.contains("reference-snapshot-major-body-high-curvature-summary"));
    assert!(rendered.contains("major-body-high-curvature-summary"));
    assert!(rendered.contains("reference-snapshot-2500-major-body-boundary-summary"));
    assert!(rendered.contains("2500-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2500-selected-body-boundary-summary"));
    assert!(rendered.contains("2500-selected-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2453000-major-body-boundary-summary"));
    assert!(rendered.contains("2453000-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2500000-major-body-boundary-summary"));
    assert!(rendered.contains("2500000-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451545-major-body-boundary-summary"));
    assert!(rendered.contains("2451545-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451910-major-body-boundary-summary"));
    assert!(rendered.contains("2451910-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451911-major-body-boundary-summary"));
    assert!(rendered.contains("2451911-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451912-major-body-boundary-summary"));
    assert!(rendered.contains("2451912-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451913-major-body-boundary-summary"));
    assert!(rendered.contains("2451913-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451914-major-body-boundary-summary"));
    assert!(rendered.contains("2451914-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451914-major-body-pre-bridge-summary"));
    assert!(rendered.contains("2451914-major-body-pre-bridge-summary"));
    assert!(rendered.contains("reference-snapshot-2451914-major-body-bridge-summary"));
    assert!(rendered.contains("2451914-major-body-bridge-summary"));
    assert!(rendered.contains("reference-snapshot-2451914-major-body-bridge-day-summary"));
    assert!(rendered.contains("2451914-major-body-bridge-day-summary"));
    assert!(rendered.contains("reference-snapshot-2451915-major-body-boundary-summary"));
    assert!(rendered.contains("2451915-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451915-major-body-bridge-summary"));
    assert!(rendered.contains("2451915-major-body-bridge-summary"));
    assert!(rendered.contains("reference-snapshot-2451917-major-body-boundary-summary"));
    assert!(rendered.contains("2451917-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451918-major-body-boundary-summary"));
    assert!(rendered.contains("2451918-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451916-major-body-dense-boundary-summary"));
    assert!(rendered.contains("2451916-major-body-dense-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-2451916-major-body-interior-summary"));
    assert!(rendered.contains("2451916-major-body-interior-summary"));
    assert!(rendered.contains("reference-snapshot-2451920-major-body-interior-summary"));
    assert!(rendered.contains("2451920-major-body-interior-summary"));
    assert!(rendered.contains("reference-snapshot-1749-major-body-boundary-summary"));
    assert!(rendered.contains("1749-major-body-boundary-summary"));
    assert!(rendered.contains("2360233-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-1800-major-body-boundary-summary"));
    assert!(rendered.contains("1800-major-body-boundary-summary"));
    assert!(rendered.contains("2378499-major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-major-body-boundary-summary"));
    assert!(rendered.contains("major-body-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-mars-jupiter-boundary-summary"));
    assert!(rendered.contains("mars-jupiter-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-major-body-boundary-window-summary"));
    assert!(rendered.contains("major-body-boundary-window-summary"));
    assert!(rendered.contains("reference-high-curvature-window-summary"));
    assert!(rendered.contains("high-curvature-window-summary"));
    assert!(rendered.contains("reference-snapshot-major-body-high-curvature-window-summary"));
    assert!(rendered.contains("major-body-high-curvature-window-summary"));
    assert!(rendered.contains("reference-high-curvature-epoch-coverage-summary"));
    assert!(rendered.contains("high-curvature-epoch-coverage-summary"));
    assert!(
        rendered.contains("reference-snapshot-major-body-high-curvature-epoch-coverage-summary")
    );
    assert!(rendered.contains("major-body-high-curvature-epoch-coverage-summary"));
    assert!(rendered.contains("reference-snapshot-sparse-boundary-summary"));
    assert!(rendered.contains("sparse-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-boundary-epoch-coverage-summary"));
    assert!(rendered.contains(
        "reference-snapshot-boundary-epoch-coverage  Alias for reference-snapshot-boundary-epoch-coverage-summary"
    ));
    assert!(rendered.contains("boundary-epoch-coverage-summary"));
    assert!(rendered.contains("reference-snapshot-pre-bridge-boundary-summary"));
    assert!(rendered.contains(
        "reference-snapshot-pre-bridge-boundary  Alias for reference-snapshot-pre-bridge-boundary-summary"
    ));
    assert!(rendered.contains("pre-bridge-boundary-summary"));
    assert!(rendered.contains("reference-snapshot-dense-boundary-summary"));
    assert!(rendered.contains("dense-boundary-summary"));
    assert!(rendered.contains("early-major-body-boundary-summary"));
    assert!(rendered.contains("1800-major-body-boundary-summary"));
    assert!(rendered.contains("source-documentation-summary"));
    assert!(
        rendered.contains("source-documentation         Alias for source-documentation-summary")
    );
    assert!(rendered.contains("source-documentation-health-summary"));
    assert!(rendered
        .contains("source-documentation-health  Alias for source-documentation-health-summary"));
    assert!(rendered
        .contains("source-audit-summary      Print the compact VSOP87 source audit summary"));
    assert!(rendered.contains("source-audit              Alias for source-audit-summary"));
    assert!(rendered.contains(
        "generated-binary-audit-summary  Print the compact VSOP87 generated binary audit summary"
    ));
    assert!(rendered.contains("generated-binary-audit    Alias for generated-binary-audit-summary"));
    assert!(rendered.contains("time-scale-policy-summary"));
    assert!(rendered.contains("mean-obliquity-frame-round-trip-summary"));
    assert!(rendered.contains(
        "mean-obliquity-frame-round-trip  Alias for mean-obliquity-frame-round-trip-summary"
    ));
    assert!(rendered.contains("production-generation-body-class-coverage-summary"));
    assert!(rendered.contains("production-body-class-coverage-summary"));
    assert!(rendered.contains("production-generation-boundary-request-corpus-summary"));
    assert!(rendered.contains("comparison-snapshot-body-class-coverage-summary"));
    assert!(rendered.contains("comparison-body-class-coverage-summary"));
    assert!(rendered.contains("comparison-corpus-summary"));
    assert!(rendered.contains("comparison-corpus         Alias for comparison-corpus-summary"));
    assert!(rendered.contains("comparison-corpus-release-guard-summary"));
    assert!(rendered.contains(
        "comparison-corpus-release-guard  Alias for comparison-corpus-release-guard-summary"
    ));
    assert!(rendered.contains("comparison-corpus-guard-summary"));
    assert!(rendered.contains("comparison-envelope-summary"));
    assert!(rendered.contains("comparison-envelope       Alias for comparison-envelope-summary"));
    assert!(rendered.contains("comparison-tolerance-summary"));
    assert!(rendered
        .contains("comparison-tolerance-summary  Alias for comparison-tolerance-policy-summary"));
    assert!(rendered.contains("comparison-tolerance-scope-coverage-summary"));
    assert!(rendered.contains(
        "comparison-tolerance-scope-coverage-summary  Print the compact comparison tolerance scope coverage summary"
    ));
    assert!(rendered.contains(
        "comparison-tolerance-scope-coverage  Alias for comparison-tolerance-scope-coverage-summary"
    ));
    assert!(rendered.contains("comparison-body-class-tolerance-summary"));
    assert!(rendered.contains(
        "comparison-body-class-tolerance-summary  Print the compact comparison body-class tolerance summary"
    ));
    assert!(rendered.contains(
        "comparison-body-class-tolerance  Alias for comparison-body-class-tolerance-summary"
    ));
    assert!(rendered.contains("comparison-body-class-error-envelope-summary"));
    assert!(rendered.contains(
        "comparison-body-class-error-envelope  Alias for comparison-body-class-error-envelope-summary"
    ));
    assert!(rendered.contains("comparison-body-class-tolerance-posture-summary"));
    assert!(rendered.contains(
        "comparison-body-class-tolerance-posture-summary  Print the compact comparison body-class tolerance posture summary"
    ));
    assert!(rendered.contains(
        "comparison-body-class-tolerance-posture  Alias for comparison-body-class-tolerance-posture-summary"
    ));
    assert!(rendered.contains("benchmark-corpus-summary"));
    assert!(rendered.contains("comparison-snapshot-summary"));
    assert!(rendered.contains("comparison-snapshot-batch-parity-summary"));
    assert!(rendered.contains(
        "comparison-snapshot-batch-parity  Alias for comparison-snapshot-batch-parity-summary"
    ));
    assert!(rendered.contains("reference-snapshot-body-class-coverage-summary"));
    assert!(rendered.contains("reference-body-class-coverage-summary"));
    assert!(rendered.contains("reference-snapshot-summary"));
    assert!(rendered.contains("reference-snapshot-batch-parity-summary"));
    assert!(rendered.contains("reference-snapshot-equatorial-parity-summary"));
    assert!(rendered.contains("workspace-audit-summary"));
    assert!(rendered.contains("native-dependency-audit-summary"));
    assert!(rendered.contains("independent-holdout-source-window-summary"));
    assert!(rendered.contains("independent-holdout-body-class-coverage-summary"));
    assert!(rendered.contains("holdout-body-class-coverage-summary"));
    assert!(rendered.contains("independent-holdout-batch-parity-summary"));
    assert!(rendered.contains("independent-holdout-equatorial-parity-summary"));
    assert!(rendered.contains("lunar-theory-summary"));
    assert!(rendered.contains("lunar-theory-request-policy-summary"));
    assert!(rendered
        .contains("lunar-theory-request-policy  Alias for lunar-theory-request-policy-summary"));
    assert!(rendered.contains("request-policy-summary    Print the compact request-policy summary"));
    assert!(rendered.contains("request-policy           Alias for request-policy-summary"));
    assert!(
        rendered.contains("request-semantics-summary  Print the compact request-semantics summary")
    );
    assert!(rendered.contains("request-semantics        Alias for request-semantics-summary"));
    assert!(rendered.contains("lunar-theory-frame-treatment-summary"));
    assert!(rendered
        .contains("lunar-theory-frame-treatment  Alias for lunar-theory-frame-treatment-summary"));
    assert!(rendered.contains("lunar-theory-limitations-summary"));
    assert!(rendered.contains("lunar-theory-capability-summary"));
    assert!(rendered.contains("lunar-theory-source-summary"));
    assert!(rendered.contains("lunar-theory-source-family-summary"));
    assert!(rendered
        .contains("lunar-theory-source-family  Alias for lunar-theory-source-family-summary"));
    assert!(rendered.contains("lunar-theory-catalog-summary"));
    assert!(rendered.contains("lunar-theory-catalog-validation-summary"));
    assert!(rendered.contains("lunar-theory-catalog      Alias for lunar-theory-catalog-summary"));
    assert!(rendered.contains(
        "lunar-theory-catalog-validation  Alias for lunar-theory-catalog-validation-summary"
    ));
    assert!(rendered.contains("zodiac-policy-summary"));
    assert!(rendered.contains("zodiac-policy         Alias for zodiac-policy-summary"));
    assert!(rendered.contains("observer-policy-summary"));
    assert!(rendered.contains("apparentness-policy-summary"));
    assert!(rendered.contains("compare-backends-audit"));
    assert!(rendered.contains("Caller-supplied signed TDB-TT offset for TT-tagged instants"));
    assert!(rendered.contains("Caller-supplied signed TT-TDB offset for TDB-tagged instants"));
}

#[test]
fn help_text_lists_the_packaged_lookup_epoch_policy_summary_command() {
    let help = render_cli(&["help"]).expect("help text should render");
    assert!(help.contains(
        "packaged-lookup-epoch-policy-summary  Print the packaged lookup epoch policy summary"
    ));
    assert!(help.contains(
        "packaged-lookup-epoch-policy         Alias for packaged-lookup-epoch-policy-summary"
    ));
    assert!(help.contains("packaged-artifact-lookup-epoch-policy-summary"));
    assert!(help.contains("Alias for packaged-artifact-lookup-epoch-policy-summary"));
    assert!(help.contains(
        "benchmark-matrix-summary [--rounds N]  Print the compact benchmark matrix summary"
    ));
    assert!(help.contains("benchmark-matrix [--rounds N]  Alias for benchmark-matrix-summary"));
    assert!(help.contains(
        "production-generation-summary  Print the compact production-generation coverage summary"
    ));
    assert!(help.contains("source-corpus-posture-summary  Alias for source-corpus-summary"));
    assert!(help.contains("source-corpus-posture     Alias for source-corpus-posture-summary"));
    assert!(help.contains(
        "production-generation-boundary-summary  Print the compact production-generation boundary overlay summary"
    ));
    assert!(help.contains(
        "production-generation-boundary         Alias for production-generation-boundary-summary"
    ));
    assert!(help.contains(
        "production-generation-quarter-day-boundary-summary  Print the compact production-generation quarter-day boundary samples summary"
    ));
    assert!(help.contains(
        "production-generation-boundary-request-corpus-summary  Print the compact production-generation boundary request corpus summary"
    ));
    assert!(help.contains(
        "production-generation-boundary-request-corpus-equatorial-summary  Print the compact production-generation boundary request corpus summary in the equatorial frame"
    ));
    assert!(help.contains(
        "production-generation-boundary-request-corpus-equatorial  Alias for production-generation-boundary-request-corpus-equatorial-summary"
    ));
    assert!(help.contains(
        "production-generation-source-revision-summary  Print the compact production-generation source revision summary"
    ));
    assert!(help.contains(
        "production-generation-source-revision  Alias for production-generation-source-revision-summary"
    ));
    assert!(help.contains(
        "production-generation-manifest-summary  Print the compact production-generation manifest summary"
    ));
    assert!(help.contains(
        "production-generation-manifest  Alias for production-generation-manifest-summary"
    ));
    assert!(help.contains(
        "production-generation-manifest-checksum-summary  Print the compact production-generation manifest checksum summary"
    ));
    assert!(help.contains(
        "production-generation-manifest-checksum  Alias for production-generation-manifest-checksum-summary"
    ));
    assert!(help.contains(
        "production-generation-source-window-summary  Print the compact production-generation source windows summary"
    ));
    assert!(help.contains(
        "production-generation-corpus-shape-summary  Print the compact production-generation corpus shape summary"
    ));
    assert!(help.contains(
        "interpolation-quality-request-corpus-summary  Print the compact JPL interpolation-quality sample request corpus summary"
    ));
    assert!(help.contains(
        "production-generation-boundary-source-summary  Print the compact production-generation boundary source summary"
    ));
    assert!(help.contains(
        "production-generation-source-summary  Print the compact production-generation source summary"
    ));
    assert!(help.contains(
        "production-generation-source      Alias for production-generation-source-summary"
    ));
    assert!(
        help.contains("production-generation           Alias for production-generation-summary")
    );
    assert!(help.contains(
        "production-generation-boundary-window-summary  Print the compact production-generation boundary windows summary"
    ));
    assert!(help.contains(
        "production-generation-boundary-window  Alias for production-generation-boundary-window-summary"
    ));
    assert!(help.contains(
        "compatibility-caveats-summary  Print the compact compatibility caveats summary"
    ));
    assert!(help.contains("compatibility-caveats    Alias for compatibility-caveats-summary"));
    assert!(help.contains("workspace-audit-summary   Print the compact workspace audit summary"));
    assert!(help.contains("native-dependency-audit-summary  Alias for workspace-audit-summary"));
    assert!(help
        .contains("workspace-provenance-summary  Print the compact workspace provenance summary"));
    assert!(help.contains("workspace-provenance     Alias for workspace-provenance-summary"));
    assert!(help.contains(
        "catalog-inventory-summary  Print the compact compatibility catalog inventory summary"
    ));
    assert!(help.contains("catalog-inventory        Alias for catalog-inventory-summary"));
    assert!(help.contains(
        "catalog-posture-summary   Print the compact compatibility catalog posture summary"
    ));
    assert!(help.contains("catalog-posture         Alias for catalog-posture-summary"));
    assert!(
        help.contains("known-gaps-summary      Print the compact compatibility known-gaps summary")
    );
    assert!(help.contains("known-gaps              Alias for known-gaps-summary"));
    assert!(help.contains(
        "custom-definition-ayanamsa-labels-summary  Print the compact custom-definition ayanamsa labels summary"
    ));
    assert!(help.contains(
        "custom-definition-ayanamsa-labels  Alias for custom-definition-ayanamsa-labels-summary"
    ));
    assert!(
        help.contains("ayanamsa-provenance-summary  Print the compact ayanamsa provenance summary")
    );
    assert!(help.contains("ayanamsa-provenance        Alias for ayanamsa-provenance-summary"));
    assert!(help.contains("ayanamsa-audit-summary    Print the compact ayanamsa audit summary"));
    assert!(help.contains("ayanamsa-audit            Alias for ayanamsa-audit-summary"));
    assert!(help.contains(
        "release-house-system-canonical-names-summary  Print the compact release-specific house-system canonical names summary"
    ));
    assert!(help.contains(
        "release-house-system-canonical-names  Alias for release-house-system-canonical-names-summary"
    ));
    assert!(help.contains(
        "release-ayanamsa-canonical-names-summary  Print the compact release-specific ayanamsa canonical names summary"
    ));
    assert!(help.contains(
        "release-ayanamsa-canonical-names  Alias for release-ayanamsa-canonical-names-summary"
    ));
    assert!(help.contains(
        "house-latitude-sensitive-summary  Print the compact latitude-sensitive house systems summary"
    ));
    assert!(help.contains("house-latitude-sensitive  Alias for house-latitude-sensitive-summary"));
    assert!(help.contains("profile-summary           Alias for compatibility-profile-summary"));
    assert!(
        help.contains("release-profile-identifiers  Alias for release-profile-identifiers-summary")
    );
    assert!(help.contains(
        "artifact-profile-coverage-summary  Print the packaged-artifact profile coverage summary"
    ));
    assert!(help.contains(
        "packaged-artifact-output-support-summary  Print the packaged-artifact output support summary"
    ));
    assert!(help.contains(
        "packaged-artifact-output-support       Alias for packaged-artifact-output-support-summary"
    ));
    assert!(help.contains(
        "packaged-artifact-body-class-span-cap-summary  Print the packaged-artifact body-class span caps summary"
    ));
    assert!(help.contains(
        "packaged-artifact-body-class-span-cap  Alias for packaged-artifact-body-class-span-cap-summary"
    ));
    assert!(help.contains(
        "packaged-artifact-speed-policy-summary  Print the packaged-artifact speed policy summary"
    ));
    assert!(help.contains(
        "packaged-artifact-speed-policy       Alias for packaged-artifact-speed-policy-summary"
    ));
    assert!(help.contains("motion-policy-summary         Print the compact motion policy summary"));
    assert!(help.contains("motion-policy               Alias for motion-policy-summary"));
    assert!(help
        .contains("packaged-artifact-access-summary  Print the packaged-artifact access summary"));
    assert!(help.contains("packaged-artifact-access  Alias for packaged-artifact-access-summary"));
    assert!(help.contains(
        "packaged-artifact-path-policy-summary  Alias for packaged-artifact-access-summary"
    ));
    assert!(help.contains(
        "packaged-artifact-path-policy  Alias for packaged-artifact-path-policy-summary"
    ));
    assert!(help.contains(
        "packaged-artifact-storage-summary  Print the packaged-artifact storage/reconstruction summary"
    ));
    assert!(help.contains(
        "packaged-artifact-storage           Alias for packaged-artifact-storage-summary"
    ));
    assert!(help.contains(
        "packaged-artifact-production-profile-summary  Print the packaged-artifact production profile draft summary"
    ));
    assert!(help.contains(
        "packaged-artifact-production-profile  Alias for packaged-artifact-production-profile-summary"
    ));
    assert!(help.contains(
        "packaged-artifact-target-threshold-summary  Print the packaged-artifact target thresholds summary"
    ));
    assert!(help.contains(
        "packaged-artifact-target-threshold  Alias for packaged-artifact-target-threshold-summary"
    ));
    assert!(help.contains(
        "packaged-artifact-target-threshold-scope-envelopes-summary  Print the packaged-artifact target-threshold scope envelopes summary"
    ));
    assert!(help.contains(
        "packaged-artifact-target-threshold-scope-envelopes  Alias for packaged-artifact-target-threshold-scope-envelopes-summary"
    ));
    assert!(help.contains(
        "packaged-artifact-source-fit-holdout-sync-summary  Print the packaged-artifact source-fit and hold-out sync summary"
    ));
    assert!(help.contains(
        "packaged-artifact-source-fit-holdout-sync  Alias for packaged-artifact-source-fit-holdout-sync-summary"
    ));
    assert!(help.contains(
        "packaged-artifact-fit-envelope-summary  Print the packaged-artifact fit envelope summary"
    ));
    assert!(help.contains(
        "packaged-artifact-fit-envelope  Alias for packaged-artifact-fit-envelope-summary"
    ));
    assert!(help.contains(
        "packaged-artifact-fit-sample-classes-summary  Print the packaged-artifact fit sample classes summary"
    ));
    assert!(help.contains(
        "packaged-artifact-fit-sample-classes  Alias for packaged-artifact-fit-sample-classes-summary"
    ));
    assert!(help.contains(
        "packaged-artifact-fit-outliers-summary  Print the packaged-artifact body/channel fit outlier summary"
    ));
    assert!(help.contains(
        "packaged-artifact-fit-outliers  Alias for packaged-artifact-fit-outliers-summary"
    ));
    assert!(help.contains(
        "packaged-artifact-fit-threshold-violation-count-summary  Print the packaged-artifact fit threshold violation count summary"
    ));
    assert!(help.contains(
        "packaged-artifact-fit-threshold-violation-count  Alias for packaged-artifact-fit-threshold-violation-count-summary"
    ));
    assert!(help.contains(
        "packaged-artifact-fit-threshold-violations-summary  Print the packaged-artifact fit threshold violations summary"
    ));
    assert!(help.contains(
        "packaged-artifact-fit-threshold-violations  Alias for packaged-artifact-fit-threshold-violations-summary"
    ));
    assert!(help.contains(
        "packaged-artifact-generation-manifest-summary  Print the packaged-artifact generation manifest summary"
    ));
    assert!(help.contains(
        "packaged-artifact-generation-manifest  Alias for packaged-artifact-generation-manifest-summary"
    ));
    assert!(help.contains(
        "packaged-artifact-generation-policy-summary  Print the packaged-artifact generation policy summary"
    ));
    assert!(help.contains(
        "packaged-artifact-generation-policy     Alias for packaged-artifact-generation-policy-summary"
    ));
    assert!(help.contains(
        "packaged-artifact-generation-residual-summary  Alias for packaged-artifact-generation-residual-bodies-summary"
    ));
    assert!(help.contains(
        "packaged-artifact-generation-residual-bodies-summary  Print the packaged-artifact generation residual bodies summary"
    ));
    assert!(help.contains(
        "packaged-artifact-regeneration-summary  Print the packaged-artifact regeneration summary"
    ));
    assert!(help.contains(
        "packaged-artifact-regeneration      Alias for packaged-artifact-regeneration-summary"
    ));
    assert!(help.contains("packaged-frame-parity-summary  Print the packaged frame parity summary"));
    assert!(help.contains("packaged-frame-parity         Alias for packaged-frame-parity-summary"));
    assert!(help
        .contains("packaged-frame-treatment-summary  Print the packaged frame treatment summary"));
    assert!(
        help.contains("comparison-envelope-summary  Print the compact comparison envelope summary")
    );
    assert!(help.contains(
        "comparison-body-class-error-envelope-summary  Print the compact comparison body-class error envelope summary"
    ));
    assert!(help.contains(
        "comparison-body-class-error-envelope  Alias for comparison-body-class-error-envelope-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-1749-major-body-boundary-summary  Print the compact reference 1749 major-body boundary evidence summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2268932-selected-body-boundary-summary  Print the compact reference 2268932 selected-body boundary evidence summary"
    ));
    assert!(help.contains(
        "2268932-selected-body-boundary-summary  Alias for reference-snapshot-2268932-selected-body-boundary-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-1600-selected-body-boundary-summary  Print the compact reference 1600 selected-body boundary evidence summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2305457-selected-body-boundary-summary  Print the compact reference 2305457 selected-body boundary evidence summary"
    ));
    assert!(help.contains(
        "2305457-selected-body-boundary-summary  Alias for reference-snapshot-2305457-selected-body-boundary-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-1750-selected-body-boundary-summary  Print the compact reference 1750 selected-body boundary evidence summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2200-selected-body-boundary-summary  Print the compact reference 2200 selected-body boundary evidence summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2524593-selected-body-boundary-summary  Print the compact reference 2524593 selected-body boundary evidence summary"
    ));
    assert!(help.contains(
        "2524593-selected-body-boundary-summary  Alias for reference-snapshot-2524593-selected-body-boundary-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2634167-selected-body-boundary-summary  Print the compact reference 2634167 selected-body boundary evidence summary"
    ));
    assert!(help.contains(
        "2634167-selected-body-boundary-summary  Alias for reference-snapshot-2634167-selected-body-boundary-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-early-major-body-boundary-summary  Print the compact reference early major-body boundary evidence summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2378498-major-body-boundary-summary  Print the compact reference 2378498 major-body boundary evidence summary"
    ));
    assert!(help.contains(
        "2378498-major-body-boundary-summary  Alias for reference-snapshot-2378498-major-body-boundary-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-1800-major-body-boundary-summary  Print the compact reference 1800 major-body boundary evidence summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2500-major-body-boundary-summary  Print the compact reference 2500 major-body boundary evidence summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2453000-major-body-boundary-summary  Print the compact reference 2453000 major-body boundary evidence summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2451910-major-body-boundary-summary  Print the compact reference 2451910 major-body boundary evidence summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2451911-major-body-boundary-summary  Print the compact reference 2451911 major-body boundary evidence summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2451915-major-body-bridge-summary  Print the compact reference 2451915 major-body bridge evidence summary"
    ));
    assert!(help.contains(
        "2451915-major-body-bridge-summary  Alias for reference-snapshot-2451915-major-body-bridge-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2451916-major-body-dense-boundary-summary  Print the compact reference 2451916 major-body dense boundary evidence summary"
    ));
    assert!(help.contains(
        "2451916-major-body-dense-boundary-summary  Alias for reference-snapshot-2451916-major-body-dense-boundary-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2451916-major-body-boundary-summary  Print the compact reference 2451916 major-body boundary evidence summary"
    ));
    assert!(help.contains(
        "2451916-major-body-boundary-summary  Alias for reference-snapshot-2451916-major-body-boundary-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2451917-major-body-boundary-summary  Print the compact reference 2451917 major-body boundary evidence summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2451918-major-body-boundary-summary  Print the compact reference 2451918 major-body boundary evidence summary"
    ));
    assert!(help.contains(
        "2451918-major-body-boundary-summary  Alias for reference-snapshot-2451918-major-body-boundary-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2451919-major-body-boundary-summary  Print the compact reference 2451919 major-body boundary evidence summary"
    ));
    assert!(help.contains(
        "2451919-major-body-boundary-summary  Alias for reference-snapshot-2451919-major-body-boundary-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2451920-major-body-interior-summary  Print the compact reference 2451920 major-body interior evidence summary"
    ));
    assert!(help.contains(
        "2451920-major-body-interior-summary  Alias for reference-snapshot-2451920-major-body-interior-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-major-body-boundary-summary  Print the compact reference major-body boundary evidence summary"
    ));
    assert!(help.contains(
        "major-body-boundary-summary  Alias for reference-snapshot-major-body-boundary-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-major-body-bridge-summary  Print the compact reference major-body bridge evidence summary"
    ));
    assert!(help.contains("bridge-summary  Alias for reference-snapshot-major-body-bridge-summary"));
    assert!(help.contains(
        "reference-snapshot-major-body-boundary-window-summary  Print the compact reference major-body boundary windows summary"
    ));
    assert!(help.contains(
        "major-body-boundary-window-summary  Alias for reference-snapshot-major-body-boundary-window-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-mars-jupiter-boundary-summary  Print the compact reference Mars/Jupiter boundary evidence summary"
    ));
    assert!(help.contains(
        "mars-jupiter-boundary-summary  Alias for reference-snapshot-mars-jupiter-boundary-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-mars-outer-boundary-summary  Print the compact reference Mars outer-boundary evidence summary"
    ));
    assert!(help.contains(
        "mars-outer-boundary-summary  Alias for reference-snapshot-mars-outer-boundary-summary"
    ));
    assert!(help.contains(
        "lunar-reference-error-envelope-summary  Print the compact lunar reference error envelope summary"
    ));
    assert!(help.contains(
        "lunar-reference-error-envelope  Alias for lunar-reference-error-envelope-summary"
    ));
    assert!(help.contains(
        "lunar-reference-evidence-summary  Print the compact lunar reference evidence summary"
    ));
    assert!(help.contains("lunar-reference-evidence  Alias for lunar-reference-evidence-summary"));
    assert!(help.contains(
        "lunar-equatorial-reference-error-envelope-summary  Print the compact lunar equatorial reference error envelope summary"
    ));
    assert!(help.contains(
        "lunar-equatorial-reference-error-envelope  Alias for lunar-equatorial-reference-error-envelope-summary"
    ));
    assert!(help.contains(
        "lunar-apparent-comparison-summary  Print the compact lunar apparent comparison summary"
    ));
    assert!(help.contains("lunar-apparent-comparison  Alias for lunar-apparent-comparison-summary"));
    assert!(help
        .contains("lunar-source-window-summary  Print the compact lunar source windows summary"));
    assert!(help.contains(
        "reference-snapshot-lunar-source-window-summary  Alias for lunar-source-window-summary"
    ));
    assert!(help.contains("lunar-source-window  Alias for lunar-source-window-summary"));
    assert!(help.contains(
        "lunar-reference-mixed-time-scale-batch-parity-summary  Print the compact lunar reference mixed TT/TDB batch parity summary"
    ));
    assert!(help.contains(
        "lunar-reference-mixed-tt-tdb-batch-parity-summary  Alias for lunar-reference-mixed-time-scale-batch-parity-summary"
    ));
    assert!(help.contains(
        "lunar-reference-mixed-tt-tdb-batch-parity  Alias for lunar-reference-mixed-time-scale-batch-parity-summary"
    ));
    assert!(help.contains(
        "comparison-snapshot-manifest-summary  Print the compact comparison snapshot manifest summary"
    ));
    assert!(help
        .contains("comparison-snapshot-manifest  Alias for comparison-snapshot-manifest-summary"));
    assert!(help.contains(
        "independent-holdout-batch-parity-summary  Print the compact independent hold-out batch parity summary"
    ));
    assert!(help.contains(
        "independent-holdout-batch-parity  Alias for independent-holdout-batch-parity-summary"
    ));
    assert!(help.contains(
        "independent-holdout-equatorial-parity-summary  Print the compact independent hold-out equatorial parity summary"
    ));
    assert!(help.contains(
        "independent-holdout-equatorial-parity  Alias for independent-holdout-equatorial-parity-summary"
    ));
    assert!(
        help.contains("comparison-snapshot-summary  Print the compact comparison snapshot summary")
    );
    assert!(help.contains("j2000-snapshot           Alias for comparison-snapshot-summary"));
    assert!(help.contains("comparison-snapshot         Alias for comparison-snapshot-summary"));
    assert!(help.contains(
        "comparison-snapshot-source-summary  Print the compact comparison snapshot source summary"
    ));
    assert!(help.contains(
        "comparison-snapshot-source-window  Alias for comparison-snapshot-source-window-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-manifest-summary  Print the compact reference snapshot manifest summary"
    ));
    assert!(
        help.contains("reference-snapshot-manifest  Alias for reference-snapshot-manifest-summary")
    );
    assert!(help.contains("reference-snapshot         Alias for reference-snapshot-summary"));
    assert!(help.contains(
        "reference-snapshot-source-window  Alias for reference-snapshot-source-window-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-source-summary  Print the compact reference snapshot source summary"
    ));
    assert!(help.contains(
        "reference-snapshot-boundary-day-summary  Print the compact reference snapshot boundary day summary"
    ));
    assert!(help.contains(
        "reference-snapshot-boundary-day  Alias for reference-snapshot-boundary-day-summary"
    ));
    assert!(
        help.contains("boundary-day-summary     Alias for reference-snapshot-boundary-day-summary")
    );
    assert!(
        help.contains("reference-snapshot-summary  Print the compact reference snapshot summary")
    );
    assert!(help.contains(
        "reference-snapshot-exact-j2000-evidence-summary  Print the compact reference snapshot exact J2000 evidence summary"
    ));
    assert!(help.contains(
        "reference-snapshot-batch-parity-summary  Print the compact reference snapshot batch parity summary"
    ));
    assert!(help.contains(
        "reference-snapshot-batch-parity          Alias for reference-snapshot-batch-parity-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-mixed-time-scale-batch-parity-summary  Print the compact reference snapshot mixed TT/TDB batch parity summary"
    ));
    assert!(help.contains(
        "reference-snapshot-mixed-tt-tdb-batch-parity-summary  Alias for reference-snapshot-mixed-time-scale-batch-parity-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-mixed-tt-tdb-batch-parity  Alias for reference-snapshot-mixed-time-scale-batch-parity-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-equatorial-parity-summary  Print the compact reference snapshot equatorial parity summary"
    ));
    assert!(help.contains(
        "reference-snapshot-equatorial-parity     Alias for reference-snapshot-equatorial-parity-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-exact-j2000-evidence  Alias for reference-snapshot-exact-j2000-evidence-summary"
    ));
    assert!(help.contains(
        "exact-j2000-evidence    Alias for reference-snapshot-exact-j2000-evidence-summary"
    ));
    assert!(help.contains(
        "selected-asteroid-source-evidence-summary  Print the compact selected-asteroid source evidence summary"
    ));
    assert!(help.contains(
        "reference-snapshot-selected-asteroid-source-summary  Print the compact selected-asteroid source evidence summary"
    ));
    assert!(help.contains(
        "selected-asteroid-source-request-corpus-summary  Print the compact selected-asteroid source request corpus summary"
    ));
    assert!(help.contains(
        "selected-asteroid-source-request-corpus  Alias for selected-asteroid-source-request-corpus-summary"
    ));
    assert!(help.contains(
        "selected-asteroid-source-request-corpus-equatorial-summary  Print the compact selected-asteroid source request corpus summary in the equatorial frame"
    ));
    assert!(help.contains(
        "selected-asteroid-source-request-corpus-equatorial  Alias for selected-asteroid-source-request-corpus-equatorial-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-selected-asteroid-source-window-summary  Print the compact selected-asteroid source windows summary"
    ));
    assert!(help.contains(
        "reference-snapshot-selected-asteroid-source-window  Alias for reference-snapshot-selected-asteroid-source-window-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2453000-selected-asteroid-source-summary  Print the compact reference 2003-12-27 selected-asteroid source evidence summary"
    ));
    assert!(help.contains(
        "2453000-selected-asteroid-source-summary  Alias for reference-snapshot-2453000-selected-asteroid-source-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2500000-selected-asteroid-source-summary  Print the compact reference selected-asteroid 2500000 source evidence summary"
    ));
    assert!(help.contains(
        "2500000-selected-asteroid-source-summary  Alias for reference-snapshot-2500000-selected-asteroid-source-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-2634167-selected-asteroid-source-summary  Print the compact reference selected-asteroid 2634167 source evidence summary"
    ));
    assert!(help.contains(
        "2634167-selected-asteroid-source-summary  Alias for reference-snapshot-2634167-selected-asteroid-source-summary"
    ));
    assert!(help.contains(
        "reference-snapshot-selected-asteroid-dense-boundary-summary  Print the compact selected-asteroid dense boundary evidence summary"
    ));
    assert!(help.contains(
        "selected-asteroid-dense-boundary-summary  Alias for reference-snapshot-selected-asteroid-dense-boundary-summary"
    ));
    assert!(help.contains(
        "selected-asteroid-batch-parity-summary  Print the compact selected-asteroid batch-parity summary"
    ));
    assert!(help.contains(
        "selected-asteroid-batch-parity  Alias for selected-asteroid-batch-parity-summary"
    ));
    assert!(help.contains(
        "reference-asteroid-evidence-summary  Print the compact reference asteroid evidence summary"
    ));
    assert!(help.contains("reference-asteroid-equatorial-evidence-summary  Print the compact reference asteroid equatorial evidence summary"));
    assert!(help.contains("reference-asteroid-equatorial-evidence  Alias for reference-asteroid-equatorial-evidence-summary"));
    assert!(help.contains("reference-asteroid-source-window-summary  Print the compact reference asteroid source windows summary"));
    assert!(help.contains(
        "reference-asteroid-source-window  Alias for reference-asteroid-source-window-summary"
    ));
    assert!(help.contains(
        "reference-asteroid-source-summary  Alias for reference-asteroid-source-window-summary"
    ));
    assert!(help.contains("selected-asteroid-source-window-summary  Print the compact selected-asteroid source windows summary"));
    assert!(help.contains("reference-snapshot-2451917-selected-asteroid-source-summary  Print the compact reference selected-asteroid 2001-01-08 source evidence summary"));
    assert!(help.contains("2451917-selected-asteroid-source-summary  Alias for reference-snapshot-2451917-selected-asteroid-source-summary"));
    assert!(help.contains(
        "selected-asteroid-source-window  Alias for selected-asteroid-source-window-summary"
    ));
    assert!(help.contains("independent-holdout-source-window-summary  Print the compact independent hold-out source windows summary"));
    assert!(help.contains("independent-holdout-manifest-summary  Print the compact independent hold-out manifest summary"));
    assert!(help.contains(
        "independent-holdout-manifest            Alias for independent-holdout-manifest-summary"
    ));
    assert!(help.contains("independent-holdout-quarter-day-boundary-summary  Print the compact independent hold-out quarter-day boundary samples summary"));
    assert!(help.contains("independent-holdout-quarter-day-boundary  Alias for independent-holdout-quarter-day-boundary-summary"));
    assert!(help
        .contains("independent-holdout-summary  Print the compact independent hold-out summary"));
    assert!(help.contains(
        "independent-holdout-source-summary  Print the compact independent hold-out source summary"
    ));
    assert!(help.contains("independent-holdout-high-curvature-summary  Print the compact independent hold-out high-curvature evidence summary"));
    assert!(help.contains(
        "holdout-high-curvature-summary  Alias for independent-holdout-high-curvature-summary"
    ));
    assert!(
        help.contains("source-audit-summary      Print the compact VSOP87 source audit summary")
    );
    assert!(help.contains(
        "generated-binary-audit-summary  Print the compact VSOP87 generated binary audit summary"
    ));
    assert!(help.contains(
        "benchmark [--rounds N]    Benchmark the candidate backend on the representative 1600-2600 window corpus and full chart assembly on representative house scenarios"
    ));
}
