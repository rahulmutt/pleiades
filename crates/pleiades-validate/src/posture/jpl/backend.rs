//! Relocated backend-struct renderers (InterpolationQualitySample,
//! SnapshotManifestSummary) copied from `pleiades-jpl::backend` (Slice D).

use pleiades_jpl::{InterpolationQualitySample, SnapshotManifestSummary};

/// Compact release-facing summary line for one interpolation-quality sample.
/// Verbatim copy of `InterpolationQualitySample::summary_line` (backend.rs:114).
pub(crate) fn interpolation_quality_sample_summary_line(s: &InterpolationQualitySample) -> String {
    format!(
        "{} at {}: {} interpolation, bracket span {:.1} d, |Δlon|={:.12}°, |Δlat|={:.12}°, |Δdist|={:.12} AU",
        s.body,
        s.epoch.summary_line(), // Instant::summary_line (pleiades-time) — NOT moved, stays
        s.interpolation_kind.label(),
        s.bracket_span_days,
        s.longitude_error_deg,
        s.latitude_error_deg,
        s.distance_error_au,
    )
}

/// Compact release-facing summary line for a manifest summary wrapper.
///
/// Verbatim copy of the rendering reached by
/// `SnapshotManifestSummary::summary_line` (backend.rs:798), which delegates
/// to `SnapshotManifest::summary_line_with_defaults` (backend.rs:549). The
/// source/coverage derivations call the `pub` data accessors
/// `SnapshotManifest::source_or`/`coverage_or` (backend.rs:433/438) directly —
/// exactly as jpl's `summary_line_with_defaults` does (backend.rs:562-563);
/// those accessors stay in jpl. The `columns` logic (`pub(crate)`
/// `columns_summary`, not callable cross-crate) and the title/redistribution
/// trim logic (matching the private `trimmed_or` helper) are inlined here
/// reading the struct's public fields directly. Does NOT copy
/// `validate()`/`validated_summary_line()` gate logic — that stays in jpl.
pub(crate) fn snapshot_manifest_summary_line(s: &SnapshotManifestSummary) -> String {
    let manifest = &s.manifest;
    let label = s.label;
    let source_fallback = s.source_fallback;
    let coverage_fallback = s.coverage_fallback;

    let title = manifest
        .title
        .as_deref()
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .unwrap_or("unknown");
    let source = manifest.source_or(source_fallback);
    let coverage = manifest.coverage_or(coverage_fallback);
    let columns = if manifest.columns.is_empty() {
        "none".to_string()
    } else {
        manifest.columns.join(", ")
    };
    let mut text =
        format!("{label}: {title}; source={source}; coverage={coverage}; columns={columns}");
    if let Some(redistribution) = manifest
        .redistribution
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        text.push_str("; redistribution=");
        text.push_str(redistribution);
    }
    text
}

#[cfg(test)]
mod golden {
    use pleiades_jpl::{
        independent_holdout_snapshot_manifest, interpolation_quality_sample_list,
        reference_snapshot_manifest, SnapshotManifestSummary,
    };

