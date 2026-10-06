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
    const EXPECTED_INTERPOLATION_QUALITY_SAMPLE_LINES: &str = r"Sun at JD 2451545 TDB: cubic interpolation, bracket span 36890.0 d, |Δlon|=121.663164344081°, |Δlat|=0.072977887833°, |Δdist|=43.862485432864 AU
Sun at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=0.000003672927°, |Δlat|=0.000000314487°, |Δdist|=0.000000110036 AU
Sun at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000847223°, |Δlat|=0.000000074879°, |Δdist|=0.000000026462 AU
Sun at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000491802°, |Δlat|=0.000000044928°, |Δdist|=0.000000016030 AU
Sun at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000000164001°, |Δlat|=0.000000015036°, |Δdist|=0.000000003462 AU
Sun at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000074541°, |Δlat|=0.000000006853°, |Δdist|=0.000000001303 AU
Sun at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000044318°, |Δlat|=0.000000004112°, |Δdist|=0.000000000789 AU
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
Moon at JD 2451545 TDB: cubic interpolation, bracket span 36890.0 d, |Δlon|=130.490408927591°, |Δlat|=6.344730465084°, |Δdist|=228.421961382032 AU
Moon at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=0.002719936170°, |Δlat|=0.000142811672°, |Δdist|=0.000005301797 AU
Moon at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.005124889524°, |Δlat|=0.000112029477°, |Δdist|=0.000001240366 AU
Moon at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.006563948700°, |Δlat|=0.000018274549°, |Δdist|=0.000000696224 AU
Moon at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.001708589255°, |Δlat|=0.000030316687°, |Δdist|=0.000000218859 AU
Moon at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000746208015°, |Δlat|=0.000020838629°, |Δdist|=0.000000099124 AU
Moon at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000604079514°, |Δlat|=0.000022851024°, |Δdist|=0.000000056765 AU
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
Mercury at JD 2451545 TDB: cubic interpolation, bracket span 36890.0 d, |Δlon|=107.521396111750°, |Δlat|=2.359766143492°, |Δdist|=718.855991367373 AU
Mercury at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=0.000127792246°, |Δlat|=0.000001338763°, |Δdist|=0.000002892999 AU
Mercury at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000031333841°, |Δlat|=0.000000308995°, |Δdist|=0.000000667160 AU
Mercury at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000019344384°, |Δlat|=0.000000176746°, |Δdist|=0.000000386942 AU
Mercury at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000005465140°, |Δlat|=0.000000005588°, |Δdist|=0.000000114848 AU
Mercury at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000002385449°, |Δlat|=0.000000005936°, |Δdist|=0.000000050847 AU
Mercury at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000001453268°, |Δlat|=0.000000004098°, |Δdist|=0.000000030006 AU
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
Venus at JD 2451545 TDB: cubic interpolation, bracket span 36890.0 d, |Δlon|=106.742072222760°, |Δlat|=1.523350922243°, |Δdist|=116.981456505093 AU
Venus at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=0.000024789519°, |Δlat|=0.000000834407°, |Δdist|=0.000000185456 AU
Venus at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000005894111°, |Δlat|=0.000000200991°, |Δdist|=0.000000045743 AU
Venus at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000003530364°, |Δlat|=0.000000122270°, |Δdist|=0.000000028384 AU
Venus at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000000961569°, |Δlat|=0.000000034325°, |Δdist|=0.000000006919 AU
Venus at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000416047°, |Δlat|=0.000000014977°, |Δdist|=0.000000002831 AU
Venus at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000249604°, |Δlat|=0.000000009052°, |Δdist|=0.000000001731 AU
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
Jupiter at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=0.000026288814°, |Δlat|=0.000001257925°, |Δdist|=0.000001653170 AU
Jupiter at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000006251271°, |Δlat|=0.000000298288°, |Δdist|=0.000000392976 AU
Jupiter at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000003745629°, |Δlat|=0.000000178238°, |Δdist|=0.000000235418 AU
Jupiter at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000002476045°, |Δlat|=0.000000166528°, |Δdist|=0.000000255893 AU
Jupiter at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000001235376°, |Δlat|=0.000000083107°, |Δdist|=0.000000128003 AU
Jupiter at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000001622986°, |Δlat|=0.000000015321°, |Δdist|=0.000000017163 AU
Jupiter at JD 2451915 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000177924°, |Δlat|=0.000000086584°, |Δdist|=0.000000120733 AU
Jupiter at JD 2451915.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000001542939°, |Δlat|=0.000000021550°, |Δdist|=0.000000041701 AU
Jupiter at JD 2451916 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000895731°, |Δlat|=0.000000072558°, |Δdist|=0.000000102253 AU
Jupiter at JD 2451916.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000001342912°, |Δlat|=0.000000108609°, |Δdist|=0.000000153325 AU
Jupiter at JD 2451917 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000001437290°, |Δlat|=0.000000226298°, |Δdist|=0.000000331439 AU
Jupiter at JD 2451918.5 TDB: cubic interpolation, bracket span 2.5 d, |Δlon|=0.000002581350°, |Δlat|=0.000000036609°, |Δdist|=0.000000016606 AU
Jupiter at JD 2451919.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000003217442°, |Δlat|=0.000000045655°, |Δdist|=0.000000020986 AU
Jupiter at JD 2451920.5 TDB: cubic interpolation, bracket span 1081.0 d, |Δlon|=0.000011976973°, |Δlat|=0.000000170032°, |Δdist|=0.000000079150 AU
Mars at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=0.000003921763°, |Δlat|=0.000000161774°, |Δdist|=0.000000009176 AU
Mars at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000939385°, |Δlat|=0.000000038936°, |Δdist|=0.000000001887 AU
Mars at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000566997°, |Δlat|=0.000000023615°, |Δdist|=0.000000000954 AU
Mars at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000000134855°, |Δlat|=0.000000007377°, |Δdist|=0.000000001136 AU
Mars at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000067704°, |Δlat|=0.000000003707°, |Δdist|=0.000000000558 AU
Mars at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000042767°, |Δlat|=0.000000002717°, |Δdist|=0.000000000512 AU
Mars at JD 2451915 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000039419°, |Δlat|=0.000000002759°, |Δdist|=0.000000000628 AU
Mars at JD 2451915.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000035306°, |Δlat|=0.000000002914°, |Δdist|=0.000000000740 AU
Mars at JD 2451916 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000029746°, |Δlat|=0.000000002823°, |Δdist|=0.000000000828 AU
Mars at JD 2451916.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000044949°, |Δlat|=0.000000004253°, |Δdist|=0.000000001236 AU
Mars at JD 2451917 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000091346°, |Δlat|=0.000000011874°, |Δdist|=0.000000004033 AU
Mars at JD 2451918.5 TDB: cubic interpolation, bracket span 2.5 d, |Δlon|=0.000000148266°, |Δlat|=0.000000024347°, |Δdist|=0.000000015456 AU
Mars at JD 2451919.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000179837°, |Δlat|=0.000000030608°, |Δdist|=0.000000019375 AU
Mars at JD 2451920.5 TDB: cubic interpolation, bracket span 1081.0 d, |Δlon|=0.000000650619°, |Δlat|=0.000000114938°, |Δdist|=0.000000072528 AU
Neptune at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=0.000002599724°, |Δlat|=0.000000732823°, |Δdist|=0.000002957871 AU
Neptune at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000618039°, |Δlat|=0.000000174438°, |Δdist|=0.000000704461 AU
Neptune at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000370258°, |Δlat|=0.000000104638°, |Δdist|=0.000000422801 AU
Neptune at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000000312624°, |Δlat|=0.000000012187°, |Δdist|=0.000000394710 AU
Neptune at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000156182°, |Δlat|=0.000000006092°, |Δdist|=0.000000197382 AU
Neptune at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000068696°, |Δlat|=0.000000005951°, |Δdist|=0.000000069391 AU
Neptune at JD 2451915 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000008510°, |Δlat|=0.000000008564°, |Δdist|=0.000000005119 AU
Neptune at JD 2451915.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000000589°, |Δlat|=0.000000008641°, |Δdist|=0.000000004681 AU
Neptune at JD 2451916 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000004007°, |Δlat|=0.000000008685°, |Δdist|=0.000000005141 AU
Neptune at JD 2451916.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000006014°, |Δlat|=0.000000013027°, |Δdist|=0.000000007711 AU
Neptune at JD 2451917 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000040670°, |Δlat|=0.000000010211°, |Δdist|=0.000000006452 AU
Neptune at JD 2451918.5 TDB: cubic interpolation, bracket span 2.5 d, |Δlon|=0.000000021052°, |Δlat|=0.000000182468°, |Δdist|=0.000000088571 AU
Neptune at JD 2451919.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000026443°, |Δlat|=0.000000228048°, |Δdist|=0.000000110705 AU
Neptune at JD 2451920.5 TDB: cubic interpolation, bracket span 1081.0 d, |Δlon|=0.000000099196°, |Δlat|=0.000000851251°, |Δdist|=0.000000413267 AU
Pluto at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=0.000003647771°, |Δlat|=0.000011677391°, |Δdist|=0.000007821441 AU
Pluto at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000871144°, |Δlat|=0.000002780975°, |Δdist|=0.000001861936 AU
Pluto at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000524257°, |Δlat|=0.000001668991°, |Δdist|=0.000001116974 AU
Pluto at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000000392304°, |Δlat|=0.000000474853°, |Δdist|=0.000000205352 AU
Pluto at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000196222°, |Δlat|=0.000000237466°, |Δdist|=0.000000102646 AU
Pluto at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000089384°, |Δlat|=0.000000227607°, |Δdist|=0.000000005170 AU
Pluto at JD 2451915 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000027896°, |Δlat|=0.000000224289°, |Δdist|=0.000000050963 AU
Pluto at JD 2451915.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000040735°, |Δlat|=0.000000167863°, |Δdist|=0.000000094916 AU
Pluto at JD 2451916 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000100256°, |Δlat|=0.000000071740°, |Δdist|=0.000000116385 AU
Pluto at JD 2451916.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000150506°, |Δlat|=0.000000107622°, |Δdist|=0.000000174554 AU
Pluto at JD 2451917 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000612848°, |Δlat|=0.000000362331°, |Δdist|=0.000000433316 AU
Pluto at JD 2451918.5 TDB: cubic interpolation, bracket span 2.5 d, |Δlon|=0.000000057392°, |Δlat|=0.000004324278°, |Δdist|=0.000001466570 AU
Pluto at JD 2451919.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000069252°, |Δlat|=0.000005407113°, |Δdist|=0.000001833140 AU
Pluto at JD 2451920.5 TDB: cubic interpolation, bracket span 1081.0 d, |Δlon|=0.000000249331°, |Δlat|=0.000020193332°, |Δdist|=0.000006843431 AU
Saturn at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=0.000000910029°, |Δlat|=0.000000009769°, |Δdist|=0.000000035795 AU
Saturn at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000216405°, |Δlat|=0.000000002312°, |Δdist|=0.000000008499 AU
Saturn at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000129678°, |Δlat|=0.000000001379°, |Δdist|=0.000000005086 AU
Saturn at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000000062989°, |Δlat|=0.000000007976°, |Δdist|=0.000000001028 AU
Saturn at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000031472°, |Δlat|=0.000000003986°, |Δdist|=0.000000000512 AU
Saturn at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000011966°, |Δlat|=0.000000006229°, |Δdist|=0.000000002258 AU
Saturn at JD 2451915 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000004086°, |Δlat|=0.000000003084°, |Δdist|=0.000000000108 AU
Saturn at JD 2451915.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000015544°, |Δlat|=0.000000007394°, |Δdist|=0.000000001874 AU
Saturn at JD 2451916 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000029692°, |Δlat|=0.000000000745°, |Δdist|=0.000000002300 AU
Saturn at JD 2451916.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000044496°, |Δlat|=0.000000001116°, |Δdist|=0.000000003452 AU
Saturn at JD 2451917 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000003596°, |Δlat|=0.000000036349°, |Δdist|=0.000000012694 AU
Saturn at JD 2451918.5 TDB: cubic interpolation, bracket span 2.5 d, |Δlon|=0.000000001621°, |Δlat|=0.000000003840°, |Δdist|=0.000000014092 AU
Saturn at JD 2451919.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000002083°, |Δlat|=0.000000004783°, |Δdist|=0.000000017615 AU
Saturn at JD 2451920.5 TDB: cubic interpolation, bracket span 1081.0 d, |Δlon|=0.000000007970°, |Δlat|=0.000000017792°, |Δdist|=0.000000065762 AU
Uranus at JD 2451910.5 TDB: cubic interpolation, bracket span 366.5 d, |Δlon|=0.000000332541°, |Δlat|=0.000000814204°, |Δdist|=0.000000024269 AU
Uranus at JD 2451911.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000079147°, |Δlat|=0.000000193760°, |Δdist|=0.000000005754 AU
Uranus at JD 2451912.5 TDB: cubic interpolation, bracket span 2.0 d, |Δlon|=0.000000047472°, |Δlat|=0.000000116198°, |Δdist|=0.000000003437 AU
Uranus at JD 2451913.5 TDB: cubic interpolation, bracket span 1.5 d, |Δlon|=0.000000024117°, |Δlat|=0.000000029530°, |Δdist|=0.000000016257 AU
Uranus at JD 2451914 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000012046°, |Δlat|=0.000000014762°, |Δdist|=0.000000008131 AU
Uranus at JD 2451914.5 TDB: cubic interpolation, bracket span 1.0 d, |Δlon|=0.000000003758°, |Δlat|=0.000000013544°, |Δdist|=0.000000004965 AU
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
asteroid:99942-Apophis at JD 2451915.5 TDB: quadratic interpolation, bracket span 373.5 d, |Δlon|=0.031634478128°, |Δlat|=0.001187669876°, |Δdist|=0.003407563811 AU
asteroid:99942-Apophis at JD 2451918.5 TDB: quadratic interpolation, bracket span 4.0 d, |Δlon|=0.006296585256°, |Δlat|=0.000251476835°, |Δdist|=0.000871183801 AU";

    #[test]
    fn interpolation_quality_sample_lines_byte_identical() {
        let after = interpolation_quality_sample_list()
            .iter()
            .map(super::interpolation_quality_sample_summary_line)
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(after, EXPECTED_INTERPOLATION_QUALITY_SAMPLE_LINES);
    }

    const EXPECTED_REFERENCE_SNAPSHOT_MANIFEST_SUMMARY_LINE: &str = r"Reference snapshot: JPL Horizons reference snapshot.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.; coverage=major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; Sun, Moon, Mercury and Venus sampled at 2415020.5 (1900-01-01), 2451915.25 and 2451915.75; Sun through Pluto sampled at 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451916.0, 2451916.5, 2451917.0, 2451917.5, 2451918.5, 2451919.5, 2451920.5, and 2453000.5; Ceres, Pallas, Juno, Vesta and asteroid:433-Eros sampled at 2378498.5, 2451545, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.0, 2451914.5, 2451915.0, 2451915.5, 2451916.5, 2451917.5, 2451918.5, 2451919.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis sampled at 2378498.5, 2451545, 2451915.5, 2451917.5, 2451918.5, and 2451919.5.; columns=epoch_jd, body, x_km, y_km, z_km; redistribution=repository-checked regression fixtures, not a broad public corpus.";
    const EXPECTED_INDEPENDENT_HOLDOUT_SNAPSHOT_MANIFEST_SUMMARY_LINE: &str = r"Independent hold-out snapshot: Independent JPL Horizons hold-out snapshot used only for interpolation validation.; source=NASA/JPL Horizons API, DE441, geocentric ecliptic J2000 vector tables.; coverage=major-body samples are confined to the 1900-2100 window [JD 2415020.5, 2488069.5]; Mars and Jupiter at 2001-01-01 through 2001-01-03, plus Mercury and Venus at 2451545, 2451915.25, and 2451915.75, plus Jupiter, Saturn, Uranus, Neptune, and Pluto at 2451545, plus Mars at 2451545, plus Sun at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Moon at 2451545, 2451915.25, 2451915.75, and 2451915.5, plus Mercury at 2451915.5, plus Venus at 2451915.5, plus major bodies at 2451915.5 for Sun through Pluto, plus selected asteroids at 2378498.5, 2451545, 2451915.5, 2451917.5, 2453000.5, 2500000, and 2634167; asteroid:99942-Apophis now also appears at 2378498.5 so the selected-asteroid hold-out bridge matches the reference slice; total slice size is 64 rows across 16 bodies and 12 epochs.; columns=epoch_jd, body, x_km, y_km, z_km; redistribution=repository-checked regression fixtures, not a broad public corpus.";

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
