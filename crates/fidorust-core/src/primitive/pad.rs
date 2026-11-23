//! PCB pad style (FidoCAD `PA`).

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PadStyle {
    Oval = 0,
    Rectangular = 1,
    RoundedRect = 2,
}

impl PadStyle {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::Rectangular,
            2 => Self::RoundedRect,
            _ => Self::Oval,
        }
    }
}

impl fmt::Display for PadStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Oval => "Round",
            Self::Rectangular => "Square",
            Self::RoundedRect => "SquareRounded",
        })
    }
}

impl FromStr for PadStyle {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Square" => Ok(Self::Rectangular),
            "SquareRounded" => Ok(Self::RoundedRect),
            "Round" => Ok(Self::Oval),
            _ => Err(()),
        }
    }
}

/// Which board faces may bring a track to this pad.
/// `Both` is the default, so classic files stay solder-side routes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PadAccess {
    #[default]
    Both,
    Solder,
    Component,
}

impl PadAccess {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::Solder,
            2 => Self::Component,
            _ => Self::Both,
        }
    }

    pub fn to_i32(self) -> i32 {
        match self {
            Self::Both => 0,
            Self::Solder => 1,
            Self::Component => 2,
        }
    }

    /// `0` is the solder face, `1` the component face.
    pub fn allows(self, face: u8) -> bool {
        match self {
            Self::Both => true,
            Self::Solder => face == 0,
            Self::Component => face == 1,
        }
    }
}

impl fmt::Display for PadAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Both => "Both",
            Self::Solder => "Solder",
            Self::Component => "Component",
        })
    }
}

impl FromStr for PadAccess {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Both" => Ok(Self::Both),
            "Solder" => Ok(Self::Solder),
            "Component" => Ok(Self::Component),
            _ => Err(()),
        }
    }
}
