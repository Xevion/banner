use std::{ops::RangeInclusive, str::FromStr};

use anyhow::Context;
use serde::{Deserialize, Serialize};

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

impl Term {
    pub fn to_string(&self) -> String {
        format!("{}{}", self.year, self.season.to_string())
    }
}

impl Season {
    pub fn to_string(&self) -> String {
        (match self {
            Season::Fall => "10",
            Season::Spring => "20",
            Season::Summer => "30",
        })
        .to_string()
    }

    pub fn from_string(s: &str) -> Option<Season> {
        match s {
            "10" => Some(Season::Fall),
            "20" => Some(Season::Spring),
            "30" => Some(Season::Summer),
            _ => None,
        }
    }
}

const CURRENT_YEAR: u32 = compile_time::date!().year() as u32;
const VALID_YEARS: RangeInclusive<u32> = 2007..=(CURRENT_YEAR + 10);

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
            Season::from_string(&s[4..6]).ok_or_else(|| anyhow::anyhow!("Invalid season code"))?;

        Ok(Term { year, season })
    }
}

impl FromStr for Season {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s).ok_or(())
    }
}
