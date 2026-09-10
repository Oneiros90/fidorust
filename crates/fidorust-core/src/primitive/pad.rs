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
