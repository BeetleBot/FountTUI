pub mod home;
pub mod xray;

use unicode_width::UnicodeWidthStr;
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
        .border_style(Style::default().fg(mode_bg));

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
    .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));

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

    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::LightMagenta)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
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
    f.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            dir_str,
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )])),
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
                (" ", Color::LightBlue)
            } else {
                ("󰈙 ", Color::White)
            };

            let style = if is_selected {
                Style::default().bg(Color::LightMagenta).fg(Color::Black)
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
            Style::default().bg(Color::LightGreen).fg(Color::Black)
        } else {
            Style::default().fg(Color::LightGreen)
        };
        display_items.push(ListItem::new(Line::from(vec![
            Span::styled(if is_selected { " › " } else { "   " }, style),
            Span::styled("󰒓 ", style),
            Span::styled(
                format!("Confirm: {}", state.filename_input),
                style.add_modifier(Modifier::BOLD),
            ),
        ])));
    }

    let list = List::new(display_items).highlight_style(Style::default());
    f.render_stateful_widget(list, layout[1], &mut state.list_state);

    // 3. Input Label
    if state.action != crate::app::FilePickerAction::Open {
        f.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                " Filename: ",
                Style::default().fg(Color::DarkGray),
            )])),
            layout[2],
        );

        // 4. Filename Input
        let input_style = Style::default().fg(Color::White).bg(Color::Rgb(30, 30, 30));
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
        .border_style(Style::default().fg(mode_bg));

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
            .bg(Color::Rgb(50, 50, 50))
            .add_modifier(Modifier::BOLD),
    );

    f.render_stateful_widget(table, modal_area, &mut app.snapshot_list_state);
}

