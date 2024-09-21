mod app;
mod constants;
mod dream;
mod interface;

use crate::dream::{Dream, Intensity, Style};
use app::{DreamApp, InputField, InputMode};
use constants::{DREAM_FILE, MAX_TRACK, TICK_RATE_DURATION};
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use interface::{draw_ui, INTENSITY_OPTIONS, STYLE_OPTIONS};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{
    error::Error,
    io::{self},
    sync::mpsc,
    thread,
    time::Duration,
};

enum Event<I> {
    Input(I),
    Tick,
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide,)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let app = DreamApp::new();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::Show,
    )?;


    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: DreamApp,
) -> Result<(), Box<dyn Error>> {
    let tick_rate = Duration::from_millis(TICK_RATE_DURATION);
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
        terminal.draw(|f| draw_ui(f, &mut app))?;

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
                            date: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
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
                            if app.selected >= app.visible_start + MAX_TRACK {
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
                    InputField::Experience => match event {
                        KeyEvent {
                            code: KeyCode::F(1),
                            modifiers: KeyModifiers::NONE,
                            kind: crossterm::event::KeyEventKind::Press,
                            state: crossterm::event::KeyEventState::NONE,
                        } => {
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
                                if app.dreams.len() > MAX_TRACK {
                                    app.visible_start = app.dreams.len() - MAX_TRACK;
                                } else {
                                    app.visible_start = 0;
                                }
                            }

                            app.input_mode = InputMode::Normal;
                            app.input_field = InputField::None;
                            app.editing_index = None;
                            app.unsaved_changes = true;
                        }
                        KeyEvent {
                            code: KeyCode::Enter,
                            modifiers: KeyModifiers::NONE,
                            kind: crossterm::event::KeyEventKind::Press,
                            state: crossterm::event::KeyEventState::NONE,
                        } => {
                            app.input.push('\n');
                        }
                        KeyEvent {
                            code: KeyCode::Char(c),
                            ..
                        } => {
                            app.input.push(c);
                        }
                        KeyEvent {
                            code: KeyCode::Backspace,
                            ..
                        } => {
                            app.input.pop();
                        }
                        KeyEvent {
                            code: KeyCode::Esc, ..
                        } => {
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

fn export_dreams(dreams: &Vec<Dream>) -> Result<(), Box<dyn Error>> {
    let serialized = serde_json::to_string_pretty(dreams)?;
    std::fs::write(DREAM_FILE, serialized)?;
    Ok(())
}
