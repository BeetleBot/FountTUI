use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
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
    let in_command_mode = app.mode == AppMode::Command;
    let footer_height = if in_command_mode {
        if show_bottom { 2 } else { 1 }
    } else {
        if show_bottom { 1 } else { 0 }
    };

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
    if app.mode == AppMode::SettingsPane || app.mode == AppMode::Shortcuts || app.mode == AppMode::ExportPane {
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

                // Selection highlight (overrides search)
                let mut sel_highlights = HashSet::new();
                if let Some(((sel_sl, sel_sc), (sel_el, sel_ec))) = app.selection_range() {
                    let li = row.line_idx;
                    if li >= sel_sl && li <= sel_el {
                        let line_len = full_logical_line.chars().count();
                        let from = if li == sel_sl { sel_sc } else { 0 };
                        let to = if li == sel_el { sel_ec.min(line_len) } else { line_len };
                        for idx in from..to {
                            sel_highlights.insert(idx);
                        }
                    }
                }

                // Merge: sel_highlights takes priority — remove those from row_highlights
                // so render_inline gets the clean search set, and we'll override selected below.
                for idx in &sel_highlights {
                    row_highlights.remove(idx);
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
                    &sel_highlights,
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
            AppMode::ExportPane => "  EXPORT OPTIONS".to_string(),
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
            ("Autocomplete", &app.config.autocomplete),
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

    if app.mode == AppMode::ExportPane {
        let export_options = vec![
            format!(" Format: {}", app.config.export_format.to_uppercase()),
            format!(" Paper: {}", app.config.paper_size.to_uppercase()),
            format!(" Bold Headings: {}", if app.config.export_bold_scene_headings { "[X]" } else { "[ ]" }),
            " [ EXPORT ]".to_string(),
        ];

        let items: Vec<ListItem> = export_options
            .into_iter()
            .enumerate()
            .map(|(i, label)| {
                let style = if i == app.selected_export_option {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                ListItem::new(label).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(" Export  [?] ")
                .border_style(panel_style),
        );
        f.render_widget(list, app.settings_area);
    }

    if app.mode == AppMode::Shortcuts {
        let items = vec![
            ListItem::new("  ── COMMANDS ──").style(Style::default().add_modifier(Modifier::DIM)),
            ListItem::new("  /w            Save Buffer"),
            ListItem::new("  /q            Quit Application"),
            ListItem::new("  /renum        Renumber Scenes"),
            ListItem::new("  /set [opt]    Change Settings"),
            ListItem::new("  /search [q]   Global Search"),
            ListItem::new("  /s[num]       Jump to Scene"),
            ListItem::new(""),
            ListItem::new("  ── EDITING ──").style(Style::default().add_modifier(Modifier::DIM)),
            ListItem::new("  ^A            Select All"),
            ListItem::new("  ^C            Copy Selection"),
            ListItem::new("  ^X            Cut Selection"),
            ListItem::new("  ^V            Paste Clipboard"),
            ListItem::new("  Shift+Arrows  Manual Selection"),
            ListItem::new(""),
            ListItem::new("  ── PANES ──").style(Style::default().add_modifier(Modifier::DIM)),
            ListItem::new("  ^H            Scene Navigator"),
            ListItem::new("  ^P            Settings Pane"),
            ListItem::new("  ^E            Export Options"),
            ListItem::new("  F1            Command Legend"),
        ];

        let list = List::new(items).block(
            Block::default()
                .title(" Command Legend ")
                .border_style(panel_style),
        );
        f.render_widget(list, app.settings_area);
    }

    // ── Footer rendering ──────────────────────────────────────────────────
    if footer_area.height > 0 {
        // When in command mode, split the footer into status bar (top) + command bar (bottom)
        let (status_area, cmd_area) = if in_command_mode && footer_area.height >= 2 {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Length(1)])
                .split(footer_area);
            (rows[0], Some(rows[1]))
        } else {
            (footer_area, None)
        };

        // ── Status bar ────────────────────────────────────────────────────
        let (mode_str, mode_bg) = match app.mode {
            AppMode::Normal => (" NORMAL ".to_string(), Color::LightBlue),
            AppMode::Command => (" COMMAND ".to_string(), Color::Yellow),
            AppMode::SceneNavigator => (" NAVIGATOR ".to_string(), Color::LightCyan),
            AppMode::SettingsPane => (" SETTINGS ".to_string(), Color::LightCyan),
            AppMode::ExportPane => (" EXPORT ".to_string(), Color::LightCyan),
            AppMode::Search => (" SEARCH ".to_string(), Color::LightMagenta),
            _ => (" PROMPT ".to_string(), Color::LightRed),
        };

        let mode_fg = Color::Black;
        let mid_bg = Color::Rgb(40, 44, 52); // Darker grey
        let mid_fg = Color::White;

        let mut spans = Vec::new();

        spans.push(Span::styled(mode_str, Style::default().bg(mode_bg).fg(mode_fg).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled("", Style::default().fg(mode_bg).bg(mid_bg)));

        let center_text = if let Some(msg) = &app.status_msg {
            format!(" {} ", msg)
        } else {
            match app.mode {
                AppMode::Search => {
                    let prompt_base = if app.last_search.is_empty() {
                        " SEARCH: ".to_string()
                    } else {
                        format!(" SEARCH [{}]: ", app.last_search)
                    };
                    format!(" {}{}", prompt_base, app.search_query)
                }
                AppMode::PromptSave => " SAVE MODIFIED SCRIPT? (Y/N/C) ".to_string(),
                AppMode::PromptFilename => format!(" FILENAME: {} ", app.filename_input),
                AppMode::PromptExportFilename => format!(" EXPORT {}: {} ", app.config.export_format.to_uppercase(), app.filename_input),
                _ => {
                    let fname = app.file.as_ref()
                        .and_then(|p| p.file_name())
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "[No Name]".to_string());
                    let dirty_str = if app.dirty { " [+] " } else { " " };
                    format!(" {}{}", fname, dirty_str)
                }
            }
        };
        
        spans.push(Span::styled(center_text.clone(), Style::default().bg(mid_bg).fg(mid_fg)));

        let right_bg1 = Color::Rgb(60, 65, 75); // Lighter gray than mid_bg
        let right_fg1 = Color::White;
        let total_lines = app.layout.len();
        let scroll_str = if total_lines <= app.visible_height {
            " All ".to_string()
        } else if app.scroll == 0 {
            " Top ".to_string()
        } else if app.scroll + app.visible_height >= total_lines {
            " Bot ".to_string()
        } else {
            let pct = (app.scroll as f32 / total_lines as f32 * 100.0) as u8;
            format!(" {}% ", pct)
        };
        
        let right_bg2 = mode_bg;
        let right_fg2 = Color::Black;
        let pos_str = format!(" {}:{} ", app.cursor_y + 1, app.cursor_x + 1);

        let mut right_spans = Vec::new();
        right_spans.push(Span::styled("", Style::default().fg(right_bg1).bg(mid_bg)));
        
        let right_text1 = format!(" Δ {} ", scroll_str.trim());
        right_spans.push(Span::styled(right_text1, Style::default().bg(right_bg1).fg(right_fg1)));
        right_spans.push(Span::styled("", Style::default().fg(right_bg2).bg(right_bg1)));
        right_spans.push(Span::styled(pos_str, Style::default().bg(right_bg2).fg(right_fg2).add_modifier(Modifier::BOLD)));

        let left_width: usize = spans.iter().map(|s| UnicodeWidthStr::width(s.content.as_ref())).sum();
        let right_width: usize = right_spans.iter().map(|s| UnicodeWidthStr::width(s.content.as_ref())).sum();
        let total_width = status_area.width as usize;

        if total_width > left_width + right_width {
            let pad_len = total_width - left_width - right_width;
            spans.push(Span::styled(" ".repeat(pad_len), Style::default().bg(mid_bg)));
        }
        
        spans.extend(right_spans);

        let status_line = Line::from(spans);
        f.render_widget(Paragraph::new(status_line), status_area);

        // ── Command bar (only in Command mode) ────────────────────────────
        if let Some(cmd_rect) = cmd_area {
            let cmd_style = if app.command_error {
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            };

            // Build styled spans: bold '/' prefix + command text + blinking cursor hint
            let prefix = Span::styled(" / ", cmd_style.add_modifier(Modifier::BOLD));
            let cmd_text = Span::styled(
                format!("{} ", app.command_input),
                cmd_style,
            );
            let hint_text = if app.command_input.is_empty() && !app.command_error {
                Span::styled(" type a command, Tab to complete, Esc to cancel ", 
                    Style::default().fg(Color::DarkGray).bg(Color::Yellow))
            } else {
                Span::raw("")
            };

            let cmd_line = ratatui::text::Line::from(vec![prefix, cmd_text, hint_text]);
            f.render_widget(Paragraph::new(cmd_line), cmd_rect);

            // Place cursor in command bar
            let cur_x = cmd_rect.x + 3 + UnicodeWidthStr::width(app.command_input.as_str()) as u16;
            f.set_cursor_position((cur_x, cmd_rect.y));
        }
    }

    // ── Cursor for non-command modes ──────────────────────────────────────
    if !in_command_mode {
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
                let cur_screen_x = footer_area.x
                    + center_start as u16
                    + UnicodeWidthStr::width(prompt_base) as u16;
                f.set_cursor_position((cur_screen_x, footer_area.y));
            }
            AppMode::PromptExportFilename if footer_area.height > 0 => {
                let prompt_base = format!("EXPORT {}: ", app.config.export_format.to_uppercase());
                let query_w = UnicodeWidthStr::width(prompt_base.as_str())
                    + UnicodeWidthStr::width(app.filename_input.as_str());
                let center_start = (footer_area.width as usize).saturating_sub(query_w) / 2;
                let cur_screen_x = footer_area.x
                    + center_start as u16
                    + UnicodeWidthStr::width(prompt_base.as_str()) as u16;
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
}
