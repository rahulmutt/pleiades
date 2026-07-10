//! Time-scale, delta-T, observer, and request policy summary formatters and text.

use crate::*;

pub(crate) fn format_time_scale_policy_summary_for_report(
    summary: &crate::posture::backend_policy::TimeScalePolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line.to_string(),
        Err(error) => format!("time-scale policy unavailable ({error})"),
    }
}

pub(crate) fn format_delta_t_policy_summary_for_report(
    summary: &crate::posture::backend_policy::DeltaTPolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line.to_string(),
        Err(error) => format!("delta T policy unavailable ({error})"),
    }
}

pub(crate) fn format_observer_policy_summary_for_report(
    summary: &crate::posture::backend_policy::ObserverPolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line.to_string(),
        Err(error) => format!("observer policy unavailable ({error})"),
    }
}

pub(crate) fn format_apparentness_policy_summary_for_report(
    summary: &crate::posture::backend_policy::ApparentnessPolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line.to_string(),
        Err(error) => format!("apparentness policy unavailable ({error})"),
    }
}

pub(crate) fn format_request_policy_summary_for_report(
    summary: &crate::posture::backend_policy::RequestPolicySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("request policy unavailable ({error})"),
    }
}

pub(crate) fn validated_request_policy_summary_for_report(
) -> Result<crate::posture::backend_policy::RequestPolicySummary, String> {
    let summary = request_policy_summary_for_report();
    summary.validate().map_err(|error| error.to_string())?;
    Ok(summary)
}

pub(crate) fn validated_production_generation_body_class_coverage_summary_for_report() -> String {
    match validated_production_generation_snapshot_body_class_coverage_summary_for_report() {
        Ok(summary) => summary,
        Err(error) => format!("Production generation body-class coverage unavailable ({error})"),
    }
}

pub(crate) fn format_request_semantics_summary_for_report(
    time_scale_policy: &crate::posture::backend_policy::TimeScalePolicySummary,
) -> String {
    use std::fmt::Write as _;

    let mut text = String::new();
    let _ = writeln!(
        text,
        "Time-scale policy: {}",
        format_time_scale_policy_summary_for_report(time_scale_policy)
    );

    let utc_convenience_policy =
        crate::posture::backend_policy::validated_utc_convenience_policy_summary_for_report();
    let _ = writeln!(text, "UTC convenience policy: {}", utc_convenience_policy);

    let delta_t_policy = delta_t_policy_summary_for_report();
    let _ = writeln!(
        text,
        "Delta T policy: {}",
        format_delta_t_policy_summary_for_report(&delta_t_policy)
    );

    let native_sidereal_policy =
        crate::posture::backend_policy::validated_native_sidereal_policy_summary_for_report();
    let _ = writeln!(text, "Native sidereal policy: {}", native_sidereal_policy);

    let request_policy = match validated_request_policy_summary_for_report() {
        Ok(summary) => summary,
        Err(error) => {
            let _ = writeln!(text, "Observer policy unavailable ({error})");
            let _ = writeln!(text, "Apparentness policy unavailable ({error})");
            let _ = writeln!(text, "Request policy unavailable ({error})");
            return text;
        }
    };

    let observer_policy = crate::posture::backend_policy::observer_policy_summary_for_report();
    let apparentness_policy =
        crate::posture::backend_policy::apparentness_policy_summary_for_report();
    let _ = writeln!(
        text,
        "Observer policy: {}",
        format_observer_policy_summary_for_report(&observer_policy)
    );
    let _ = writeln!(
        text,
        "Apparentness policy: {}",
        format_apparentness_policy_summary_for_report(&apparentness_policy)
    );
    let _ = writeln!(
        text,
        "Request policy: {}",
        format_request_policy_summary_for_report(&request_policy)
    );
    text
}

pub(crate) fn render_time_scale_policy_summary_text() -> String {
    match time_scale_policy_summary_for_report().validated_summary_line() {
        Ok(summary) => format!(
            "Time-scale policy summary\nTime-scale policy: {}\n",
            summary
        ),
        Err(error) => {
            format!("Time-scale policy summary\nTime-scale policy unavailable ({error})\n")
        }
    }
}

pub(crate) fn render_delta_t_policy_summary_text() -> String {
    match delta_t_policy_summary_for_report().validated_summary_line() {
        Ok(summary) => format!("Delta T policy summary\nDelta T policy: {}\n", summary),
        Err(error) => format!("Delta T policy summary\nDelta T policy unavailable ({error})\n"),
    }
}

pub(crate) fn render_zodiac_policy_summary_text() -> String {
    format!(
        "Zodiac policy summary\nZodiac policy: {}\n",
        crate::posture::backend_policy::validated_zodiac_policy_summary_for_report()
    )
}

pub(crate) fn render_utc_convenience_policy_summary_text() -> String {
    format!(
        "UTC convenience policy summary\nUTC convenience policy: {}\n",
        crate::posture::backend_policy::validated_utc_convenience_policy_summary_for_report()
    )
}

pub(crate) fn render_observer_policy_summary_text() -> String {
    match crate::posture::backend_policy::observer_policy_summary_for_report()
        .validated_summary_line()
    {
        Ok(summary) => format!("Observer policy summary\nObserver policy: {}\n", summary),
        Err(error) => format!("Observer policy summary\nObserver policy unavailable ({error})\n"),
    }
}

pub(crate) fn render_apparentness_policy_summary_text() -> String {
    match crate::posture::backend_policy::apparentness_policy_summary_for_report()
        .validated_summary_line()
    {
        Ok(summary) => format!(
            "Apparentness policy summary\nApparentness policy: {}\n",
            summary
        ),
        Err(error) => {
            format!("Apparentness policy summary\nApparentness policy unavailable ({error})\n")
        }
    }
}

pub(crate) fn render_native_sidereal_policy_summary_text() -> String {
    format!(
        "Native sidereal policy summary\nNative sidereal policy: {}\n",
        crate::posture::backend_policy::validated_native_sidereal_policy_summary_for_report()
    )
}

pub(crate) fn render_interpolation_posture_summary_text() -> String {
    match jpl_interpolation_posture_summary() {
        Some(summary) => {
            match summary.validated_summary_line() {
                Ok(summary) => format!(
                    "Interpolation posture summary\nInterpolation posture: {}\n",
                    summary
                ),
                Err(error) => {
                    format!("Interpolation posture summary\nInterpolation posture unavailable ({error})\n")
                }
            }
        }
        None => "Interpolation posture summary\nInterpolation posture unavailable\n".to_string(),
    }
}

pub(crate) fn render_interpolation_quality_summary_text() -> String {
    format!(
        "Interpolation quality summary\n{}\n",
        format_jpl_interpolation_quality_summary_for_report()
    )
}

pub(crate) fn render_comparison_snapshot_summary_text() -> String {
    format!(
        "Comparison snapshot summary\n{}\n",
        comparison_snapshot_summary_for_report()
    )
}
