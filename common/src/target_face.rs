use crate::class::Class;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetFace {
    Yellow,
    Blue,
    Red,
}

impl TargetFace {
    pub fn for_cls(cls: Class) -> &'static [TargetFace] {
        use Class::*;
        use TargetFace::*;
        match cls {
            R20 | C120 | B220 => &[Yellow],
            B230 => &[Yellow],
            R30 | C130 => &[Blue],
            B210 | B211 | B212 => &[Blue],
            R10 | R11 | R12 | R40 | C110 | C111 | C112 => &[Red],
            OO => &[Yellow],
        }
    }
}

impl std::fmt::Display for TargetFace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TargetFace::Yellow => "Gelber Pflock (5m-40m)",
                TargetFace::Blue => "Blauer Pflock (5m-50m)",
                TargetFace::Red => "Roter Pflock (10m-60m)",
            }
        )
    }
}