    // jpl's inherent renderer (`InterpolationQualitySample::summary_line`,
    // `SnapshotManifestSummary::summary_line`) was deleted in the Task 14
    // contract sweep. `EXPECTED_*` below are byte-exact captures of that
    // renderer's output taken immediately before deletion (Slice D Task
    // 14a); this still fails closed on any drift in the validate copy, just
    // pinned to a literal instead of a live jpl call. Only the jpl DATA
    // accessors (`interpolation_quality_sample_list`,
    // `reference_snapshot_manifest`, `independent_holdout_snapshot_manifest`)
    // remain referenced here.
    const EXPECTED_INTERPOLATION_QUALITY_SAMPLE_LINES: &str = r"Sun at JD 2451545 TDB: cubic interpolation, bracket span 36890.0 d, |Δlon|=3.634581158194°, |Δlat|=0.106219187481°, |Δdist|=8040143.062837206759 AU
Sun at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=176.051193436505°, |Δlat|=0.124214630231°, |Δdist|=4.894920066915 AU
Sun at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=1.475549934769°, |Δlat|=0.066482211715°, |Δdist|=1.632702544666 AU
Sun at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=81.036390382646°, |Δlat|=4.473652598106°, |Δdist|=0.959940989727 AU
Sun at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=58.817100566331°, |Δlat|=15.640023257853°, |Δdist|=0.976530809023 AU
Sun at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.057288018277°, |Δlat|=0.033126797453°, |Δdist|=0.233315879340 AU
Sun at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.086449912659°, |Δlat|=0.013269670688°, |Δdist|=0.139981599743 AU
Sun at JD 2451915 TDB: cubic interpolation, bracket span 0.8 d, |Δlon|=0.000000012186°, |Δlat|=0.000000001126°, |Δdist|=0.000000000158 AU
Sun at JD 2451915.25 TDB: cubic interpolation, bracket span 0.5 d, |Δlon|=0.000000006073°, |Δlat|=0.000000000563°, |Δdist|=0.000000000080 AU
Sun at JD 2451915.5 TDB: cubic interpolation, bracket span 0.5 d, |Δlon|=0.000000004150°, |Δlat|=0.000000000384°, |Δdist|=0.000000000047 AU
Sun at JD 2451915.75 TDB: cubic interpolation, bracket span 0.5 d, |Δlon|=0.000000006207°, |Δlat|=0.000000000575°, |Δdist|=0.000000000071 AU
Sun at JD 2451916 TDB: cubic interpolation, bracket span 0.8 d, |Δlon|=0.000000012598°, |Δlat|=0.000000001164°, |Δdist|=0.000000000121 AU
Sun at JD 2451916.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000050441°, |Δlat|=0.000000004640°, |Δdist|=0.000000000381 AU
Sun at JD 2451917 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000251207°, |Δlat|=0.000000023198°, |Δdist|=0.000000001944 AU
Sun at JD 2451918.5 TDB: cubic interpolation, bracket span 2.5 d, |Δlon|=0.000000819080°, |Δlat|=0.000000064223°, |Δdist|=0.000000008821 AU
Sun at JD 2451919.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000001035076°, |Δlat|=0.000000080276°, |Δdist|=0.000000010712 AU
Sun at JD 2451920.5 TDB: cubic interpolation, bracket span 1081.0 d, |Δlon|=0.000003904938°, |Δlat|=0.000000299685°, |Δdist|=0.000000038806 AU
Moon at JD 2451545 TDB: cubic interpolation, bracket span 36890.0 d, |Δlon|=61.903954788758°, |Δlat|=27.907605668674°, |Δdist|=8038683.994324577972 AU
Moon at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=116.513470497044°, |Δlat|=27.472390335047°, |Δdist|=6.854597844570 AU
Moon at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=75.568044341933°, |Δlat|=17.603741870230°, |Δdist|=1.631011580750 AU
Moon at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=91.690570226503°, |Δlat|=28.023202076212°, |Δdist|=0.976992997025 AU
Moon at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=78.869465189464°, |Δlat|=27.923081373622°, |Δdist|=0.977562751030 AU
Moon at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=71.817672318457°, |Δlat|=27.675975546099°, |Δdist|=0.231355293936 AU
Moon at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=113.178323285782°, |Δlat|=18.130434951550°, |Δdist|=0.136576090325 AU
Moon at JD 2451915 TDB: cubic interpolation, bracket span 0.8 d, |Δlon|=0.000141008841°, |Δlat|=0.000006700752°, |Δdist|=0.000000016296 AU
Moon at JD 2451915.25 TDB: cubic interpolation, bracket span 0.5 d, |Δlon|=0.000081803673°, |Δlat|=0.000004266433°, |Δdist|=0.000000007949 AU
Moon at JD 2451915.5 TDB: cubic interpolation, bracket span 0.5 d, |Δlon|=0.000053958257°, |Δlat|=0.000003057879°, |Δdist|=0.000000005578 AU
Moon at JD 2451915.75 TDB: cubic interpolation, bracket span 0.5 d, |Δlon|=0.000092829258°, |Δlat|=0.000005663709°, |Δdist|=0.000000008136 AU
Moon at JD 2451916 TDB: cubic interpolation, bracket span 0.8 d, |Δlon|=0.000182854059°, |Δlat|=0.000011917295°, |Δdist|=0.000000017138 AU
Moon at JD 2451916.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000795670024°, |Δlat|=0.000057958267°, |Δdist|=0.000000070994 AU
Moon at JD 2451917 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.005021899342°, |Δlat|=0.000399338206°, |Δdist|=0.000000330575 AU
Moon at JD 2451918.5 TDB: cubic interpolation, bracket span 2.5 d, |Δlon|=0.012816315754°, |Δlat|=0.001169249555°, |Δdist|=0.000002960337 AU
Moon at JD 2451919.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.038649752070°, |Δlat|=0.003555974624°, |Δdist|=0.000003394953 AU
Moon at JD 2451920.5 TDB: cubic interpolation, bracket span 1081.0 d, |Δlon|=0.220192466827°, |Δlat|=0.018928043873°, |Δdist|=0.000010614634 AU
Mercury at JD 2451545 TDB: cubic interpolation, bracket span 36890.0 d, |Δlon|=11.431261839556°, |Δlat|=8.943221746738°, |Δdist|=8524651.303514530882 AU
Mercury at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=178.825501552118°, |Δlat|=8.359845997309°, |Δdist|=4.458025893786 AU
Mercury at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=1.407460378624°, |Δlat|=5.485129496235°, |Δdist|=1.719117960103 AU
Mercury at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=11.180350792967°, |Δlat|=24.491535052763°, |Δdist|=0.974019148648 AU
Mercury at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=15.527249491306°, |Δlat|=24.387153955477°, |Δdist|=0.964616251227 AU
Mercury at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=1.410334503566°, |Δlat|=2.133582709876°, |Δdist|=0.240792877735 AU
Mercury at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.712283673342°, |Δlat|=0.963193465087°, |Δdist|=0.145238771914 AU
Mercury at JD 2451915 TDB: cubic interpolation, bracket span 0.8 d, |Δlon|=0.000000384448°, |Δlat|=0.000000003025°, |Δdist|=0.000000008218 AU
Mercury at JD 2451915.25 TDB: cubic interpolation, bracket span 0.5 d, |Δlon|=0.000000193741°, |Δlat|=0.000000001555°, |Δdist|=0.000000004075 AU
Mercury at JD 2451915.5 TDB: cubic interpolation, bracket span 0.5 d, |Δlon|=0.000000132266°, |Δlat|=0.000000001348°, |Δdist|=0.000000002815 AU
Mercury at JD 2451915.75 TDB: cubic interpolation, bracket span 0.5 d, |Δlon|=0.000000199979°, |Δlat|=0.000000002069°, |Δdist|=0.000000004188 AU
Mercury at JD 2451916 TDB: cubic interpolation, bracket span 0.8 d, |Δlon|=0.000000409511°, |Δlat|=0.000000005156°, |Δdist|=0.000000008684 AU
Mercury at JD 2451916.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000001694771°, |Δlat|=0.000000026120°, |Δdist|=0.000000036002 AU
Mercury at JD 2451917 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000008612155°, |Δlat|=0.000000135386°, |Δdist|=0.000000177062 AU
Mercury at JD 2451918.5 TDB: cubic interpolation, bracket span 2.5 d, |Δlon|=0.000064372593°, |Δlat|=0.000002260066°, |Δdist|=0.000001534394 AU
Mercury at JD 2451919.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000083498783°, |Δlat|=0.000002956363°, |Δdist|=0.000001862143 AU
Mercury at JD 2451920.5 TDB: cubic interpolation, bracket span 1081.0 d, |Δlon|=0.000323195716°, |Δlat|=0.000011590734°, |Δdist|=0.000006737322 AU
Venus at JD 2451545 TDB: cubic interpolation, bracket span 36890.0 d, |Δlon|=43.017271475583°, |Δlat|=15.502483698467°, |Δdist|=7943230.719248073176 AU
Venus at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=132.456746915984°, |Δlat|=15.983452160694°, |Δdist|=5.411364183966 AU
Venus at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=29.126991037538°, |Δlat|=8.647849007845°, |Δdist|=1.454143468302 AU
Venus at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=79.656565136249°, |Δlat|=18.420204457245°, |Δdist|=0.089145702691 AU
Venus at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=79.399575047407°, |Δlat|=18.121162658815°, |Δdist|=0.068617420206 AU
Venus at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=14.510392103770°, |Δlat|=4.478672734024°, |Δdist|=0.133746603693 AU
Venus at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=6.433364261240°, |Δlat|=1.962072938505°, |Δdist|=0.098866589232 AU
Venus at JD 2451915 TDB: cubic interpolation, bracket span 0.8 d, |Δlon|=0.000000065962°, |Δlat|=0.000000002382°, |Δdist|=0.000000000410 AU
Venus at JD 2451915.25 TDB: cubic interpolation, bracket span 0.5 d, |Δlon|=0.000000032987°, |Δlat|=0.000000001195°, |Δdist|=0.000000000207 AU
Venus at JD 2451915.5 TDB: cubic interpolation, bracket span 0.5 d, |Δlon|=0.000000022493°, |Δlat|=0.000000000809°, |Δdist|=0.000000000134 AU
Venus at JD 2451915.75 TDB: cubic interpolation, bracket span 0.5 d, |Δlon|=0.000000033750°, |Δlat|=0.000000001218°, |Δdist|=0.000000000203 AU
Venus at JD 2451916 TDB: cubic interpolation, bracket span 0.8 d, |Δlon|=0.000000069017°, |Δlat|=0.000000002461°, |Δdist|=0.000000000391 AU
Venus at JD 2451916.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000283106°, |Δlat|=0.000000009913°, |Δdist|=0.000000001494 AU
Venus at JD 2451917 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000001417027°, |Δlat|=0.000000049921°, |Δdist|=0.000000007646 AU
Venus at JD 2451918.5 TDB: cubic interpolation, bracket span 2.5 d, |Δlon|=0.000009757323°, |Δlat|=0.000000227539°, |Δdist|=0.000000020598 AU
Venus at JD 2451919.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000012278863°, |Δlat|=0.000000286526°, |Δdist|=0.000000028695 AU
Venus at JD 2451920.5 TDB: cubic interpolation, bracket span 1081.0 d, |Δlon|=0.000046139500°, |Δlat|=0.000001080589°, |Δdist|=0.000000118042 AU
Jupiter at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=25.504145317200°, |Δlat|=59.916609967121°, |Δdist|=11.044809529242 AU
Jupiter at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=12.703331391062°, |Δlat|=40.310578798191°, |Δdist|=0.612062661726 AU
Jupiter at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=5.921297300151°, |Δlat|=22.040745622382°, |Δdist|=0.771330859967 AU
Jupiter at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=5.914696763654°, |Δlat|=21.996269064051°, |Δdist|=0.770079262316 AU
Jupiter at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=3.096927420610°, |Δlat|=11.919361469034°, |Δdist|=0.294039688174 AU
Jupiter at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=1.098411051006°, |Δlat|=4.263854669176°, |Δdist|=0.050462490405 AU
Jupiter at JD 2451915 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000177924°, |Δlat|=0.000000086584°, |Δdist|=0.000000120733 AU
Jupiter at JD 2451915.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000001542939°, |Δlat|=0.000000021550°, |Δdist|=0.000000041701 AU
Jupiter at JD 2451916 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000895731°, |Δlat|=0.000000072558°, |Δdist|=0.000000102253 AU
Jupiter at JD 2451916.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000001342912°, |Δlat|=0.000000108609°, |Δdist|=0.000000153325 AU
Jupiter at JD 2451917 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000001437290°, |Δlat|=0.000000226298°, |Δdist|=0.000000331439 AU
Jupiter at JD 2451918.5 TDB: cubic interpolation, bracket span 2.5 d, |Δlon|=0.000002581350°, |Δlat|=0.000000036609°, |Δdist|=0.000000016606 AU
Jupiter at JD 2451919.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000003217442°, |Δlat|=0.000000045655°, |Δdist|=0.000000020986 AU
Jupiter at JD 2451920.5 TDB: cubic interpolation, bracket span 1081.0 d, |Δlon|=0.000011976973°, |Δlat|=0.000000170032°, |Δdist|=0.000000079150 AU
Mars at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=95.668774860143°, |Δlat|=3.609438931611°, |Δdist|=4.613095680426 AU
Mars at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=32.490870542512°, |Δlat|=1.003497091331°, |Δdist|=1.033230088124 AU
Mars at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=32.701194047482°, |Δlat|=1.404985899434°, |Δdist|=0.099945587028 AU
Mars at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=32.941824306671°, |Δlat|=1.414040809531°, |Δdist|=0.108142935154 AU
Mars at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=15.971861753682°, |Δlat|=0.661679843768°, |Δdist|=0.126394095456 AU
Mars at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=4.698501346433°, |Δlat|=0.178316405542°, |Δdist|=0.070002171036 AU
Mars at JD 2451915 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000039419°, |Δlat|=0.000000002759°, |Δdist|=0.000000000628 AU
Mars at JD 2451915.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000035306°, |Δlat|=0.000000002914°, |Δdist|=0.000000000740 AU
Mars at JD 2451916 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000029746°, |Δlat|=0.000000002823°, |Δdist|=0.000000000828 AU
Mars at JD 2451916.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000044949°, |Δlat|=0.000000004253°, |Δdist|=0.000000001236 AU
Mars at JD 2451917 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000091346°, |Δlat|=0.000000011874°, |Δdist|=0.000000004033 AU
Mars at JD 2451918.5 TDB: cubic interpolation, bracket span 2.5 d, |Δlon|=0.000000148266°, |Δlat|=0.000000024347°, |Δdist|=0.000000015456 AU
Mars at JD 2451919.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000179837°, |Δlat|=0.000000030608°, |Δdist|=0.000000019375 AU
Mars at JD 2451920.5 TDB: cubic interpolation, bracket span 1081.0 d, |Δlon|=0.000000650619°, |Δlat|=0.000000114938°, |Δdist|=0.000000072528 AU
Neptune at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=38.088537442546°, |Δlat|=76.097249521556°, |Δdist|=38.785352949131 AU
Neptune at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=4.053601583792°, |Δlat|=24.526676102872°, |Δdist|=7.840522630079 AU
Neptune at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=3.010387358958°, |Δlat|=18.749506979073°, |Δdist|=0.910491794992 AU
Neptune at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=3.012743831034°, |Δlat|=18.744890415981°, |Δdist|=0.909848103846 AU
Neptune at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=1.444270713106°, |Δlat|=9.233723056066°, |Δdist|=0.872780590239 AU
Neptune at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.456011245105°, |Δlat|=2.935228262374°, |Δdist|=0.466534121537 AU
Neptune at JD 2451915 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000008510°, |Δlat|=0.000000008564°, |Δdist|=0.000000005119 AU
Neptune at JD 2451915.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000000589°, |Δlat|=0.000000008641°, |Δdist|=0.000000004681 AU
Neptune at JD 2451916 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000004007°, |Δlat|=0.000000008685°, |Δdist|=0.000000005141 AU
Neptune at JD 2451916.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000006014°, |Δlat|=0.000000013027°, |Δdist|=0.000000007711 AU
Neptune at JD 2451917 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000040670°, |Δlat|=0.000000010211°, |Δdist|=0.000000006452 AU
Neptune at JD 2451918.5 TDB: cubic interpolation, bracket span 2.5 d, |Δlon|=0.000000021052°, |Δlat|=0.000000182468°, |Δdist|=0.000000088571 AU
Neptune at JD 2451919.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000026443°, |Δlat|=0.000000228048°, |Δdist|=0.000000110705 AU
Neptune at JD 2451920.5 TDB: cubic interpolation, bracket span 1081.0 d, |Δlon|=0.000000099196°, |Δlat|=0.000000851251°, |Δdist|=0.000000413267 AU
Pluto at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=8.833636859803°, |Δlat|=82.892429935290°, |Δdist|=49.532146124898 AU
Pluto at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=1.565310553126°, |Δlat|=27.689487853854°, |Δdist|=9.826313111645 AU
Pluto at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=1.020461269568°, |Δlat|=22.137139714736°, |Δdist|=0.828995716896 AU
Pluto at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=1.019611728823°, |Δlat|=22.143061391861°, |Δdist|=0.828999233936 AU
Pluto at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.501416777211°, |Δlat|=10.921942580198°, |Δdist|=0.988129196032 AU
Pluto at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.163555436081°, |Δlat|=3.443390235351°, |Δdist|=0.569445900644 AU
Pluto at JD 2451915 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000027896°, |Δlat|=0.000000224289°, |Δdist|=0.000000050963 AU
Pluto at JD 2451915.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000040735°, |Δlat|=0.000000167863°, |Δdist|=0.000000094916 AU
Pluto at JD 2451916 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000100256°, |Δlat|=0.000000071740°, |Δdist|=0.000000116385 AU
Pluto at JD 2451916.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000150506°, |Δlat|=0.000000107622°, |Δdist|=0.000000174554 AU
Pluto at JD 2451917 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000612848°, |Δlat|=0.000000362331°, |Δdist|=0.000000433316 AU
Pluto at JD 2451918.5 TDB: cubic interpolation, bracket span 2.5 d, |Δlon|=0.000000057392°, |Δlat|=0.000004324278°, |Δdist|=0.000001466570 AU
Pluto at JD 2451919.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000069252°, |Δlat|=0.000005407113°, |Δdist|=0.000001833140 AU
Pluto at JD 2451920.5 TDB: cubic interpolation, bracket span 1081.0 d, |Δlon|=0.000000249331°, |Δlat|=0.000020193332°, |Δdist|=0.000006843431 AU
Saturn at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=17.439126382284°, |Δlat|=66.539484975822°, |Δdist|=15.582349709964 AU
Saturn at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=5.261346807881°, |Δlat|=32.259512835287°, |Δdist|=1.388506963584 AU
Saturn at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=2.949094350907°, |Δlat|=20.204962452334°, |Δdist|=0.666406116491 AU
Saturn at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=2.945907398958°, |Δlat|=20.177862033754°, |Δdist|=0.665559237177 AU
Saturn at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=1.491603993158°, |Δlat|=10.470665892788°, |Δdist|=0.194394524114 AU
Saturn at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.505464497060°, |Δlat|=3.549665236945°, |Δdist|=0.000683671457 AU
Saturn at JD 2451915 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000004086°, |Δlat|=0.000000003084°, |Δdist|=0.000000000108 AU
Saturn at JD 2451915.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000015544°, |Δlat|=0.000000007394°, |Δdist|=0.000000001874 AU
Saturn at JD 2451916 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000029692°, |Δlat|=0.000000000745°, |Δdist|=0.000000002300 AU
Saturn at JD 2451916.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000044496°, |Δlat|=0.000000001116°, |Δdist|=0.000000003452 AU
Saturn at JD 2451917 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000003596°, |Δlat|=0.000000036349°, |Δdist|=0.000000012694 AU
Saturn at JD 2451918.5 TDB: cubic interpolation, bracket span 2.5 d, |Δlon|=0.000000001621°, |Δlat|=0.000000003840°, |Δdist|=0.000000014092 AU
Saturn at JD 2451919.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000002083°, |Δlat|=0.000000004783°, |Δdist|=0.000000017615 AU
Saturn at JD 2451920.5 TDB: cubic interpolation, bracket span 1081.0 d, |Δlon|=0.000000007970°, |Δlat|=0.000000017792°, |Δdist|=0.000000065762 AU
Uranus at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=45.455163919423°, |Δlat|=67.907234519722°, |Δdist|=17.433944369328 AU
Uranus at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=5.788754567829°, |Δlat|=19.846686608386°, |Δdist|=4.099835675335 AU
Uranus at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=4.236686647302°, |Δlat|=14.686871848151°, |Δdist|=0.797639033068 AU
Uranus at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=4.238203229960°, |Δlat|=14.678488782842°, |Δdist|=0.796740235417 AU
Uranus at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=2.039600792718°, |Δlat|=7.198236284446°, |Δdist|=0.578337008093 AU
Uranus at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.646631804580°, |Δlat|=2.297019495577°, |Δdist|=0.268155639032 AU
Uranus at JD 2451915 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000003889°, |Δlat|=0.000000010311°, |Δdist|=0.000000000829 AU
Uranus at JD 2451915.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000002566°, |Δlat|=0.000000001593°, |Δdist|=0.000000001806 AU
Uranus at JD 2451916 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000001852°, |Δlat|=0.000000009154°, |Δdist|=0.000000000238 AU
Uranus at JD 2451916.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000002777°, |Δlat|=0.000000013727°, |Δdist|=0.000000000357 AU
Uranus at JD 2451917 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000028159°, |Δlat|=0.000000008697°, |Δdist|=0.000000016491 AU
Uranus at JD 2451918.5 TDB: cubic interpolation, bracket span 2.5 d, |Δlon|=0.000000049575°, |Δlat|=0.000000203173°, |Δdist|=0.000000019974 AU
Uranus at JD 2451919.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000062003°, |Δlat|=0.000000253861°, |Δdist|=0.000000024947 AU
Uranus at JD 2451920.5 TDB: cubic interpolation, bracket span 1081.0 d, |Δlon|=0.000000231607°, |Δlat|=0.000000947365°, |Δdist|=0.000000093056 AU
Ceres at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=0.000001621985°, |Δlat|=0.000000121700°, |Δdist|=0.000000074859 AU
Ceres at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000384685°, |Δlat|=0.000000028969°, |Δdist|=0.000000017998 AU
Ceres at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000229906°, |Δlat|=0.000000017375°, |Δdist|=0.000000010903 AU
Ceres at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000000064261°, |Δlat|=0.000000004977°, |Δdist|=0.000000001886 AU
Ceres at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000032101°, |Δlat|=0.000000002490°, |Δdist|=0.000000000950 AU
Ceres at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000021922°, |Δlat|=0.000000001724°, |Δdist|=0.000000000473 AU
Ceres at JD 2451915 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000032867°, |Δlat|=0.000000002589°, |Δdist|=0.000000000717 AU
Ceres at JD 2451915.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000000065600°, |Δlat|=0.000000005250°, |Δdist|=0.000000000860 AU
Ceres at JD 2451916.5 TDB: cubic interpolation, bracket span 3.0 d, |Δlon|=0.000000482280°, |Δlat|=0.000000039632°, |Δdist|=0.000000000674 AU
Ceres at JD 2451918.5 TDB: cubic interpolation, bracket span 3.0 d, |Δlon|=0.000001312767°, |Δlat|=0.000000111452°, |Δdist|=0.000000028856 AU
Ceres at JD 2451919.5 TDB: cubic interpolation, bracket span 1082.0 d, |Δlon|=0.000003389443°, |Δlat|=0.000000288247°, |Δdist|=0.000000072714 AU
Ceres at JD 2453000.5 TDB: cubic interpolation, bracket span 48080.5 d, |Δlon|=78.187070982423°, |Δlat|=7.358773674855°, |Δdist|=1062.507596625901 AU
Ceres at JD 2500000 TDB: cubic interpolation, bracket span 181166.5 d, |Δlon|=172.582350874973°, |Δlat|=0.290551633336°, |Δdist|=16056479.287124764174 AU
Pallas at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=0.000002378968°, |Δlat|=0.000000255505°, |Δdist|=0.000000023511 AU
Pallas at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000567452°, |Δlat|=0.000000062530°, |Δdist|=0.000000005778 AU
Pallas at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000341092°, |Δlat|=0.000000038548°, |Δdist|=0.000000003574 AU
Pallas at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000000086471°, |Δlat|=0.000000004658°, |Δdist|=0.000000000007 AU
Pallas at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000043307°, |Δlat|=0.000000002381°, |Δdist|=0.000000000003 AU
Pallas at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000028227°, |Δlat|=0.000000000683°, |Δdist|=0.000000000150 AU
Pallas at JD 2451915 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000042426°, |Δlat|=0.000000001069°, |Δdist|=0.000000000218 AU
Pallas at JD 2451915.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000000080273°, |Δlat|=0.000000000982°, |Δdist|=0.000000000926 AU
Pallas at JD 2451916.5 TDB: cubic interpolation, bracket span 3.0 d, |Δlon|=0.000000536050°, |Δlat|=0.000000046271°, |Δdist|=0.000000012833 AU
Pallas at JD 2451918.5 TDB: cubic interpolation, bracket span 3.0 d, |Δlon|=0.000001249551°, |Δlat|=0.000000284462°, |Δdist|=0.000000058036 AU
Pallas at JD 2451919.5 TDB: cubic interpolation, bracket span 1082.0 d, |Δlon|=0.000003244791°, |Δlat|=0.000000733274°, |Δdist|=0.000000148127 AU
Pallas at JD 2453000.5 TDB: cubic interpolation, bracket span 48080.5 d, |Δlon|=179.218062830799°, |Δlat|=27.987857088489°, |Δdist|=1062.226629163986 AU
Pallas at JD 2500000 TDB: cubic interpolation, bracket span 181166.5 d, |Δlon|=97.735388600524°, |Δlat|=10.680576356669°, |Δdist|=15204272.270293446258 AU
Juno at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=0.000001232825°, |Δlat|=0.000000044866°, |Δdist|=0.000000116648 AU
Juno at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000297373°, |Δlat|=0.000000010138°, |Δdist|=0.000000027670 AU
Juno at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000180726°, |Δlat|=0.000000005764°, |Δdist|=0.000000016538 AU
Juno at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000000025519°, |Δlat|=0.000000002612°, |Δdist|=0.000000004403 AU
Juno at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000012929°, |Δlat|=0.000000001283°, |Δdist|=0.000000002199 AU
Juno at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000005073°, |Δlat|=0.000000001027°, |Δdist|=0.000000001470 AU
Juno at JD 2451915 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000007787°, |Δlat|=0.000000001518°, |Δdist|=0.000000002203 AU
Juno at JD 2451915.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000000003419°, |Δlat|=0.000000003511°, |Δdist|=0.000000004295 AU
Juno at JD 2451916.5 TDB: cubic interpolation, bracket span 3.0 d, |Δlon|=0.000000122491°, |Δlat|=0.000000031317°, |Δdist|=0.000000030364 AU
Juno at JD 2451918.5 TDB: cubic interpolation, bracket span 3.0 d, |Δlon|=0.000000894614°, |Δlat|=0.000000102669°, |Δdist|=0.000000078267 AU
Juno at JD 2451919.5 TDB: cubic interpolation, bracket span 1082.0 d, |Δlon|=0.000002259675°, |Δlat|=0.000000258807°, |Δdist|=0.000000202125 AU
Juno at JD 2453000.5 TDB: cubic interpolation, bracket span 48080.5 d, |Δlon|=67.246945137174°, |Δlat|=9.678379301622°, |Δdist|=1126.737535855378 AU
Juno at JD 2500000 TDB: cubic interpolation, bracket span 181166.5 d, |Δlon|=168.403848647583°, |Δlat|=8.991292942619°, |Δdist|=17603418.447958711535 AU
Vesta at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=0.000000668272°, |Δlat|=0.000000100066°, |Δdist|=0.000000122746 AU
Vesta at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000163142°, |Δlat|=0.000000023620°, |Δdist|=0.000000029160 AU
Vesta at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000100288°, |Δlat|=0.000000014049°, |Δdist|=0.000000017456 AU
Vesta at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000000006889°, |Δlat|=0.000000002420°, |Δdist|=0.000000004479 AU
Vesta at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000003614°, |Δlat|=0.000000001206°, |Δdist|=0.000000002239 AU
Vesta at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000000721°, |Δlat|=0.000000000593°, |Δdist|=0.000000001469 AU
Vesta at JD 2451915 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000000908°, |Δlat|=0.000000000888°, |Δdist|=0.000000002203 AU
Vesta at JD 2451915.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000000012140°, |Δlat|=0.000000001031°, |Δdist|=0.000000004204 AU
Vesta at JD 2451916.5 TDB: cubic interpolation, bracket span 3.0 d, |Δlon|=0.000000215720°, |Δlat|=0.000000001446°, |Δdist|=0.000000028543 AU
Vesta at JD 2451918.5 TDB: cubic interpolation, bracket span 3.0 d, |Δlon|=0.000001071913°, |Δlat|=0.000000037209°, |Δdist|=0.000000068656 AU
Vesta at JD 2451919.5 TDB: cubic interpolation, bracket span 1082.0 d, |Δlon|=0.000002720568°, |Δlat|=0.000000093543°, |Δdist|=0.000000177696 AU
Vesta at JD 2453000.5 TDB: cubic interpolation, bracket span 48080.5 d, |Δlon|=80.889905744435°, |Δlat|=1.132811528511°, |Δdist|=1086.436413697026 AU
Vesta at JD 2500000 TDB: cubic interpolation, bracket span 181166.5 d, |Δlon|=166.561893803484°, |Δlat|=2.466999360285°, |Δdist|=17618636.143089406192 AU
asteroid:433-Eros at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=0.000001518738°, |Δlat|=0.000000621762°, |Δdist|=0.000000124336 AU
asteroid:433-Eros at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000369959°, |Δlat|=0.000000147329°, |Δdist|=0.000000029471 AU
asteroid:433-Eros at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000226970°, |Δlat|=0.000000087965°, |Δdist|=0.000000017601 AU
asteroid:433-Eros at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000000030645°, |Δlat|=0.000000024533°, |Δdist|=0.000000004594 AU
asteroid:433-Eros at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000015665°, |Δlat|=0.000000012246°, |Δdist|=0.000000002294 AU
asteroid:433-Eros at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000005816°, |Δlat|=0.000000008376°, |Δdist|=0.000000001521 AU
asteroid:433-Eros at JD 2451915 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000009072°, |Δlat|=0.000000012548°, |Δdist|=0.000000002279 AU
asteroid:433-Eros at JD 2451915.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000000002460°, |Δlat|=0.000000025130°, |Δdist|=0.000000004410 AU
asteroid:433-Eros at JD 2451916.5 TDB: cubic interpolation, bracket span 3.0 d, |Δlon|=0.000000168638°, |Δlat|=0.000000186569°, |Δdist|=0.000000030911 AU
asteroid:433-Eros at JD 2451918.5 TDB: cubic interpolation, bracket span 3.0 d, |Δlon|=0.000001143909°, |Δlat|=0.000000517887°, |Δdist|=0.000000079532 AU
asteroid:433-Eros at JD 2451919.5 TDB: cubic interpolation, bracket span 1082.0 d, |Δlon|=0.000002870449°, |Δlat|=0.000001333548°, |Δdist|=0.000000205605 AU
asteroid:433-Eros at JD 2453000.5 TDB: cubic interpolation, bracket span 48080.5 d, |Δlon|=81.899642420790°, |Δlat|=0.191743730422°, |Δdist|=1255.284945444964 AU
asteroid:433-Eros at JD 2500000 TDB: cubic interpolation, bracket span 181166.5 d, |Δlon|=28.623416023889°, |Δlat|=2.947118967681°, |Δdist|=18591789.124255102128 AU
asteroid:99942-Apophis at JD 2451915 TDB: cubic interpolation, bracket span 370.5 d, |Δlon|=169.905378713881°, |Δlat|=0.836749413373°, |Δdist|=0.779512497811 AU
asteroid:99942-Apophis at JD 2451915.5 TDB: cubic interpolation, bracket span 3.5 d, |Δlon|=160.372862582780°, |Δlat|=0.317422248225°, |Δdist|=1.389940360057 AU
asteroid:99942-Apophis at JD 2451918.5 TDB: cubic interpolation, bracket span 4.0 d, |Δlon|=0.267237124786°, |Δlat|=0.263828504787°, |Δdist|=3.513610058333 AU
asteroid:99942-Apophis at JD 2451919.5 TDB: cubic interpolation, bracket span 1082.0 d, |Δlon|=177.875283504251°, |Δlat|=1.345378453694°, |Δdist|=2.661350752902 AU
asteroid:99942-Apophis at JD 2453000.5 TDB: cubic interpolation, bracket span 48080.5 d, |Δlon|=173.447596041720°, |Δlat|=0.544778564171°, |Δdist|=420691121.400366902351 AU
asteroid:99942-Apophis at JD 2500000 TDB: cubic interpolation, bracket span 181166.5 d, |Δlon|=163.519174261286°, |Δlat|=0.878285229676°, |Δdist|=30376623.745747849345 AU";

