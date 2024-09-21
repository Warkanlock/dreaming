use crate::{constants::DREAM_FILE, dream::{Dream, Intensity, Style}};

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    ConfirmExport,
    ConfirmDelete,
    ConfirmQuit,
    ViewingDream,
}

pub enum InputField {
    Intensity,
    Frequency,
    Style,
    Experience,
    None,
}

pub struct DreamApp {
    pub dreams: Vec<Dream>,
    pub input_mode: InputMode,
    pub input_field: InputField,
    pub input: String,
    pub current_dream: Dream,
    pub selected: usize,
    pub visible_start: usize,
    pub selection_index: usize,
    pub frequency_value: u8,
    pub editing_index: Option<usize>,
    pub unsaved_changes: bool, 
}

impl DreamApp {
    pub fn new() -> DreamApp {
        let dreams = match std::fs::read_to_string(DREAM_FILE) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_else(|_| Vec::new()),
            Err(_) => Vec::new(),
        };

        DreamApp {
            dreams,
            input_mode: InputMode::Normal,
            input_field: InputField::None,
            input: String::new(),
            current_dream: Dream {
                date: "N/A".to_string(),
                intensity: Intensity::Low,
                experience: String::new(),
                frequency: 0,
                style: Style::Lucid,
            },
            selected: 0,
            visible_start: 0,
            selection_index: 0,
            frequency_value: 0,
            editing_index: None,
            unsaved_changes: false,
        }
    }
}

