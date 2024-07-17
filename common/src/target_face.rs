use crate::class::Class;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetFace {
    M60cm122,
    M50cm80,
    M40cm122,
    M20cm122,
}

impl TargetFace {
    pub fn for_cls(cls: Class) -> &'static [TargetFace] {
        use Class::*;
        use TargetFace::*;
        match cls {
            RSU => &[M20cm122],
            RST => &[M40cm122],
            CC => &[M50cm80],
            RP => &[M60cm122],
        }
    }
}

impl std::fmt::Display for TargetFace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TargetFace::M60cm122 => "60m / 122cm",
                TargetFace::M50cm80 => "50m / 80cm",
                TargetFace::M40cm122 => "40m / 122cm",
                TargetFace::M20cm122 => "20m / 122cm",
            }
        )
    }
}
