use core::fmt;

/// A summary of occupied houses in a chart snapshot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HouseSummary {
    /// Placements in the first house.
    pub first: usize,
    /// Placements in the second house.
    pub second: usize,
    /// Placements in the third house.
    pub third: usize,
    /// Placements in the fourth house.
    pub fourth: usize,
    /// Placements in the fifth house.
    pub fifth: usize,
    /// Placements in the sixth house.
    pub sixth: usize,
    /// Placements in the seventh house.
    pub seventh: usize,
    /// Placements in the eighth house.
    pub eighth: usize,
    /// Placements in the ninth house.
    pub ninth: usize,
    /// Placements in the tenth house.
    pub tenth: usize,
    /// Placements in the eleventh house.
    pub eleventh: usize,
    /// Placements in the twelfth house.
    pub twelfth: usize,
    /// Placements without an assigned house.
    pub unknown: usize,
}

impl HouseSummary {
    pub(super) fn increment(&mut self, house: usize) {
        match house {
            1 => self.first += 1,
            2 => self.second += 1,
            3 => self.third += 1,
            4 => self.fourth += 1,
            5 => self.fifth += 1,
            6 => self.sixth += 1,
            7 => self.seventh += 1,
            8 => self.eighth += 1,
            9 => self.ninth += 1,
            10 => self.tenth += 1,
            11 => self.eleventh += 1,
            12 => self.twelfth += 1,
            _ => self.unknown += 1,
        }
    }

    /// Returns `true` when the snapshot contains at least one assigned house placement.
    pub fn has_known_houses(self) -> bool {
        self.first
            + self.second
            + self.third
            + self.fourth
            + self.fifth
            + self.sixth
            + self.seventh
            + self.eighth
            + self.ninth
            + self.tenth
            + self.eleventh
            + self.twelfth
            > 0
    }

    /// Returns the occupied house numbers in ascending order.
    pub fn occupied_houses(self) -> Vec<usize> {
        let mut houses = Vec::new();
        for (count, house) in [
            (self.first, 1usize),
            (self.second, 2),
            (self.third, 3),
            (self.fourth, 4),
            (self.fifth, 5),
            (self.sixth, 6),
            (self.seventh, 7),
            (self.eighth, 8),
            (self.ninth, 9),
            (self.tenth, 10),
            (self.eleventh, 11),
            (self.twelfth, 12),
        ] {
            if count > 0 {
                houses.push(house);
            }
        }
        houses
    }

    /// Returns a compact one-line summary of the occupied houses.
    pub fn summary_line(self) -> String {
        self.to_string()
    }

    /// Validates that the summary covers the expected number of placements.
    pub fn validate(self, placement_count: usize) -> Result<(), HouseSummaryValidationError> {
        let actual = self.first
            + self.second
            + self.third
            + self.fourth
            + self.fifth
            + self.sixth
            + self.seventh
            + self.eighth
            + self.ninth
            + self.tenth
            + self.eleventh
            + self.twelfth
            + self.unknown;
        if actual == placement_count {
            Ok(())
        } else {
            Err(HouseSummaryValidationError::PlacementCountMismatch {
                expected: placement_count,
                actual,
            })
        }
    }

    /// Returns the compact summary line after validating the house summary.
    pub fn validated_summary_line(
        self,
        placement_count: usize,
    ) -> Result<String, HouseSummaryValidationError> {
        self.validate(placement_count)?;
        Ok(self.summary_line())
    }
}

/// Structured validation errors for a house summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HouseSummaryValidationError {
    /// The summary total did not match the expected placement count.
    PlacementCountMismatch { expected: usize, actual: usize },
}

impl fmt::Display for HouseSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlacementCountMismatch { expected, actual } => write!(
                f,
                "house summary placement count mismatch: expected {expected}, found {actual}"
            ),
        }
    }
}

impl std::error::Error for HouseSummaryValidationError {}

pub(super) fn dominant_house_summary(summary: HouseSummary) -> HouseSummary {
    let max = [
        summary.first,
        summary.second,
        summary.third,
        summary.fourth,
        summary.fifth,
        summary.sixth,
        summary.seventh,
        summary.eighth,
        summary.ninth,
        summary.tenth,
        summary.eleventh,
        summary.twelfth,
    ]
    .into_iter()
    .max()
    .unwrap_or(0);

    if max == 0 {
        return HouseSummary::default();
    }

    HouseSummary {
        first: if summary.first == max {
            summary.first
        } else {
            0
        },
        second: if summary.second == max {
            summary.second
        } else {
            0
        },
        third: if summary.third == max {
            summary.third
        } else {
            0
        },
        fourth: if summary.fourth == max {
            summary.fourth
        } else {
            0
        },
        fifth: if summary.fifth == max {
            summary.fifth
        } else {
            0
        },
        sixth: if summary.sixth == max {
            summary.sixth
        } else {
            0
        },
        seventh: if summary.seventh == max {
            summary.seventh
        } else {
            0
        },
        eighth: if summary.eighth == max {
            summary.eighth
        } else {
            0
        },
        ninth: if summary.ninth == max {
            summary.ninth
        } else {
            0
        },
        tenth: if summary.tenth == max {
            summary.tenth
        } else {
            0
        },
        eleventh: if summary.eleventh == max {
            summary.eleventh
        } else {
            0
        },
        twelfth: if summary.twelfth == max {
            summary.twelfth
        } else {
            0
        },
        unknown: 0,
    }
}

pub(super) fn house_ordinal(house: usize) -> &'static str {
    match house {
        1 => "1st",
        2 => "2nd",
        3 => "3rd",
        4 => "4th",
        5 => "5th",
        6 => "6th",
        7 => "7th",
        8 => "8th",
        9 => "9th",
        10 => "10th",
        11 => "11th",
        12 => "12th",
        _ => "unknown",
    }
}

impl fmt::Display for HouseSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut wrote_any = false;
        for (count, house) in [
            (self.first, 1usize),
            (self.second, 2),
            (self.third, 3),
            (self.fourth, 4),
            (self.fifth, 5),
            (self.sixth, 6),
            (self.seventh, 7),
            (self.eighth, 8),
            (self.ninth, 9),
            (self.tenth, 10),
            (self.eleventh, 11),
            (self.twelfth, 12),
        ] {
            if count == 0 {
                continue;
            }
            if wrote_any {
                f.write_str(", ")?;
            }
            wrote_any = true;
            write!(f, "{} in {} house", count, house_ordinal(house))?;
        }

        if self.unknown > 0 {
            if wrote_any {
                f.write_str(", ")?;
            }
            wrote_any = true;
            write!(f, "{} unassigned", self.unknown)?;
        }

        if !wrote_any {
            f.write_str("no house placements")?;
        }

        Ok(())
    }
}
