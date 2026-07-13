//! `pleiades-jpl` report/summary prose relocated from the functional crate
//! (report-surface relocation program, Slice D). Rendering only — the
//! functional crate keeps the structured evidence structs, their
//! `*_details()` constructors, `validate()`/`label()` methods, and all
//! release-gate data.
#![allow(dead_code)]

pub(crate) mod backend;
pub(crate) mod comparison;
pub(crate) mod data_phase2_alignment;
pub(crate) mod holdout;
pub(crate) mod jpl_posture;
pub(crate) mod production_generation;
pub(crate) mod reference_asteroid;
pub(crate) mod reference_snapshot;
pub(crate) mod selected_asteroid;

// Slice D Task 13: flat glob re-exports so every renderer copied under
// `posture/jpl/` is reachable as `crate::posture::jpl::<name>`, regardless of
// which submodule it lives in. This lets aggregator files (e.g.
// `jpl_posture.rs`) call other modules' renderers without a `pleiades_jpl::`
// detour, self-containing validate ahead of Task 14 deleting jpl's render
// layer.
#[allow(unused_imports)]
pub(crate) use backend::*;
#[allow(unused_imports)]
pub(crate) use comparison::*;
#[allow(unused_imports)]
pub(crate) use data_phase2_alignment::*;
#[allow(unused_imports)]
pub(crate) use holdout::*;
#[allow(unused_imports)]
pub(crate) use jpl_posture::*;
#[allow(unused_imports)]
pub(crate) use production_generation::*;
#[allow(unused_imports)]
pub(crate) use reference_asteroid::*;
#[allow(unused_imports)]
pub(crate) use reference_snapshot::*;
#[allow(unused_imports)]
pub(crate) use selected_asteroid::*;
