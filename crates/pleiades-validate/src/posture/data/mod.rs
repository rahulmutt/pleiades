//! `pleiades-data` report/summary prose relocated from the functional crate
//! (report-surface relocation program, Slice C). Rendering only — the
//! functional crate keeps the structured data, the `&'static str` accessors,
//! its inherent methods, and all release-gate data.
#![allow(dead_code)]

pub(crate) mod accuracy_baseline;
pub(crate) mod coverage;
pub(crate) mod lookup;
pub(crate) mod regenerate;
pub(crate) mod thresholds;
