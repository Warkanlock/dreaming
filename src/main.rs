mod dream;

use crate::dream::{Dream, Intensity, Style};
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color as TuiColor, Modifier, Style as TuiStyle},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::{
    error::Error,
    io::{self},
    sync::mpsc,
    thread,
    time::Duration,
};

#[derive(PartialEq)]
enum InputMode {
    Normal,
    Editing,
    ConfirmExport,
    ConfirmDelete,
    ConfirmQuit,
    ViewingDream,
}

enum InputField {
    Intensity,
    Frequency,
    Style,
    Experience,
    None,
}

struct App {
    dreams: Vec<Dream>,
    input_mode: InputMode,
    input_field: InputField,
    input: String,
    current_dream: Dream,
    selected: usize,
    visible_start: usize,
    selection_index: usize,
    frequency_value: u8,
    editing_index: Option<usize>,
    unsaved_changes: bool, 
}

impl App {
    fn new() -> App {
        let dreams = match std::fs::read_to_string("dreams_export.json") {
            Ok(data) => serde_json::from_str(&data).unwrap_or_else(|_| Vec::new()),
            Err(_) => Vec::new(),
        };

        App {
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

const INTENSITY_OPTIONS: &[Intensity] = &[Intensity::Low, Intensity::Medium, Intensity::High];
const STYLE_OPTIONS: &[Style] = &[
    Style::Lucid,
    Style::Nightmare,
    Style::Recurring,
    Style::Prophetic,
    Style::Normal,
];

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::cursor::Hide,
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::Show,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

enum Event<I> {
    Input(I),
    Tick,
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> Result<(), Box<dyn Error>> {
    let tick_rate = Duration::from_millis(250);
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || loop {
        if event::poll(tick_rate).unwrap() {
            if let CEvent::Key(key) = event::read().unwrap() {
                tx.send(Event::Input(key)).unwrap();
            }
        }
        tx.send(Event::Tick).unwrap();
    });

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        match rx.recv()? {
            Event::Input(event) => match app.input_mode {
                InputMode::Normal => match event.code {
                    KeyCode::Char('q') => {
                        app.input_mode = InputMode::ConfirmQuit;
                    }
                    KeyCode::Char('a') => {
                        app.input_mode = InputMode::Editing;
                        app.input_field = InputField::Intensity;
                        app.selection_index = 0;
                        app.frequency_value = 0;
                        app.input.clear();
                        app.current_dream = Dream {
                            date: "Today".to_string(),
                            intensity: Intensity::Low,
                            experience: String::new(),
                            frequency: 0,
                            style: Style::Lucid,
                        };
                        app.editing_index = None;
                    }
                    KeyCode::Char('d') => {
                        if !app.dreams.is_empty() {
                            app.input_mode = InputMode::ConfirmDelete;
                        }
                    }
                    KeyCode::Char('s') => {
                        app.input_mode = InputMode::ConfirmExport;
                    }
                    KeyCode::Char('e') => {
                        if !app.dreams.is_empty() {
                            app.input_mode = InputMode::Editing;
                            app.input_field = InputField::Intensity;
                            app.current_dream = app.dreams[app.selected].clone();
                            app.editing_index = Some(app.selected);
                            app.selection_index = INTENSITY_OPTIONS
                                .iter()
                                .position(|i| *i == app.current_dream.intensity)
                                .unwrap_or(0);
                            app.frequency_value = app.current_dream.frequency;
                            app.input = app.current_dream.experience.clone();
                        }
                    }
                    KeyCode::Right => {
                        if app.selected < app.dreams.len().saturating_sub(1) {
                            app.selected += 1;
                            if app.selected >= app.visible_start + 7 {
                                app.visible_start += 1;
                            }
                        }
                    }
                    KeyCode::Left => {
                        if app.selected > 0 {
                            app.selected -= 1;
                            if app.selected < app.visible_start {
                                app.visible_start = app.visible_start.saturating_sub(1);
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if !app.dreams.is_empty() {
                            app.input_mode = InputMode::ViewingDream;
                        }
                    }
                    _ => {}
                },
                InputMode::Editing => match app.input_field {
                    InputField::Intensity | InputField::Style => match event.code {
                        KeyCode::Up => {
                            if app.selection_index > 0 {
                                app.selection_index -= 1;
                            }
                        }
                        KeyCode::Down => {
                            let options_len = match app.input_field {
                                InputField::Intensity => INTENSITY_OPTIONS.len(),
                                InputField::Style => STYLE_OPTIONS.len(),
                                _ => 0,
                            };
                            if app.selection_index < options_len - 1 {
                                app.selection_index += 1;
                            }
                        }
                        KeyCode::Enter => {
                            match app.input_field {
                                InputField::Intensity => {
                                    app.current_dream.intensity =
                                        INTENSITY_OPTIONS[app.selection_index].clone();
                                    app.input_field = InputField::Frequency;
                                    if app.editing_index.is_none() {
                                        app.frequency_value = 0;
                                    }
                                }
                                InputField::Style => {
                                    app.current_dream.style =
                                        STYLE_OPTIONS[app.selection_index].clone();
                                    app.input_field = InputField::Experience;
                                    if app.editing_index.is_none() {
                                        app.input.clear();
                                    }
                                }
                                _ => {}
                            }
                            app.selection_index = 0;
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                            app.input_field = InputField::None;
                            app.editing_index = None;
                        }
                        _ => {}
                    },
                    InputField::Frequency => match event.code {
                        KeyCode::Up => {
                            if app.frequency_value < 10 {
                                app.frequency_value += 1;
                            }
                        }
                        KeyCode::Down => {
                            if app.frequency_value > 0 {
                                app.frequency_value -= 1;
                            }
                        }
                        KeyCode::Enter => {
                            app.current_dream.frequency = app.frequency_value;
                            app.input_field = InputField::Style;
                            app.selection_index = 0;
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                            app.input_field = InputField::None;
                            app.editing_index = None;
                        }
                        _ => {}
                    },
                    InputField::Experience => match event.code {
                        KeyCode::Enter => {
                            if app.input.trim().is_empty() {
                                app.current_dream.experience = "N/A".to_string();
                            } else {
                                app.current_dream.experience = app.input.drain(..).collect();
                            }
                            if let Some(index) = app.editing_index {
                                app.dreams[index] = app.current_dream.clone();
                            } else {
                                app.dreams.push(app.current_dream.clone());
                                app.selected = app.dreams.len() - 1;
                                if app.dreams.len() > 7 {
                                    app.visible_start = app.dreams.len() - 7;
                                } else {
                                    app.visible_start = 0;
                                }
                            }
                            app.input_mode = InputMode::Normal;
                            app.input_field = InputField::None;
                            app.editing_index = None;
                            app.unsaved_changes = true;
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                            app.input_field = InputField::None;
                            app.editing_index = None;
                        }
                        _ => {}
                    },
                    _ => {}
                },
                InputMode::ConfirmExport => match event.code {
                    KeyCode::Char('y') => {
                        export_dreams(&app.dreams)?;
                        app.input_mode = InputMode::Normal;
                        app.unsaved_changes = false;
                    }
                    KeyCode::Char('n') | KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                InputMode::ConfirmDelete => match event.code {
                    KeyCode::Char('y') => {
                        if !app.dreams.is_empty() {
                            app.dreams.remove(app.selected);
                            if app.selected > 0 {
                                app.selected -= 1;
                            }
                            if app.visible_start > 0 && app.selected < app.visible_start {
                                app.visible_start -= 1;
                            }
                            app.unsaved_changes = true;
                        }
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char('n') | KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                InputMode::ConfirmQuit => match event.code {
                    KeyCode::Char('y') => {
                        return Ok(());
                    }
                    KeyCode::Char('n') | KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                InputMode::ViewingDream => match event.code {
                    KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
            },
            Event::Tick => {}
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();
    let background = Block::default().style(TuiStyle::default().bg(TuiColor::Rgb(0, 0, 50)));
    f.render_widget(background, size);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(5),
                Constraint::Percentage(85),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(size);

    
    let save_status = if app.unsaved_changes {
        Paragraph::new("Changes ●")
            .style(TuiStyle::default().fg(TuiColor::Red))
            .alignment(Alignment::Right)
    } else {
        Paragraph::new("Up to date ●")
            .style(TuiStyle::default().fg(TuiColor::Green))
            .alignment(Alignment::Right)
    };

    let logo = Paragraph::new("Dreaming Journal")
        .block(Block::default())
        .style(
            TuiStyle::default()
                .fg(TuiColor::Cyan)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::ITALIC),
        )
        .alignment(Alignment::Left);

    let header = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(chunks[0]);

    f.render_widget(logo, header[0]);
    f.render_widget(save_status, header[1]);

    let days_constraints = vec![Constraint::Percentage(100 / 7); 7];

    let days_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(days_constraints.clone())
        .split(chunks[1]);

    for (i, chunk) in days_chunks.iter().enumerate() {
        let dream_index = app.visible_start + i;
        let dream = app.dreams.get(dream_index);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Day {}", dream_index + 1))
            .style(TuiStyle::default().bg(TuiColor::Rgb(0, 0, 50)));
        if let Some(dream) = dream {
            let content = format!(
                "{}\nIntensity: {}\nFrequency: {}\nStyle: {}",
                dream.date, dream.intensity, dream.frequency, dream.style
            );
            let list_item = ListItem::new(content).style(TuiStyle::default().fg(TuiColor::Gray));

            let mut state = ratatui::widgets::ListState::default();
            if app.selected == dream_index {
                state.select(Some(0));
            }
            let dream_list = List::new(vec![list_item])
                .block(block)
                .highlight_style(TuiStyle::default().add_modifier(Modifier::REVERSED));
            f.render_stateful_widget(dream_list, *chunk, &mut state);
        } else {
            let empty_paragraph = Paragraph::new("No Dream")
                .block(block)
                .style(TuiStyle::default().fg(TuiColor::DarkGray));
            f.render_widget(empty_paragraph, *chunk);
        }
    }

    let instructions = Paragraph::new(
        "Press 'a' to add, 'e' to edit, 'd' to delete, 's' to save, 'q' to quit.\nUse Left/Right to navigate.",
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Instructions")
            .style(TuiStyle::default().bg(TuiColor::Rgb(100, 216, 230))), 
    )
    .style(TuiStyle::default().fg(TuiColor::Black));

    f.render_widget(instructions, chunks[2]);

    match app.input_mode {
        InputMode::Editing => {
            let area = centered_rect(60, 40, size);

            
            let shadow_area = Rect {
                x: area.x.saturating_sub(1),
                y: area.y.saturating_sub(1),
                width: area.width + 2,
                height: area.height + 2,
            };
            let shadow = Block::default().style(TuiStyle::default().bg(TuiColor::Rgb(0, 0, 40)));
            f.render_widget(shadow, shadow_area);

            f.render_widget(Clear, area);
            let input_field_title = match app.input_field {
                InputField::Intensity => "Select the intensity of your dream",
                InputField::Style => "Select the style",
                InputField::Frequency => "Set frequency (0-10) (Up/Down)",
                InputField::Experience => "Describe the experience",
                _ => "",
            };

            let input_block = Block::default()
                .borders(Borders::ALL)
                .title(input_field_title)
                .style(TuiStyle::default().bg(TuiColor::Rgb(0, 0, 50)));

            match app.input_field {
                InputField::Intensity => {
                    let options: Vec<ListItem> = INTENSITY_OPTIONS
                        .iter()
                        .map(|opt| ListItem::new(opt.to_string()))
                        .collect();
                    let options_list = List::new(options)
                        .highlight_style(
                            TuiStyle::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(TuiColor::Gray),
                        )
                        .highlight_symbol(">> ");

                    let mut selection_state = ratatui::widgets::ListState::default();
                    selection_state.select(Some(app.selection_index));

                    f.render_stateful_widget(
                        options_list.block(input_block),
                        area,
                        &mut selection_state,
                    );
                }
                InputField::Style => {
                    let options: Vec<ListItem> = STYLE_OPTIONS
                        .iter()
                        .map(|opt| ListItem::new(opt.to_string()))
                        .collect();
                    let options_list = List::new(options)
                        .highlight_style(
                            TuiStyle::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(TuiColor::Gray),
                        )
                        .highlight_symbol(">> ");

                    let mut selection_state = ratatui::widgets::ListState::default();
                    selection_state.select(Some(app.selection_index));

                    f.render_stateful_widget(
                        options_list.block(input_block),
                        area,
                        &mut selection_state,
                    );
                }
                InputField::Frequency => {
                    let frequency_display = format!("Frequency: {}", app.frequency_value);
                    let frequency_paragraph = Paragraph::new(frequency_display)
                        .block(input_block)
                        .alignment(ratatui::layout::Alignment::Center)
                        .style(TuiStyle::default().fg(TuiColor::Gray));

                    f.render_widget(frequency_paragraph, area);
                }
                InputField::Experience => {
                    let input = Paragraph::new(app.input.as_ref())
                        .style(TuiStyle::default().fg(TuiColor::Gray))
                        .block(input_block);

                    f.render_widget(input, area);

                    f.set_cursor(area.x + app.input.len() as u16 + 1, area.y + 1);
                }
                _ => {}
            }
        }
        InputMode::ConfirmExport | InputMode::ConfirmDelete | InputMode::ConfirmQuit => {
            let area = centered_rect(60, 10, size);

            
            let shadow_area = Rect {
                x: area.x.saturating_sub(1),
                y: area.y.saturating_sub(1),
                width: area.width + 2,
                height: area.height + 2,
            };
            let shadow = Block::default().style(TuiStyle::default().bg(TuiColor::Rgb(0, 0, 40)));
            f.render_widget(shadow, shadow_area);

            f.render_widget(Clear, area);
            let (title, message) = match app.input_mode {
                InputMode::ConfirmExport => (
                    "Confirm Save",
                    "Are you sure you want to save? (y/n)",
                ),
                InputMode::ConfirmDelete => (
                    "Confirm Delete",
                    "Are you sure you want to delete this dream? (y/n)",
                ),
                InputMode::ConfirmQuit => (
                    "Confirm Quit",
                    "Are you sure you want to quit? (y/n)",
                ),
                _ => ("", ""),
            };
            let confirm = Paragraph::new(message)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(title)
                        .style(TuiStyle::default().bg(TuiColor::Rgb(0, 0, 50))),
                )
                .style(TuiStyle::default().fg(TuiColor::Gray));
            f.render_widget(confirm, area);
        }
        InputMode::ViewingDream => {
            let area = centered_rect(60, 60, size);

            let shadow_area = Rect {
                x: area.x.saturating_sub(1),
                y: area.y.saturating_sub(1),
                width: area.width + 2,
                height: area.height + 2,
            };
            let shadow = Block::default().style(TuiStyle::default().bg(TuiColor::Rgb(0, 0, 40)));
            f.render_widget(shadow, shadow_area);

            f.render_widget(Clear, area);

            if let Some(dream) = app.dreams.get(app.selected) {
                let content = format!(
                    "Date: {}\nIntensity: {}\nFrequency: {}\nStyle: {}\nExperience: {}",
                    dream.date, dream.intensity, dream.frequency, dream.style, dream.experience
                );

                let paragraph = Paragraph::new(content)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Dream Details")
                            .style(TuiStyle::default().bg(TuiColor::Rgb(0, 0, 50))),
                    )
                    .style(TuiStyle::default().fg(TuiColor::Gray));

                f.render_widget(paragraph, area);
            }
        }
        _ => {}
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);
    let vertical_chunk = popup_layout[1];
    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(vertical_chunk);
    horizontal_layout[1]
}

fn export_dreams(dreams: &Vec<Dream>) -> Result<(), Box<dyn Error>> {
    let serialized = serde_json::to_string_pretty(dreams)?;
    std::fs::write("dreams.json", serialized)?;
    Ok(())
}
