//! Unit tests for the `systems` module, split by house-system formula family.
//! Relocated from the former monolithic `systems/tests.rs` per AGENTS.md
//! ("split a large file before adding to it") and the `pleiades-types`
//! precedent. Shared setup lives in `support`; family-local helpers stay in
//! the family file that uses them.

mod dispatch;
mod greatcircle;
mod primitives;
mod quadrant;
mod request;
mod sector;
mod sunshine;
mod support;
mod trivial;
