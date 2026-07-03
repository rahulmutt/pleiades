//! Ephemeris event-finding for the `pleiades` workspace: longitude crossings of
//! the Sun, Moon, and planets, derived from pleiades' validated body positions.
//!
//! The engine is generic over any [`pleiades_backend::EphemerisBackend`] and,
//! like `pleiades-eclipse`, works in TDB over the 1900–2100 CE packaged window.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod root;

pub use error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
