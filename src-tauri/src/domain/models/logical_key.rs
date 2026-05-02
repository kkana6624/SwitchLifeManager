use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogicalKey {
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    E1,
    E2,
    E3,
    E4,
    Other(u16),
}

impl fmt::Display for LogicalKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicalKey::Key1 => write!(f, "Key1"),
            LogicalKey::Key2 => write!(f, "Key2"),
            LogicalKey::Key3 => write!(f, "Key3"),
            LogicalKey::Key4 => write!(f, "Key4"),
            LogicalKey::Key5 => write!(f, "Key5"),
            LogicalKey::Key6 => write!(f, "Key6"),
            LogicalKey::Key7 => write!(f, "Key7"),
            LogicalKey::E1 => write!(f, "E1"),
            LogicalKey::E2 => write!(f, "E2"),
            LogicalKey::E3 => write!(f, "E3"),
            LogicalKey::E4 => write!(f, "E4"),
            LogicalKey::Other(id) => write!(f, "Other-{}", id),
        }
    }
}

impl FromStr for LogicalKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Key1" => Ok(LogicalKey::Key1),
            "Key2" => Ok(LogicalKey::Key2),
            "Key3" => Ok(LogicalKey::Key3),
            "Key4" => Ok(LogicalKey::Key4),
            "Key5" => Ok(LogicalKey::Key5),
            "Key6" => Ok(LogicalKey::Key6),
            "Key7" => Ok(LogicalKey::Key7),
            "E1" => Ok(LogicalKey::E1),
            "E2" => Ok(LogicalKey::E2),
            "E3" => Ok(LogicalKey::E3),
            "E4" => Ok(LogicalKey::E4),
            _ => {
                if let Some(rest) = s.strip_prefix("Other-") {
                    let id = rest.parse::<u16>().map_err(|_| format!("Invalid Other ID: {}", rest))?;
                    Ok(LogicalKey::Other(id))
                } else {
                    Err(format!("Unknown LogicalKey: {}", s))
                }
            }
        }
    }
}
