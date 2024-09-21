use crate::{app::{DreamApp, InputField, InputMode}, constants::MAX_TRACK, dream::{Intensity, Style}};

use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color as TuiColor, Modifier, Style as TuiStyle},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub const INTENSITY_OPTIONS: &[Intensity] = &[Intensity::Low, Intensity::Medium, Intensity::High];

pub const STYLE_OPTIONS: &[Style] = &[
    Style::Lucid,
    Style::Nightmare,
    Style::Recurring,
    Style::Prophetic,
    Style::Normal,
];

pub fn draw_ui<B: Backend>(f: &mut Frame<B>, app: &mut DreamApp) {
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

    let days_constraints = vec![Constraint::Percentage(100 / MAX_TRACK as u16); MAX_TRACK];

    let days_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(days_constraints.clone())
        .split(chunks[1]);

    for (i, chunk) in days_chunks.iter().enumerate() {
        let dream_index = app.visible_start + i;
        let dream = app.dreams.get(dream_index);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Record {}", dream_index + 1))
            .style(TuiStyle::default().bg(TuiColor::Rgb(0, 0, 50)));

        if let Some(dream) = dream {
            let intensity_color = match dream.intensity {
                Intensity::Low => TuiColor::Green,
                Intensity::Medium => TuiColor::Yellow,
                Intensity::High => TuiColor::Red,
            };

            let content = format!(
                "Dreamed at:\n{}\n\nIntensity: {}\nFrequency: {}\nStyle: {}",
                dream.date, dream.intensity, dream.frequency, dream.style
            );

            let list_item = ListItem::new(content).style(TuiStyle::default().fg(TuiColor::White).fg(intensity_color));

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
                InputField::Experience => "Describe the experience (F1 to save)",
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
                        .block(input_block)
                        .wrap(ratatui::widgets::Wrap { trim: false });

                    f.render_widget(input, area);
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
                    "Date: {}\nIntensity: {}\nFrequency: {}\nStyle: {}\nExperience:\n{}",
                    dream.date, dream.intensity, dream.frequency, dream.style, dream.experience
                );

                let paragraph = Paragraph::new(content)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Dream Details")
                            .style(TuiStyle::default().bg(TuiColor::Rgb(0, 0, 50))),
                    )
                    .style(TuiStyle::default().fg(TuiColor::Gray))
                    .wrap(ratatui::widgets::Wrap { trim: false });

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