    #[test]
    fn interpolation_quality_sample_lines_byte_identical() {
        let after = interpolation_quality_sample_list()
            .iter()
            .map(super::interpolation_quality_sample_summary_line)
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(after, EXPECTED_INTERPOLATION_QUALITY_SAMPLE_LINES);
    }

    const EXPECTED_REFERENCE_SNAPSHOT_MANIFEST_SUMMARY_LINE: &str = r"Reference snapshot: JPL Horizons reference snapshot.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.; coverage=major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; selected bodies sampled at 1900-01-01 for Sun, Moon, Mercury, Venus; selected bodies sampled at 2451915.25 and 2451915.75 for Sun, Moon, Mercury, Venus; major bodies sampled at 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451916.0, 2451916.5, 2451917.0, 2451917.5, 2451918.5, 2451919.5, 2451920.5, and 2453000.5; major bodies sampled at 2451915.5 for Sun through Pluto; major bodies sampled at 2451913.5 through 2451917.5 for additional boundary coverage; selected asteroids sampled at J2000, 2378498.5, 2451910.5 through 2451919.5, with 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451917.5, 2451918.5, and 2451919.5 boundary coverage, 2003-12-27, 2132-08-31, 2500-01-01, and 2634167; asteroid:99942-Apophis is now also sampled at 2378498.5 and 2451917.5 to complete the selected-asteroid bridge.; columns=epoch_jd, body, x_km, y_km, z_km; redistribution=repository-checked regression fixtures, not a broad public corpus.";
    const EXPECTED_INDEPENDENT_HOLDOUT_SNAPSHOT_MANIFEST_SUMMARY_LINE: &str = r"Independent hold-out snapshot: Independent JPL Horizons hold-out snapshot used only for interpolation validation.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.; coverage=major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Mercury and Venus at 2451545, 2451915.25, and 2451915.75, plus Jupiter, Saturn, Uranus, Neptune, and Pluto at 2451545, plus Mars at 2451545, plus Sun at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Moon at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 66 rows across 16 bodies and 12 epochs.; columns=epoch_jd, body, x_km, y_km, z_km; redistribution=repository-checked regression fixtures, not a broad public corpus.";

    #[test]
    fn snapshot_manifest_summary_lines_byte_identical() {
        let summaries = [
            (
                SnapshotManifestSummary {
                    label: "Reference snapshot",
                    manifest: reference_snapshot_manifest().clone(),
                    source_fallback: "unknown",
                    coverage_fallback: "unknown",
                },
                EXPECTED_REFERENCE_SNAPSHOT_MANIFEST_SUMMARY_LINE,
            ),
            (
                SnapshotManifestSummary {
                    label: "Independent hold-out snapshot",
                    manifest: independent_holdout_snapshot_manifest().clone(),
                    source_fallback: "unknown",
                    coverage_fallback: "unknown",
                },
                EXPECTED_INDEPENDENT_HOLDOUT_SNAPSHOT_MANIFEST_SUMMARY_LINE,
            ),
        ];

        for (summary, expected) in &summaries {
            let after = super::snapshot_manifest_summary_line(summary);
            assert_eq!(after, *expected);
        }
    }
}
