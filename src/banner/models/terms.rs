use std::{ops::RangeInclusive, str::FromStr};

use anyhow::Context;
use chrono::{Datelike, Local, NaiveDate};
use serde::{Deserialize, Serialize};

/// The current year at the time of compilation
const CURRENT_YEAR: u32 = compile_time::date!().year() as u32;

/// The valid years for terms
/// We set a semi-static upper limit to avoid having to update this value while also keeping a tight bound
/// TODO: Recheck the lower bound, it's just a guess right now.
const VALID_YEARS: RangeInclusive<u32> = 2007..=(CURRENT_YEAR + 10);

/// Represents a term in the Banner system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Term {
    pub year: u32, // 2024, 2025, etc
    pub season: Season,
}

/// Represents the term status at a specific point in time
#[derive(Debug, Clone)]
pub enum TermPoint {
    /// Currently in a term
    InTerm { current: Term },
    /// Between terms, with the next term specified
    BetweenTerms { next: Term },
}

/// Represents a season within a term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Season {
    Fall,
    Spring,
    Summer,
}

impl Term {
    /// Returns the current term status - either currently in a term or between terms
    pub fn get_current() -> TermPoint {
        let now = Local::now().naive_local();
        Self::get_status_for_date(now.date())
    }

    /// Returns the current term status for a specific date
    pub fn get_status_for_date(date: NaiveDate) -> TermPoint {
        let literal_year = date.year() as u32;
        let day_of_year = date.ordinal();
        let ranges = Self::get_season_ranges(literal_year);

        // If we're past the end of the summer term, we're 'in' the next school year.
        let term_year = if day_of_year > ranges.summer.end {
            literal_year + 1
        } else {
            literal_year
        };

        if (day_of_year < ranges.spring.start) || (day_of_year >= ranges.fall.end) {
            // Fall over, Spring not yet begun
            TermPoint::BetweenTerms {
                next: Term {
                    year: term_year,
                    season: Season::Spring,
                },
            }
        } else if (day_of_year >= ranges.spring.start) && (day_of_year < ranges.spring.end) {
            // Spring
            TermPoint::InTerm {
                current: Term {
                    year: term_year,
                    season: Season::Spring,
                },
            }
        } else if day_of_year < ranges.summer.start {
            // Spring over, Summer not yet begun
            TermPoint::BetweenTerms {
                next: Term {
                    year: term_year,
                    season: Season::Summer,
                },
            }
        } else if (day_of_year >= ranges.summer.start) && (day_of_year < ranges.summer.end) {
            // Summer
            TermPoint::InTerm {
                current: Term {
                    year: term_year,
                    season: Season::Summer,
                },
            }
        } else if day_of_year < ranges.fall.start {
            // Summer over, Fall not yet begun
            TermPoint::BetweenTerms {
                next: Term {
                    year: term_year,
                    season: Season::Fall,
                },
            }
        } else if (day_of_year >= ranges.fall.start) && (day_of_year < ranges.fall.end) {
            // Fall
            TermPoint::InTerm {
                current: Term {
                    year: term_year,
                    season: Season::Fall,
                },
            }
        } else {
            // This should never happen, but Rust requires exhaustive matching
            panic!("Impossible code reached (dayOfYear: {})", day_of_year);
        }
    }

    /// Returns the start and end day of each term for the given year.
    /// The ranges are inclusive of the start day and exclusive of the end day.
    fn get_season_ranges(year: u32) -> SeasonRanges {
        let spring_start = NaiveDate::from_ymd_opt(year as i32, 1, 14)
            .unwrap()
            .ordinal();
        let spring_end = NaiveDate::from_ymd_opt(year as i32, 5, 1)
            .unwrap()
            .ordinal();
        let summer_start = NaiveDate::from_ymd_opt(year as i32, 5, 25)
            .unwrap()
            .ordinal();
        let summer_end = NaiveDate::from_ymd_opt(year as i32, 8, 15)
            .unwrap()
            .ordinal();
        let fall_start = NaiveDate::from_ymd_opt(year as i32, 8, 18)
            .unwrap()
            .ordinal();
        let fall_end = NaiveDate::from_ymd_opt(year as i32, 12, 10)
            .unwrap()
            .ordinal();

        SeasonRanges {
            spring: YearDayRange {
                start: spring_start,
                end: spring_end,
            },
            summer: YearDayRange {
                start: summer_start,
                end: summer_end,
            },
            fall: YearDayRange {
                start: fall_start,
                end: fall_end,
            },
        }
    }

    /// Returns a long string representation of the term (e.g., "Fall 2025")
    pub fn to_long_string(&self) -> String {
        format!("{} {}", self.season, self.year)
    }
}

impl TermPoint {
    /// Returns the inner Term regardless of the status
    pub fn inner(&self) -> &Term {
        match self {
            TermPoint::InTerm { current } => current,
            TermPoint::BetweenTerms { next } => next,
        }
    }
}

/// Represents the start and end day of each term within a year
#[derive(Debug, Clone)]
struct SeasonRanges {
    spring: YearDayRange,
    summer: YearDayRange,
    fall: YearDayRange,
}

/// Represents the start and end day of a term within a year
#[derive(Debug, Clone)]
struct YearDayRange {
    start: u32,
    end: u32,
}

impl std::fmt::Display for Term {
    /// Returns the term in the format YYYYXX, where YYYY is the year and XX is the season code
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{year}{season}",
            year = self.year,
            season = self.season.to_str()
        )
    }
}

impl Season {
    /// Returns the season code as a string
    fn to_str(&self) -> &'static str {
        match self {
            Season::Fall => "10",
            Season::Spring => "20",
            Season::Summer => "30",
        }
    }
}

impl std::fmt::Display for Season {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Season::Fall => write!(f, "Fall"),
            Season::Spring => write!(f, "Spring"),
            Season::Summer => write!(f, "Summer"),
        }
    }
}

impl FromStr for Season {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let season = match s {
            "10" => Season::Fall,
            "20" => Season::Spring,
            "30" => Season::Summer,
            _ => return Err(anyhow::anyhow!("Invalid season: {s}")),
        };
        Ok(season)
    }
}

impl FromStr for Term {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 6 {
            return Err(anyhow::anyhow!("Term string must be 6 characters"));
        }

        let year = s[0..4].parse::<u32>().context("Failed to parse year")?;
        if !VALID_YEARS.contains(&year) {
            return Err(anyhow::anyhow!("Year out of range"));
        }

        let season =
            Season::from_str(&s[4..6]).map_err(|e| anyhow::anyhow!("Invalid season: {}", e))?;

        Ok(Term { year, season })
    }
}
