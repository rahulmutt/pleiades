Generate SPEC.md, a top-level specification file that links to logically decomposed subcomponents in spec/*.md, for a program that does the following:
1. Contains a modular ephemeris software similar to Swiss Ephemeris with the intention to be used in astrology software - focus should be on Sun, Moon, planets, asteriods, etc. with support for all House systems, ayanamsa, etc.
2. Must be implemented in pure Rust, no C / C++ dependencies.
3. Must define a modular ephemeris backend trait with implementations using different public data sources and algorithms (some algorithms don't require data). Examples include JPL, etc. 
  3a. You may use any pure-Rust crates for various backend implementations. Each backend implementation should be a separate crate.
4. Design a compressed representation for common use for data between 1500-2500.
5. All sub-crate names should have the form `pleiades-*` .
