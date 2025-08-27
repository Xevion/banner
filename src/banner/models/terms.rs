use std::{ops::RangeInclusive, str::FromStr};

use anyhow::Context;
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

/// Represents a season within a term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Season {
    Fall,
    Spring,
    Summer,
}

impl ToString for Term {
    /// Returns the term in the format YYYYXX, where YYYY is the year and XX is the season code
    fn to_string(&self) -> String {
        format!("{}{}", self.year, self.season.to_str())
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

impl FromStr for Season {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let season = match s {
            "10" => Season::Fall,
            "20" => Season::Spring,
            "30" => Season::Summer,
            _ => return Err(anyhow::anyhow!("Invalid season: {}", s)),
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
