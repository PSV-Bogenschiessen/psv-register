use crate::class::Class;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetFace {
    M70cm122,
    M60cm122,
    M50cm122,
    M40cm122,
    M40cm80,
    M25cm80,
    M18cm122,
    M50Spot10_5,
}

impl TargetFace {
    pub fn for_cls(cls: Class) -> &'static [TargetFace] {
        use Class::*;
        use TargetFace::*;
        match cls {
            R10 | R11 | R40 | R41 => &[M70cm122],
            R12 | R13 | R30 | R31 => &[M60cm122],
            R14 | R15 => &[M50cm122],
            R20 | R21 => &[M40cm122],
            R22 | R23 => &[M25cm80],
            R24 | R25 => &[M18cm122],
            C110 | C111 | C112 | C113 | C114 | C115 | C130 | C140 => &[M50Spot10_5],
            C120 => &[M40cm80],
            B210 | B211 | B212 | B213 | B214 | B230 => &[M50cm122],
            B220 => &[M25cm80],
            OO => &[M25cm80],
        }
    }
}

impl std::fmt::Display for TargetFace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TargetFace::M70cm122 => "70m / 122cm",
                TargetFace::M60cm122 => "60m / 122cm",
                TargetFace::M50cm122 => "50m / 122cm",
                TargetFace::M40cm122 => "40m / 122cm",
                TargetFace::M40cm80 => "40m / 80cm",
                TargetFace::M25cm80 => "25m / 80cm",
                TargetFace::M18cm122 => "18m / 122cm",
                TargetFace::M50Spot10_5 => "50m / Spot(10-5)",
            }
        )
    }
}
