//! Help text generation.

use pleiades_core::{
    validated_apparentness_policy_summary_for_report, validated_delta_t_policy_summary_for_report,
    validated_frame_policy_summary_for_report, validated_native_sidereal_policy_summary_for_report,
    validated_observer_policy_summary_for_report, validated_request_policy_summary_for_report,
    validated_request_semantics_summary_for_report, validated_time_scale_policy_summary_for_report,
    validated_utc_convenience_policy_summary_for_report,
};
use pleiades_validate::render_cli as validate_render_cli;

use crate::cli::banner;

pub(crate) fn shared_request_policy_help_block() -> String {
    let request_policy = validated_request_policy_summary_for_report();
    let request_semantics = validated_request_semantics_summary_for_report();
    let time_scale_policy = validated_time_scale_policy_summary_for_report();
    let utc_convenience_policy = validated_utc_convenience_policy_summary_for_report();
    let delta_t_policy = validated_delta_t_policy_summary_for_report();
    let observer_policy = validated_observer_policy_summary_for_report();
    let apparentness_policy = validated_apparentness_policy_summary_for_report();
    let native_sidereal_policy = validated_native_sidereal_policy_summary_for_report();
    let frame_policy = validated_frame_policy_summary_for_report();

    format!(
        "  Request policy: {}\n  Request semantics summary: {}\n  Time-scale policy: {}\n  UTC convenience policy: {}\n  Delta T policy: {}\n  Observer policy: {}\n  Apparentness policy: {}\n  Native sidereal policy: {}\n  Frame policy: {}",
        request_policy,
        request_semantics,
        time_scale_policy,
        utc_convenience_policy,
        delta_t_policy,
        observer_policy,
        apparentness_policy,
        native_sidereal_policy,
        frame_policy,
    )
}

pub(crate) fn help_text() -> String {
    let validation_help = validate_render_cli(&["help"]).expect("validation help should render");
    let validation_commands = validation_help
        .split_once(
            "

Commands:
",
        )
        .map(|(_, tail)| tail)
        .unwrap_or_else(|| validation_help.as_str());
    let validation_commands = validation_commands
        .rsplit_once(
            "
  help                      Show this help text",
        )
        .map(|(commands, _)| commands)
        .unwrap_or(validation_commands);

    format!(
        "{}

Commands:
{}
  chart                  Render a basic chart report
    --tt|--tdb|--utc|--ut1  Tag the chart instant with a time scale
    --tt-offset-seconds <seconds>  Caller-supplied TT offset for UTC/UT1-tagged instants
    --tt-from-utc-offset-seconds <seconds>  Alias for --tt-offset-seconds when the chart instant is tagged as UTC
    --tt-from-ut1-offset-seconds <seconds>  Alias for --tt-offset-seconds when the chart instant is tagged as UT1
    --tdb-offset-seconds <seconds> Caller-supplied signed TDB-TT offset for TT/UTC/UT1-tagged instants
    --tdb-from-utc-offset-seconds <seconds> Explicit UTC-tagged alias for the signed TDB-TT offset
    --tdb-from-ut1-offset-seconds <seconds> Explicit UT1-tagged alias for the signed TDB-TT offset
    --tdb-from-tt-offset-seconds <seconds> Caller-supplied signed TDB-TT offset for TT-tagged instants
    --tt-from-tdb-offset-seconds <seconds> Caller-supplied signed TT-TDB offset for TDB-tagged instants
    --mean               Force mean positions for backend queries
    --apparent           Force apparent positions for backend queries
    --body <name>        Use a built-in body or a custom catalog:designation identifier
  generate-spk-corpus <kernel.bsp> <jd...>  Sample a JPL DE SPK kernel into the corpus CSV
  generate-spk-corpus <kernel.bsp> --emit-slices <out-dir>  Generate all four corpus slices (boundary, interior, fast_clusters, holdout) plus manifest.txt into <out-dir>
  generate-fixture-golden <out-dir>  Derive the de440-independent fixture_golden.csv from the checked-in Horizons reference snapshot
  generate-artifact <kernel.bsp> --out <path> [--start <year|JD>] [--end <year|JD>]  Regenerate the packaged artifact over a coverage window (default 1900-2100)
  {}
  help                   Show this help text",
        banner(),
        validation_commands,
        shared_request_policy_help_block(),
    )
}
