use core::fmt;

use pleiades_types::ZodiacSign;

/// A summary of zodiac-sign placements in a chart snapshot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SignSummary {
    /// Placements in Aries.
    pub aries: usize,
    /// Placements in Taurus.
    pub taurus: usize,
    /// Placements in Gemini.
    pub gemini: usize,
    /// Placements in Cancer.
    pub cancer: usize,
    /// Placements in Leo.
    pub leo: usize,
    /// Placements in Virgo.
    pub virgo: usize,
    /// Placements in Libra.
    pub libra: usize,
    /// Placements in Scorpio.
    pub scorpio: usize,
    /// Placements in Sagittarius.
    pub sagittarius: usize,
    /// Placements in Capricorn.
    pub capricorn: usize,
    /// Placements in Aquarius.
    pub aquarius: usize,
    /// Placements in Pisces.
    pub pisces: usize,
}

impl SignSummary {
    pub(super) fn increment(&mut self, sign: ZodiacSign) {
        match sign {
            ZodiacSign::Aries => self.aries += 1,
            ZodiacSign::Taurus => self.taurus += 1,
            ZodiacSign::Gemini => self.gemini += 1,
            ZodiacSign::Cancer => self.cancer += 1,
            ZodiacSign::Leo => self.leo += 1,
            ZodiacSign::Virgo => self.virgo += 1,
            ZodiacSign::Libra => self.libra += 1,
            ZodiacSign::Scorpio => self.scorpio += 1,
            ZodiacSign::Sagittarius => self.sagittarius += 1,
            ZodiacSign::Capricorn => self.capricorn += 1,
            ZodiacSign::Aquarius => self.aquarius += 1,
            ZodiacSign::Pisces => self.pisces += 1,
            _ => {}
        }
    }

    /// Returns `true` when the snapshot contains at least one sign placement.
    pub fn has_known_signs(self) -> bool {
        self.aries
            + self.taurus
            + self.gemini
            + self.cancer
            + self.leo
            + self.virgo
            + self.libra
            + self.scorpio
            + self.sagittarius
            + self.capricorn
            + self.aquarius
            + self.pisces
            > 0
    }

    /// Returns the occupied zodiac signs in canonical zodiac order.
    pub fn occupied_signs(self) -> Vec<ZodiacSign> {
        let mut signs = Vec::new();
        for (count, sign) in [
            (self.aries, ZodiacSign::Aries),
            (self.taurus, ZodiacSign::Taurus),
            (self.gemini, ZodiacSign::Gemini),
            (self.cancer, ZodiacSign::Cancer),
            (self.leo, ZodiacSign::Leo),
            (self.virgo, ZodiacSign::Virgo),
            (self.libra, ZodiacSign::Libra),
            (self.scorpio, ZodiacSign::Scorpio),
            (self.sagittarius, ZodiacSign::Sagittarius),
            (self.capricorn, ZodiacSign::Capricorn),
            (self.aquarius, ZodiacSign::Aquarius),
            (self.pisces, ZodiacSign::Pisces),
        ] {
            if count > 0 {
                signs.push(sign);
            }
        }
        signs
    }

    /// Returns a compact one-line summary of the occupied zodiac signs.
    pub fn summary_line(self) -> String {
        self.to_string()
    }

    /// Validates that the summary covers the expected number of sign placements.
    pub fn validate(self, sign_count: usize) -> Result<(), SignSummaryValidationError> {
        let actual = self.aries
            + self.taurus
            + self.gemini
            + self.cancer
            + self.leo
            + self.virgo
            + self.libra
            + self.scorpio
            + self.sagittarius
            + self.capricorn
            + self.aquarius
            + self.pisces;
        if actual == sign_count {
            Ok(())
        } else {
            Err(SignSummaryValidationError::PlacementCountMismatch {
                expected: sign_count,
                actual,
            })
        }
    }

    /// Returns the compact summary line after validating the sign summary.
    pub fn validated_summary_line(
        self,
        sign_count: usize,
    ) -> Result<String, SignSummaryValidationError> {
        self.validate(sign_count)?;
        Ok(self.summary_line())
    }
}

/// Structured validation errors for a sign summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignSummaryValidationError {
    /// The summary total did not match the expected sign-placement count.
    PlacementCountMismatch { expected: usize, actual: usize },
}

impl fmt::Display for SignSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlacementCountMismatch { expected, actual } => write!(
                f,
                "sign summary placement count mismatch: expected {expected}, found {actual}"
            ),
        }
    }
}

impl std::error::Error for SignSummaryValidationError {}

pub(super) fn dominant_sign_summary(summary: SignSummary) -> SignSummary {
    let max = [
        summary.aries,
        summary.taurus,
        summary.gemini,
        summary.cancer,
        summary.leo,
        summary.virgo,
        summary.libra,
        summary.scorpio,
        summary.sagittarius,
        summary.capricorn,
        summary.aquarius,
        summary.pisces,
    ]
    .into_iter()
    .max()
    .unwrap_or(0);

    if max == 0 {
        return SignSummary::default();
    }

    SignSummary {
        aries: if summary.aries == max {
            summary.aries
        } else {
            0
        },
        taurus: if summary.taurus == max {
            summary.taurus
        } else {
            0
        },
        gemini: if summary.gemini == max {
            summary.gemini
        } else {
            0
        },
        cancer: if summary.cancer == max {
            summary.cancer
        } else {
            0
        },
        leo: if summary.leo == max { summary.leo } else { 0 },
        virgo: if summary.virgo == max {
            summary.virgo
        } else {
            0
        },
        libra: if summary.libra == max {
            summary.libra
        } else {
            0
        },
        scorpio: if summary.scorpio == max {
            summary.scorpio
        } else {
            0
        },
        sagittarius: if summary.sagittarius == max {
            summary.sagittarius
        } else {
            0
        },
        capricorn: if summary.capricorn == max {
            summary.capricorn
        } else {
            0
        },
        aquarius: if summary.aquarius == max {
            summary.aquarius
        } else {
            0
        },
        pisces: if summary.pisces == max {
            summary.pisces
        } else {
            0
        },
    }
}

impl fmt::Display for SignSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut wrote_any = false;
        for (count, sign) in [
            (self.aries, ZodiacSign::Aries),
            (self.taurus, ZodiacSign::Taurus),
            (self.gemini, ZodiacSign::Gemini),
            (self.cancer, ZodiacSign::Cancer),
            (self.leo, ZodiacSign::Leo),
            (self.virgo, ZodiacSign::Virgo),
            (self.libra, ZodiacSign::Libra),
            (self.scorpio, ZodiacSign::Scorpio),
            (self.sagittarius, ZodiacSign::Sagittarius),
            (self.capricorn, ZodiacSign::Capricorn),
            (self.aquarius, ZodiacSign::Aquarius),
            (self.pisces, ZodiacSign::Pisces),
        ] {
            if count == 0 {
                continue;
            }
            if wrote_any {
                f.write_str(", ")?;
            }
            wrote_any = true;
            write!(f, "{} {}", count, sign)?;
        }

        if !wrote_any {
            f.write_str("no sign placements")?;
        }

        Ok(())
    }
}
