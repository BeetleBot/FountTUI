pub mod home;
pub mod xray;
pub mod index_cards;

use unicode_width::UnicodeWidthStr;
use crate::theme::HexColor;
use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, List, ListItem},
};

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}


pub fn draw_sprint_stats(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = &app.theme;
    let mode_bg = Color::from(theme.ui.normal_mode_bg.clone());

    let modal_area = centered_rect(80, 60, area);
    f.render_widget(Clear, modal_area);

    let history_block = Block::default()
        .title(" [ Sprint History | Press E to Export ] ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::from(theme.ui.dim.clone())))
        .style(Style::default().bg(Color::from(theme.ui.background.clone().unwrap_or(HexColor("Reset".to_string())))).fg(Color::from(theme.ui.foreground.clone().unwrap_or(HexColor("White".to_string())))));

    let inner_area = modal_area.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });

    let header = Row::new(vec![
        Cell::from("Project"),
        Cell::from("Date"),
        Cell::from("Time"),
        Cell::from("Words"),
        Cell::from("Lines"),
    ])
    .style(
        Style::default()
            .bg(mode_bg)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = app
        .sprint_history
        .iter()
        .map(|s| {
            Row::new(vec![
                Cell::from(s.project_name.clone()),
                Cell::from(s.timestamp.format("%Y-%m-%d").to_string()),
                Cell::from(format!("{}m", s.duration_mins)),
                Cell::from(s.word_count.to_string()),
                Cell::from(s.line_count.to_string()),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(history_block)
    .row_highlight_style(Style::default().bg(Color::from(theme.ui.selection_bg.clone())).fg(Color::from(theme.ui.selection_fg.clone())));

    f.render_stateful_widget(table, inner_area, &mut app.sprint_stats_state);
}


pub fn draw_file_picker(f: &mut Frame, app: &mut App, area: Rect) {
    let state = if let Some(ref mut s) = app.file_picker {
        s
    } else {
        return;
    };

    let block_w = 70u16.min(area.width);
    let block_h = 24u16.min(area.height);
    let x = area.x + (area.width - block_w) / 2;
    let y = area.y + (area.height - block_h) / 2;
    let block_area = Rect::new(x, y, block_w, block_h);

    f.render_widget(Clear, block_area);

    let title = match state.action {
        crate::app::FilePickerAction::Open => " [ Open File ] ",
        crate::app::FilePickerAction::Save => " [ Save As ] ",
        crate::app::FilePickerAction::ExportReport => " [ Export Report ] ",
        crate::app::FilePickerAction::ExportScript => " [ Export Script ] ",
        crate::app::FilePickerAction::ExportSprints => " [ Export Sprints ] ",
    };

    let mode_bg = Color::from(app.theme.ui.normal_mode_bg.clone());
    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(mode_bg)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::from(app.theme.ui.dim.clone())))
        .style(Style::default().bg(Color::from(app.theme.ui.background.clone().unwrap_or(HexColor("Reset".to_string())))).fg(Color::from(app.theme.ui.foreground.clone().unwrap_or(HexColor("White".to_string())))));
    f.render_widget(block, block_area);

    let inner_area = block_area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    });

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Current Dir
            Constraint::Min(0),    // List
            Constraint::Length(1), // Input Label
            Constraint::Length(1), // Filename Input
        ])
        .split(inner_area);

    // 1. Current Dir
    let dir_str = format!(" Dir: {}", state.current_dir.display());
    let dir_style = if state.naming_mode {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::from(app.theme.ui.dim.clone()))
            .add_modifier(Modifier::ITALIC)
    };
    
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(dir_str, dir_style),
            if state.naming_mode {
                Span::styled(" [LOCKED]", Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD))
            } else {
                Span::raw("")
            }
        ])),
        layout[0],
    );

    // 2. List of items
    let items_len = state.items.len();
    let selected_idx = state.list_state.selected().unwrap_or(0);
    let mut display_items: Vec<ListItem> = state
        .items
        .iter()
        .enumerate()
        .map(|(i, path)| {
            let is_selected = i == selected_idx;
            let is_dir = path.is_dir();

            let name = if let Some(parent) = state.current_dir.parent() {
                if path == parent {
                    ".. (Parent Directory)".to_string()
                } else {
                    path.file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "/".to_string())
                }
            } else {
                path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "/".to_string())
            };

            let (icon, color) = if is_dir {
                if app.config.use_nerd_fonts {
                    (" ", Color::LightBlue)
                } else {
                    ("/ ", Color::LightBlue)
                }
            } else {
                if app.config.use_nerd_fonts {
                    ("󰈙 ", Color::White)
                } else {
                    ("  ", Color::White)
                }
            };

            let style = if is_selected {
                Style::default().bg(Color::from(app.theme.ui.selection_bg.clone())).fg(Color::from(app.theme.ui.selection_fg.clone()))
            } else {
                Style::default().fg(color)
            };

            ListItem::new(Line::from(vec![
                Span::styled(if is_selected { " › " } else { "   " }, style),
                Span::styled(icon, style),
                Span::styled(name, style),
            ]))
        })
        .collect();

    // Add virtual item for custom filename if in Save mode
    if state.action != crate::app::FilePickerAction::Open && !state.filename_input.is_empty() {
        let is_selected = selected_idx == items_len;
        let style = if is_selected {
            Style::default().bg(Color::from(app.theme.ui.selection_bg.clone())).fg(Color::from(app.theme.ui.selection_fg.clone()))
        } else {
            Style::default().fg(Color::from(app.theme.ui.normal_mode_bg.clone()))
        };
        display_items.push(ListItem::new(Line::from(vec![
            Span::styled(if is_selected { " › " } else { "   " }, style),
            Span::styled(if app.config.use_nerd_fonts { "󰒓 " } else { "* " }, style),
            Span::styled(
                format!("Confirm: {}", state.filename_input),
                style.add_modifier(Modifier::BOLD),
            ),
        ])));
    }

    let list = List::new(display_items).highlight_style(Style::default());
    f.render_stateful_widget(list, layout[1], &mut state.list_state);

    // 3. Input Label & Hints
    if state.action != crate::app::FilePickerAction::Open {
        let hints = if state.naming_mode {
            Line::from(vec![
                Span::styled(" Filename: ", Style::default().fg(Color::from(app.theme.ui.normal_mode_bg.clone())).add_modifier(Modifier::BOLD)),
                Span::styled(" [Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("to SAVE to locked folder", Style::default().fg(Color::from(app.theme.ui.dim.clone()))),
            ])
        } else {
            Line::from(vec![
                Span::styled(" Filename: ", Style::default().fg(Color::from(app.theme.ui.dim.clone()))),
                Span::styled(" [Tab] ", Style::default().fg(Color::from(app.theme.ui.normal_mode_bg.clone())).add_modifier(Modifier::BOLD)),
                Span::styled("to LOCK folder & type name", Style::default().fg(Color::from(app.theme.ui.dim.clone()))),
            ])
        };
        f.render_widget(Paragraph::new(hints), layout[2]);

        // 4. Filename Input
        let input_style = Style::default().fg(Color::from(app.theme.ui.selection_fg.clone())).bg(Color::from(app.theme.ui.selection_bg.clone()));
        f.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                format!("  {}", state.filename_input),
                input_style,
            )]))
            .block(Block::default().borders(Borders::NONE)),
            layout[3],
        );

        // Cursor for input
        let cursor_pos =
            layout[3].x + 2 + UnicodeWidthStr::width(state.filename_input.as_str()) as u16;
        f.set_cursor_position((cursor_pos, layout[3].y));
    }

    // Overwrite Confirmation Overlay
    if state.show_overwrite_confirm {
        let confirm_area = centered_rect(60, 30, area);
        f.render_widget(Clear, confirm_area);
        let confirm_block = Block::default()
            .title(Span::styled(" [ Confirm Overwrite ] ", Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)))
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Double)
            .border_style(Style::default().fg(Color::LightRed))
            .style(Style::default().bg(Color::from(app.theme.ui.background.clone().unwrap_or(HexColor("Reset".to_string())))));
        
        let file_name = state.target_path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy()).unwrap_or_default();
        
        let confirm_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw(" File "),
                Span::styled(file_name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" already exists!"),
            ]),
            Line::from(""),
            Line::from(" Would you like to overwrite it?"),
            Line::from(""),
            Line::from(vec![
                if state.overwrite_confirmed {
                    Span::styled("  ▶ YES  ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD))
                } else {
                    Span::styled("    Yes  ", Style::default().fg(Color::Green))
                },
                Span::raw("      "),
                if !state.overwrite_confirmed {
                    Span::styled("  ▶ NO   ", Style::default().bg(Color::Red).fg(Color::Black).add_modifier(Modifier::BOLD))
                } else {
                    Span::styled("    No   ", Style::default().fg(Color::Red))
                },
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" [←/→] ", Style::default().fg(Color::from(app.theme.ui.dim.clone()))),
                Span::raw("Switch  "),
                Span::styled(" [Enter] ", Style::default().fg(Color::from(app.theme.ui.dim.clone()))),
                Span::raw("Confirm"),
            ]),
        ];
        
        f.render_widget(Paragraph::new(confirm_text).block(confirm_block).alignment(ratatui::layout::Alignment::Center), confirm_area);
    }
}


