use std::{fmt::Display, str::FromStr};

use chrono::{Months, NaiveDate};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

lazy_static! {
    pub static ref SEASON_START: NaiveDate = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::EnumIter)]
pub enum Class {
    RSU,
    RST,
    CC,
    RP,
}

impl Class {
    pub fn name(&self) -> &'static str {
        match self {
            Self::RSU => "Start Up",
            Self::RST => "Shooting Star",
            Self::CC => "Compound",
            Self::RP => "Professional",
        }
    }
    pub fn comment(&self) -> &'static str {
        match self {
            Self::RSU => "aus den jeweiligen Einsteigerkursen / Schüler",
            Self::RST => "Jugend, Erwachsene (Recurve + Blankbogen)",
            Self::CC => "alle ab Jugend",
            Self::RP => "",
        }
    }
    pub fn all_classes() -> impl Iterator<Item = Self> {
        Self::iter()
    }
    pub fn in_range(&self, dob: NaiveDate) -> bool {
        let year_range = match self {
            _ => (1, 120),
        };

        let date_range = (*SEASON_START - Months::new(year_range.1 * 12))
            ..(*SEASON_START - Months::new((year_range.0 - 1) * 12));
        date_range.contains(&dob)
    }
    pub fn classes_for(dob: NaiveDate) -> Vec<Class> {
        Self::all_classes().filter(|c| c.in_range(dob)).collect()
    }
}

impl FromStr for Class {
    type Err = UnknownClassError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::iter()
            .find(|c| format!("{c:?}") == s)
            .ok_or(UnknownClassError { class: s.into() })
    }
}

#[derive(Debug)]
pub struct UnknownClassError {
    pub class: String,
}

impl std::error::Error for UnknownClassError {}
impl Display for UnknownClassError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown class: {}", self.class)
    }
}
