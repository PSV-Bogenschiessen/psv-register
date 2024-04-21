use std::{fmt::Display, str::FromStr};

use crate::bow_type::BowType;
use chrono::{Months, NaiveDate};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

lazy_static! {
    pub static ref SEASON_START: NaiveDate = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::EnumIter)]
pub enum Class {
    R10,
    R11,
    R20,
    R30,
    R40,
    R12,
    B210,
    B211,
    B220,
    B230,
    B212,
    C110,
    C111,
    C120,
    C130,
    C112,
    OO,
}

impl Class {
    pub fn name(&self) -> &'static str {
        match self {
            Class::R10 => "Recurve Herren",
            Class::R11 => "Recurve Damen",
            Class::R20 => "Recurve Schüler m/w",
            Class::R30 => "Recurve Jugend m/w",
            Class::R40 => "Recurve Junioren m/w",
            Class::R12 => "Recurve Master m",
            Class::B210 => "Blank Herren",
            Class::B211 => "Blank Damen",
            Class::B220 => "Blank Schüler m/w",
            Class::B230 => "Blank Jugend m/m",
            Class::B212 => "Blank Master m",
            Class::C110 => "Compound Herren",
            Class::C111 => "Compound Damen",
            Class::C120 => "Compound Schüler m/w",
            Class::C130 => "Compound Jugend m/m",
            Class::C112 => "Compound Master m",
            Class::OO => "Offene Klasse",
        }
    }
    pub fn comment(&self) -> &'static str {
        match self{
            Class::OO => "Eine Klasse für alle. Die Auflage ist größer als bei den offizielen Klassen. Dadurch ist eine Qualifikation zur Bezirksmeisterschaft ausgeschlossen.",
            _ => "Reguläre Klasse nach Sportordnung. Eine Weitermeldung zur Bezirksmeisterschaft ist möglich"
        }
    }
    pub fn all_classes() -> impl Iterator<Item = Self> {
        Self::iter()
    }
    pub fn recurve_classes() -> &'static [Self] {
        &[
            Self::R10,
            Self::R11,
            Self::R20,
            Self::R30,
            Self::R40,
            Self::R12,
            Self::OO,
        ]
    }
    pub fn barebow_classes() -> &'static [Self] {
        &[
            Self::B210,
            Self::B211,
            Self::B220,
            Self::B230,
            Self::B212,
            Self::OO,
        ]
    }
    pub fn compound_classes() -> &'static [Self] {
        &[
            Self::C110,
            Self::C111,
            Self::C120,
            Self::C130,
            Self::C112,
            Self::OO,
        ]
    }
    pub fn in_range(&self, dob: NaiveDate) -> bool {
        let year_range = match self {
            Class::R10 => (21, 49),
            Class::R11 => (21, 120),
            Class::R20 => (1, 14),
            Class::R30 => (15, 17),
            Class::R40 => (18, 20),
            Class::R12 => (50, 120),
            Class::C110 => (18, 49),
            Class::C111 => (18, 120),
            Class::C120 => (1, 14),
            Class::C130 => (15, 17),
            Class::C112 => (50, 120),
            Class::B210 => (18, 49),
            Class::B211 => (18, 120),
            Class::B220 => (1, 14),
            Class::B230 => (15, 17),
            Class::B212 => (50, 120),
            Class::OO => (15, 120),
        };

        let date_range = (*SEASON_START - Months::new(year_range.1 * 12))
            ..(*SEASON_START - Months::new((year_range.0 - 1) * 12));
        date_range.contains(&dob)
    }
    pub fn classes_for(dob: NaiveDate, bow_type: BowType) -> Vec<Class> {
        match bow_type {
            BowType::Recurve => Self::recurve_classes(),
            BowType::Compound => Self::compound_classes(),
            BowType::Barebow => Self::barebow_classes(),
        }
        .iter()
        .filter(|c| c.in_range(dob))
        .copied()
        .collect()
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

#[test]
fn test_in_range() {
    assert!(!Class::R10.in_range(NaiveDate::from_ymd_opt(1974, 12, 31).unwrap()));
    assert!(Class::R10.in_range(NaiveDate::from_ymd_opt(1975, 1, 1).unwrap()));
    assert!(Class::R10.in_range(NaiveDate::from_ymd_opt(2003, 12, 31).unwrap()));
    assert!(!Class::R10.in_range(NaiveDate::from_ymd_opt(2004, 1, 1).unwrap()));
}
