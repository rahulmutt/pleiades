//! `pleiades-jpl` report/summary prose relocated from the functional crate
//! (report-surface relocation program, Slice D). Rendering only — the
//! functional crate keeps the structured evidence structs, their
//! `*_details()` constructors, `validate()`/`label()` methods, and all
//! release-gate data.
#![allow(dead_code)]

pub(crate) mod backend;
pub(crate) mod comparison;
pub(crate) mod holdout;
pub(crate) mod jpl_posture;
pub(crate) mod production_generation;
pub(crate) mod reference_asteroid;
pub(crate) mod reference_snapshot;
