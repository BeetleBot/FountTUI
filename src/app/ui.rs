use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::collections::HashSet;
use unicode_width::UnicodeWidthStr;
use crate::{
    app::{App, AppMode},
    formatting::{RenderConfig, StringCaseExt, render_inline},
    layout::{find_visual_cursor, strip_sigils},
    types::{LineType, PAGE_WIDTH, base_style},
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    f.render_widget(ratatui::widgets::Clear, area);
    let panel_style = Style::default().add_modifier(Modifier::REVERSED);


    let is_prompt = app.mode != AppMode::Normal;
    let has_status = app.status_msg.is_some();

    let show_top = !app.config.focus_mode;
    let show_bottom = !app.config.focus_mode || is_prompt || has_status;

    let title_height = if show_top { 1 } else { 0 };
    let footer_height = if show_bottom { 1 } else { 0 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(title_height),
            Constraint::Min(0),
            Constraint::Length(footer_height),
        ])
        .split(area);

    let (_title_area, mut text_area, footer_area) = (chunks[0], chunks[1], chunks[2]);

    app.sidebar_area = Rect::default();
    if app.mode == AppMode::SceneNavigator {
        let side_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(45), Constraint::Min(0)])
            .split(text_area);
        app.sidebar_area = side_chunks[0];
        text_area = side_chunks[1];

        let sidebar_block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(panel_style);
        f.render_widget(sidebar_block, app.sidebar_area);
    }

    app.settings_area = Rect::default();
    if app.mode == AppMode::SettingsPane || app.mode == AppMode::Shortcuts {
        let side_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(35)])
            .split(text_area);
        text_area = side_chunks[0];
        app.settings_area = side_chunks[1];

        let settings_block = Block::default()
            .borders(Borders::LEFT)
            .border_style(panel_style);
        f.render_widget(settings_block, app.settings_area);
    }

    let height = text_area.height as usize;
    app.visible_height = height;
    let page_w = PAGE_WIDTH.min(text_area.width);
    let global_pad = text_area.width.saturating_sub(page_w) / 2;

    let (vis_row, vis_x) = find_visual_cursor(&app.layout, app.cursor_y, app.cursor_x);

    let mut pad_top = 0;

    if app.config.strict_typewriter_mode {
        let absolute_center = area.height / 2;
        let center_offset = absolute_center.saturating_sub(text_area.y) as usize;
        if vis_row < center_offset {
            pad_top = center_offset - vis_row;
        }
        app.scroll = vis_row.saturating_sub(center_offset);
    } else if app.config.typewriter_mode {
        let absolute_center = area.height / 2;
        let center_offset = absolute_center.saturating_sub(text_area.y) as usize;
        app.scroll = vis_row.saturating_sub(center_offset);
    } else {
        if vis_row < app.scroll {
            app.scroll = vis_row;
        }
        if vis_row >= app.scroll + height {
            app.scroll = vis_row + 1 - height;
        }
    }

    let mut dark_gray_style = Style::default();
    if !app.config.no_color {
        dark_gray_style = dark_gray_style.add_modifier(Modifier::DIM);
    }

    let mut sug_style = Style::default();
    if !app.config.no_formatting {
        sug_style = sug_style.add_modifier(Modifier::DIM | Modifier::BOLD);
    }

    let mut page_num_style = Style::default();
    if !app.config.no_color {
        page_num_style = page_num_style.add_modifier(Modifier::DIM);
    }

    let mut visible: Vec<Line> = Vec::new();
    for _ in 0..pad_top {
        visible.push(Line::raw(""));
    }

    let mirror_scenes = app.config.mirror_scene_numbers == crate::config::MirrorOption::Always;

    visible.extend(
        app.layout
            .iter()
            .skip(app.scroll)
            .take(height.saturating_sub(pad_top))
            .map(|row| {
                let mut spans = Vec::new();
                let gap_size = 6u16;

                if let Some(ref snum) = row.scene_num {
                    let s_str = snum.to_string();
                    let s_len = UnicodeWidthStr::width(s_str.as_str()) as u16;

                    if global_pad >= s_len + gap_size {
                        let pad = global_pad - s_len - gap_size;
                        spans.push(Span::raw(" ".repeat(pad as usize)));
                        spans.push(Span::styled(s_str, dark_gray_style));
                        spans.push(Span::raw(" ".repeat(gap_size as usize)));
                    } else {
                        spans.push(Span::styled(s_str, dark_gray_style));
                        spans.push(Span::raw(" "));
                    }
                } else {
                    spans.push(Span::raw(" ".repeat(global_pad as usize)));
                }

                spans.push(Span::raw(" ".repeat(row.indent as usize)));

                let mut bst = base_style(row.line_type, &app.config);
                if let Some(c) = row.override_color
                    && !app.config.no_color
                {
                    bst.fg = Some(c);
                }

                let mut display = if row.is_active || !app.config.hide_markup {
                    row.raw_text.clone()
                } else {
                    strip_sigils(&row.raw_text, row.line_type).to_string()
                };

                let reveal_markup = !app.config.hide_markup || row.is_active;
                let skip_md = row.line_type == LineType::Boneyard;

                if matches!(
                    row.line_type,
                    LineType::SceneHeading | LineType::Transition | LineType::Shot
                ) {
                    display = display.to_uppercase_1to1();
                } else if matches!(
                    row.line_type,
                    LineType::Character | LineType::DualDialogueCharacter
                ) {
                    if let Some(idx) = display.find('(') {
                        let name = display[..idx].to_uppercase_1to1();
                        let ext = &display[idx..];
                        display = format!("{}{}", name, ext);
                    } else {
                        display = display.to_uppercase_1to1();
                    }
                }

                let empty_logical_line = String::new();
                let full_logical_line = app.lines.get(row.line_idx).unwrap_or(&empty_logical_line);

                let is_last_visual_row = row.char_end == full_logical_line.chars().count();
                let mut meta_key_end = 0;

                if (row.line_type == LineType::MetadataKey
                    || (row.line_type == LineType::MetadataTitle && row.is_active))
                    && let Some(idx) = full_logical_line.find(':')
                {
                    meta_key_end = full_logical_line[..=idx].chars().count() + 1;
                }

                let mut row_highlights = HashSet::new();
                if app.show_search_highlight
                    && let Some(re) = &app.compiled_search_regex
                {
                    for mat in re.find_iter(full_logical_line) {
                        let start_byte = mat.start();
                        let end_byte = mat.end();

                        let char_start = full_logical_line[..start_byte].chars().count();
                        let char_len = full_logical_line[start_byte..end_byte].chars().count();

                        for idx in char_start..(char_start + char_len) {
                            row_highlights.insert(idx);
                        }
                    }
                }

                spans.extend(render_inline(
                    &display,
                    bst,
                    &row.fmt,
                    RenderConfig {
                        reveal_markup,
                        skip_markdown: skip_md,
                        exclude_comments: false,
                        char_offset: row.char_start,
                        meta_key_end,
                        no_color: app.config.no_color,
                        no_formatting: app.config.no_formatting,
                    },
                    &row_highlights,
                ));

                if row.is_active
                    && row.line_idx == app.cursor_y
                    && is_last_visual_row
                    && let Some(sug) = &app.suggestion
                {
                    spans.push(Span::styled(sug.clone(), sug_style));
                }

                let right_text = if mirror_scenes {
                    row.scene_num.clone()
                } else {
                    row.page_num.map(|pnum| format!("{}.", pnum))
                };

                if let Some(r_str) = right_text {
                    let current_line_width: usize = spans
                        .iter()
                        .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
                        .sum();

                    let target_pos = global_pad as usize + page_w as usize + gap_size as usize;
                    if target_pos > current_line_width {
                        spans.push(Span::raw(" ".repeat(target_pos - current_line_width)));
                        spans.push(Span::styled(r_str, page_num_style));
                    }
                }

                Line::from(spans)
            }),
    );

    f.render_widget(Paragraph::new(visible), text_area);

    if app.mode == AppMode::SceneNavigator {
        let items: Vec<ListItem> = app
            .scenes
            .iter()
            .enumerate()
            .map(|(i, (_, heading, snum, synopses, color))| {
                let is_selected = i == app.selected_scene;
                let base_style = if is_selected {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };

                let heading_style = if let Some(c) = color {
                    base_style.fg(*c)
                } else {
                    base_style
                };

                let mut lines = Vec::new();
                let heading_text = if let Some(s) = snum {
                    format!("{} - {}", s, heading)
                } else {
                    heading.clone()
                };

                let prefix = if is_selected { " » " } else { "   " };
                let max_w = 40;
                let mut current_line = String::new();
                let mut first_line = true;

                for word in heading_text.split_whitespace() {
                    if !current_line.is_empty()
                        && prefix.len() + current_line.len() + word.len() + 1 > max_w
                    {
                        lines.push(Line::from(vec![
                            Span::styled(if first_line { prefix } else { "   " }, base_style),
                            Span::styled(current_line.clone(), heading_style),
                        ]));
                        current_line.clear();
                        first_line = false;
                    }
                    if !current_line.is_empty() {
                        current_line.push(' ');
                    }
                    current_line.push_str(word);
                }
                if !current_line.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled(if first_line { prefix } else { "   " }, base_style),
                        Span::styled(current_line, heading_style),
                    ]));
                } else if first_line {
                    // Always show at least one line for the heading
                    lines.push(Line::from(vec![
                        Span::styled(prefix, base_style),
                        Span::styled("", heading_style),
                    ]));
                }

                for syn in synopses {
                    let max_w = 40;
                    let mut current_line = String::new();
                    for word in syn.split_whitespace() {
                        if current_line.len() + word.len() + 1 > max_w {
                            lines.push(Line::from(vec![
                                Span::styled("     ", base_style),
                                Span::styled(current_line.clone(), base_style),
                            ]));
                            current_line.clear();
                        }
                        if !current_line.is_empty() {
                            current_line.push(' ');
                        }
                        current_line.push_str(word);
                    }
                    if !current_line.is_empty() {
                        lines.push(Line::from(vec![
                            Span::styled("     ", base_style),
                            Span::styled(current_line, base_style),
                        ]));
                    }
                }
                lines.push(Line::from(""));
                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items);
        f.render_stateful_widget(list, app.sidebar_area, &mut app.navigator_state);
    }

    if _title_area.height > 0 {
        let left_text = match app.mode {
            AppMode::SceneNavigator => "  SCENE NAVIGATOR".to_string(),
            AppMode::SettingsPane => "  SETTINGS".to_string(),
            AppMode::Shortcuts => "  SHORTCUTS".to_string(),
            _ => {
                if app.has_multiple_buffers {
                    format!("  [{}/{}]", app.current_buf_idx + 1, app.buffers.len())
                } else {
                    "  ".to_string()
                }
            }
        };

        let right_text = if app.dirty { "Modified  " } else { "  " };
        let center_text = app
            .file
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "New Script".to_string());

        let width = _title_area.width as usize;
        let left_len = left_text.chars().count();
        let right_len = right_text.chars().count();
        let center_len = center_text.chars().count();

        let center_start = (width.saturating_sub(center_len)) / 2;
        let pad1 = center_start.saturating_sub(left_len);
        let pad2 = width.saturating_sub(left_len + pad1 + center_len + right_len);

        let title_line = format!(
            "{}{}{}{}{}",
            left_text,
            " ".repeat(pad1),
            center_text,
            " ".repeat(pad2),
            right_text
        );
        f.render_widget(Paragraph::new(title_line).style(panel_style), _title_area);
    }

    if app.mode == AppMode::SettingsPane {
        let settings = vec![
            ("Typewriter Mode", &app.config.strict_typewriter_mode),
            ("Auto-Save", &app.config.auto_save),
            ("Scene Numbers", &app.config.show_scene_numbers),
            ("Page Numbers", &app.config.show_page_numbers),
            ("Hide Markup", &app.config.hide_markup),
            ("Autocomplete", &app.config.autocomplete),
            ("Auto-CONT'D", &app.config.auto_contd),
            ("Auto-Breaks", &app.config.auto_paragraph_breaks),
            ("Focus Mode", &app.config.focus_mode),
        ];

        let items: Vec<ListItem> = settings
            .into_iter()
            .enumerate()
            .map(|(i, (label, value))| {
                let style = if i == app.selected_setting {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                let status = if *value { "[X]" } else { "[ ]" };
                ListItem::new(format!(" {} {} [?]", status, label)).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(" Settings  [?] ")
                .border_style(panel_style),
        );
        f.render_widget(list, app.settings_area);
    }

    if app.mode == AppMode::Shortcuts {
        let shortcuts = vec![
            ("^S", "Save Script"),
            ("^X", "Exit Buffer"),
            ("^K", "Cut Line"),
            ("^U", "Paste Line"),
            ("^Z", "Undo Change"),
            ("^R", "Redo Change"),
            ("^H", "Scene Navigator"),
            ("^P", "Settings Pane"),
            ("^W", "Search Text"),
            ("^C", "Cursor Position"),
            ("F1", "Toggle Legend"),
        ];

        let items: Vec<ListItem> = shortcuts
            .iter()
            .map(|(key, desc)| {
                let lines = vec![Line::from(vec![
                    Span::styled(
                        format!(" {:<3}", key),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!("  {}", desc)),
                ])];
                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(" Shortcuts ")
                .border_style(panel_style),
        );
        f.render_widget(list, app.settings_area);
    }

    if footer_area.height > 0 {
        let left_text = "  SHORTCUTS [F1]".to_string();

        let cur_page = app.current_page_number();
        let total_pages = app.total_page_count();
        let word_count = app.total_word_count();

        let right_text = format!("PAGE {}/{} | {} WORDS  ", cur_page, total_pages, word_count);

        let mut center_text = String::new();
        if let Some(msg) = &app.status_msg {
            center_text = format!(" [ {} ] ", msg);
        } else {
            match app.mode {
                AppMode::Search => {
                    let prompt_base = if app.last_search.is_empty() {
                        "SEARCH: ".to_string()
                    } else {
                        format!("SEARCH [{}]: ", app.last_search)
                    };
                    center_text = format!("{}{}", prompt_base, app.search_query);
                }
                AppMode::PromptSave => center_text = "SAVE MODIFIED SCRIPT? (Y/N/C)".to_string(),
                AppMode::PromptFilename => {
                    center_text = format!("FILENAME: {}", app.filename_input)
                }
                _ => {}
            }
        }

        let width = footer_area.width as usize;
        let left_len = left_text.chars().count();
        let right_len = right_text.chars().count();
        let center_len = center_text.chars().count();

        let center_start = (width.saturating_sub(center_len)) / 2;
        let pad_left = center_start.saturating_sub(left_len);
        let pad_right = width.saturating_sub(left_len + pad_left + center_len + right_len);

        let footer_line = format!(
            "{}{}{}{}{}",
            left_text,
            " ".repeat(pad_left),
            center_text,
            " ".repeat(pad_right),
            right_text
        );
        f.render_widget(Paragraph::new(footer_line).style(panel_style), footer_area);
    }

    match app.mode {
        AppMode::Search if footer_area.height > 0 => {
            let prompt_base = if app.last_search.is_empty() {
                "Search: ".to_string()
            } else {
                format!("Search [{}]: ", app.last_search)
            };
            let query_w = UnicodeWidthStr::width(prompt_base.as_str())
                + UnicodeWidthStr::width(app.search_query.as_str());

            let center_start = (footer_area.width as usize).saturating_sub(query_w) / 2;
            let cur_screen_x = footer_area.x
                + center_start as u16
                + UnicodeWidthStr::width(prompt_base.as_str()) as u16;
            f.set_cursor_position((cur_screen_x, footer_area.y));
        }
        AppMode::PromptFilename if footer_area.height > 0 => {
            let prompt_base = "File Name to Write: ";
            let query_w = UnicodeWidthStr::width(prompt_base)
                + UnicodeWidthStr::width(app.filename_input.as_str());
            let center_start = (footer_area.width as usize).saturating_sub(query_w) / 2;
            let cur_screen_x =
                footer_area.x + center_start as u16 + UnicodeWidthStr::width(prompt_base) as u16;
            f.set_cursor_position((cur_screen_x, footer_area.y));
        }
        AppMode::Normal => {
            let cur_screen_y =
                text_area.y + pad_top as u16 + (vis_row.saturating_sub(app.scroll)) as u16;
            let cur_screen_x = text_area.x + global_pad + vis_x;
            if cur_screen_y < text_area.y + text_area.height {
                f.set_cursor_position((cur_screen_x, cur_screen_y));
            }
        }
        _ => {}
    }
}
