//! Embedded lunar-theory data modules.
//!
//! This module groups the coefficient tables and pure position formulae that
//! are fixed by the selected lunar theory. The submodules are kept whole so
//! that regeneration tooling can round-trip without content changes.

pub mod moonposition;
