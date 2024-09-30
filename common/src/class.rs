use std::{fmt::Display, str::FromStr};

use crate::bow_type::BowType;
use chrono::{Months, NaiveDate};
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

lazy_static! {
    pub static ref SEASON_START: NaiveDate = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::EnumIter, PartialOrd, Hash,
)]
pub enum Class {
    R10,
    R11,
    R20,
    R21,
    R22,
    R23,
    R24,
    R25,
    R30,
    R31,
    R40,
    R41,
    R12,
    R13,
    R14,
    R15,
    B210,
    B211,
    B212,
    B220,
    B230,
    C110,
    C111,
    C120,
    C130,
    C140,
    C112,
    C113,
    C114,
    O,
}

impl Class {
    pub fn name(&self) -> &'static str {
        match self {
            Class::R10 => "Recurve Herren",
            Class::R11 => "Recurve Damen",
            Class::R20 => "Recurve Schüler A m",
            Class::R21 => "Recurve Schüler A w",
            Class::R22 => "Recurve Schüler B m",
            Class::R23 => "Recurve Schüler B w",
            Class::R24 => "Recurve Schüler C m",
            Class::R25 => "Recurve Schüler C w",
            Class::R30 => "Recurve Jugend m",
            Class::R31 => "Recurve Jugend w",
            Class::R40 => "Recurve Junioren m",
            Class::R41 => "Recurve Junioren w",
            Class::R12 => "Recurve Master m",
            Class::R13 => "Recurve Master w",
            Class::R14 => "Recurve Senioren m",
            Class::R15 => "Recurve Senioren w",

            Class::B210 => "Blank Herren",
            Class::B211 => "Blank Damen",
            Class::B220 => "Blank Schüler m/w",
            Class::B230 => "Blank Jugend m/w",
            Class::B212 => "Blank Master m",

            Class::C110 => "Compound Herren",
            Class::C111 => "Compound Damen",
            Class::C120 => "Compound Schüler m/w",
            Class::C130 => "Compound Jugend m/w",
            Class::C140 => "Compound Junioren m/w",
            Class::C112 => "Compound Master m",
            Class::C113 => "Compound Master w",
            Class::C114 => "Compound Senioren m",

            Class::O => "Offene Klasse",
        }
    }
    pub fn comment(&self) -> &'static str {
        ""
    }
    pub fn all_classes() -> impl Iterator<Item = Self> {
        Self::iter()
    }
    pub fn recurve_classes() -> &'static [Self] {
        &[
            Self::R10,
            Self::R11,
            Self::R20,
            Self::R21,
            Self::R22,
            Self::R23,
            Self::R24,
            Self::R25,
            Self::R30,
            Self::R31,
            Self::R40,
            Self::R41,
            Self::R12,
            Self::R13,
            Self::R14,
            Self::R15,
            Self::O,
        ]
    }
    pub fn barebow_classes() -> &'static [Self] {
        &[Self::B210, Self::B211, Self::B220, Self::B230, Self::B212]
    }
    pub fn compound_classes() -> &'static [Self] {
        &[
            Self::C110,
            Self::C111,
            Self::C120,
            Self::C130,
            Self::C140,
            Self::C112,
            Self::C113,
            Self::C114,
        ]
    }
    pub fn in_range(&self, dob: NaiveDate) -> bool {
        let year_range = match self {
            Class::R10 => (21, 49),
            Class::R11 => (21, 49),
            Class::R20 => (13, 14),
            Class::R21 => (13, 14),
            Class::R22 => (11, 12),
            Class::R23 => (11, 12),
            Class::R24 => (1, 10),
            Class::R25 => (1, 10),
            Class::R30 => (15, 17),
            Class::R31 => (15, 17),
            Class::R40 => (18, 20),
            Class::R41 => (18, 20),
            Class::R12 => (50, 65),
            Class::R13 => (50, 65),
            Class::R14 => (66, 120),
            Class::R15 => (66, 120),

            Class::C110 => (21, 49),
            Class::C111 => (21, 49),
            Class::C120 => (1, 14),
            Class::C130 => (15, 17),
            Class::C140 => (18, 20),
            Class::C112 => (50, 65),
            Class::C113 => (50, 120),
            Class::C114 => (66, 120),

            Class::B210 => (21, 49),
            Class::B211 => (21, 120),
            Class::B220 => (1, 14),
            Class::B230 => (15, 20),
            Class::B212 => (50, 120),

            Class::O => (1, 120),
        };

        let date_range = (*SEASON_START - Months::new(year_range.1 * 12))
            ..(*SEASON_START - Months::new((year_range.0 - 1) * 12));
        date_range.contains(&dob)
    }

    pub fn allowed_classes(bow_type: BowType, dob: NaiveDate) -> Vec<(Class, ClassUpgradeStatus)> {
        let default_classes = match bow_type {
            BowType::Recurve => Class::recurve_classes(),
            BowType::Compound => Class::compound_classes(),
            BowType::Barebow => Class::barebow_classes(),
        }
        .iter()
        .filter(|cls| cls.in_range(dob));

        let upgrade_classes = default_classes
            .clone()
            .map(|dc| dc.other_allowed_classes().into_iter())
            .flatten()
            .unique();
        default_classes
            .map(|&c| (c, ClassUpgradeStatus::InDefaultAgeRange))
            .chain(upgrade_classes.map(|&c| (c, ClassUpgradeStatus::Upgrade)))
            .collect()
    }

    fn other_allowed_classes(&self) -> &'static [Self] {
        use Class::*;
        match self {
            // Junioren + Master => Herren/Damen
            R40 | R12 => &[R10],
            R41 | R13 => &[R11],
            R14 => &[R12, R10],
            R15 => &[R13, R11],

            B212 => &[B210],

            C140 => &[C110, C111],
            C112 => &[C110],
            C113 => &[C111],
            C114 => &[C112, C110],

            _ => &[],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClassUpgradeStatus {
    InDefaultAgeRange,
    Upgrade,
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
    assert!(!Class::R10.in_range(NaiveDate::from_ymd_opt(1975, 12, 31).unwrap()));
    assert!(Class::R10.in_range(NaiveDate::from_ymd_opt(1976, 1, 1).unwrap()));
    assert!(Class::R10.in_range(NaiveDate::from_ymd_opt(2004, 12, 31).unwrap()));
    assert!(!Class::R10.in_range(NaiveDate::from_ymd_opt(2005, 1, 1).unwrap()));
}