pub fn draw_snapshots(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = &app.theme;
    let mode_bg = Color::from(theme.ui.normal_mode_bg.clone());

    let modal_area = centered_rect(70, 60, area);
    f.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(" [ Snapshots | Enter: Replace | O: Open in New ] ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::from(theme.ui.dim.clone())))
        .style(Style::default().bg(Color::from(theme.ui.background.clone().unwrap_or(HexColor("Reset".to_string())))).fg(Color::from(theme.ui.foreground.clone().unwrap_or(HexColor("White".to_string())))));

    let header = Row::new(vec![
        Cell::from("File Name"),
        Cell::from("Date"),
        Cell::from("Time"),
    ])
    .style(
        Style::default()
            .bg(mode_bg)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = app
        .snapshots
        .iter()
        .map(|s| {
            Row::new(vec![
                Cell::from(s.display_stem()),
                Cell::from(s.display_date()),
                Cell::from(s.display_time_only()),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .header(header)
    .block(block)
    .row_highlight_style(
        Style::default()
            .bg(Color::from(theme.ui.selection_bg.clone()))
            .fg(Color::from(theme.ui.selection_fg.clone()))
            .add_modifier(Modifier::BOLD),
    );

    f.render_stateful_widget(table, modal_area, &mut app.snapshot_list_state);
}

