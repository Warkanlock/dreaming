use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum Intensity {
    Low,
    Medium,
    High,
}

impl fmt::Display for Intensity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Intensity::Low => write!(f, "Low"),
            Intensity::Medium => write!(f, "Medium"),
            Intensity::High => write!(f, "High"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Style {
    Lucid,
    Nightmare,
    Recurring,
    Prophetic,
    Normal, 
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Style::Lucid => write!(f, "Lucid"),
            Style::Nightmare => write!(f, "Nightmare"),
            Style::Recurring => write!(f, "Recurring"),
            Style::Prophetic => write!(f, "Prophetic"),
            Style::Normal => write!(f, "Normal"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Dream {
    pub date: String,
    pub intensity: Intensity,
    pub experience: String,
    pub frequency: u8,
    pub style: Style,
}
