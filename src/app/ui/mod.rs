pub mod panes;
use self::panes::{draw_snapshots, draw_sprint_stats, draw_file_picker, home::draw_home, xray::draw_xray, index_cards::draw_index_cards};

use crate::{
    app::{App, AppMode, EnsembleItem, GoalType, shortcuts},
    formatting::{RenderConfig, StringCaseExt, render_inline},
    layout::{find_visual_cursor, strip_sigils},
    types::{LineType, PAGE_WIDTH, base_style},
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph},
};
use std::collections::HashSet;
use unicode_width::UnicodeWidthStr;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = &app.theme;


    let mut base_ui_style = Style::default();
    if let Some(bg) = &theme.ui.background {
        base_ui_style = base_ui_style.bg(Color::from(bg.clone()));
    }
    if let Some(fg) = &theme.ui.foreground {
        base_ui_style = base_ui_style.fg(Color::from(fg.clone()));
    }
    f.render_widget(Block::default().style(base_ui_style), area);

    let (mode_str, mode_bg) = match app.mode {
        AppMode::Normal => (" Normal ", Color::from(theme.ui.normal_mode_bg.clone())),
        AppMode::Command => (" Command ", Color::from(theme.ui.command_mode_bg.clone())),
        AppMode::SceneNavigator => (
            " Navigator ",
            Color::from(theme.ui.navigator_mode_bg.clone()),
        ),
        AppMode::SettingsPane => (" Settings ", Color::from(theme.ui.settings_mode_bg.clone())),
        AppMode::ExportPane => (" Export ", Color::from(theme.ui.normal_mode_bg.clone())),
        AppMode::Shortcuts => (" Legend ", Color::from(theme.ui.normal_mode_bg.clone())),
        AppMode::Search => (" Search ", Color::from(theme.ui.search_mode_bg.clone())),
        AppMode::Home => (" Home ", Color::from(theme.ui.normal_mode_bg.clone())),
        AppMode::FilePicker => (" File ", Color::from(theme.ui.normal_mode_bg.clone())),
        AppMode::Snapshots => (
            " Snapshots ",
            Color::from(theme.ui.navigator_mode_bg.clone()),
        ),
        AppMode::SprintStat => (" Sprints ", Color::from(theme.ui.normal_mode_bg.clone())),
        AppMode::XRay => (" X-Ray ", Color::from(theme.ui.navigator_mode_bg.clone())),
        AppMode::IndexCards => (" Index Cards ", Color::from(theme.ui.navigator_mode_bg.clone())),
        AppMode::ReplaceOne | AppMode::ReplaceAll => (" Replace ", Color::from(theme.ui.command_mode_bg.clone())),
        _ => (" Prompt ", Color::from(theme.ui.command_mode_bg.clone())),
    };

    let is_prompt = app.mode != AppMode::Normal;
    let has_status = app.status_msg.is_some();

    let show_bottom = !app.config.focus_mode || is_prompt || has_status;

    let _in_command_mode = app.mode == AppMode::Command;
    let footer_height = if show_bottom { 1 } else { 0 };
    let show_header = !app.config.focus_mode || is_prompt || has_status;
    let header_height: u16 = if show_header { 1 } else { 0 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Min(0),
            Constraint::Length(footer_height),
        ])
        .split(area);

    let (header_area, mut text_area, footer_area) = (chunks[0], chunks[1], chunks[2]);

    // ── Header rendering (Zen Style) ──────────────────────────────────────
    {
        let dim_color = Color::from(theme.ui.dim.clone());
        let sep = " | ";
        let sep_style = Style::default().fg(dim_color);

        let mut left_spans = Vec::new();

        // Opening bracket + app name
        left_spans.push(Span::styled("[ ", sep_style));
        left_spans.push(Span::styled(
            format!("Fount v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(mode_bg).add_modifier(Modifier::BOLD),
        ));

        // Mode label
        left_spans.push(Span::styled(sep, sep_style));
        let active_context_mode = if app.mode == AppMode::Command || app.mode == AppMode::Search {
            app.previous_mode
        } else {
            app.mode
        };

        let mode_label = match active_context_mode {
            AppMode::IndexCards => " INDEX CARDS ",
            _ => " EDITOR ",
        };
        left_spans.push(Span::styled(
            mode_label,
            Style::default().fg(mode_bg).add_modifier(Modifier::BOLD),
        ));
        left_spans.push(Span::styled(sep, sep_style));

        // Buffer tabs (if multiple buffers)
        if app.buffers.len() > 1 {
            left_spans.push(Span::styled(sep, sep_style));

            for i in 0..app.buffers.len() {
                let (file, dirty) = if i == app.current_buf_idx {
                    (&app.file, app.dirty)
                } else {
                    (&app.buffers[i].file, app.buffers[i].dirty)
                };

                let name = file
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "New Script".to_string());

                let dirty_mark = if dirty { "*" } else { "" };
                let label = format!("{}{}", name, dirty_mark);

                if i == app.current_buf_idx {
                    left_spans.push(Span::styled(
                        label,
                        Style::default()
                            .fg(Color::from(theme.ui.selection_fg.clone()))
                            .bg(Color::from(theme.ui.selection_bg.clone()))
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    left_spans.push(Span::styled(label, Style::default().fg(dim_color)));
                }

                if i + 1 < app.buffers.len() {
                    left_spans.push(Span::styled(sep, sep_style));
                }
            }
        }

        // Right side: theme name + closing bracket
        let mut right_spans = Vec::new();
        right_spans.push(Span::styled(sep, sep_style));
        right_spans.push(Span::styled(
            app.config.theme.clone(),
            Style::default().fg(dim_color),
        ));
        right_spans.push(Span::styled(" ]", sep_style));

        let left_width: usize = left_spans
            .iter()
            .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
            .sum();
        let right_width: usize = right_spans
            .iter()
            .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
            .sum();
        let total_width = header_area.width as usize;

        if total_width > left_width + right_width {
            let pad_len = total_width - left_width - right_width;
            left_spans.push(Span::raw(" ".repeat(pad_len)));
        }

        left_spans.extend(right_spans);
        f.render_widget(Paragraph::new(Line::from(left_spans)), header_area);
    }

    app.sidebar_area = Rect::default();
    if app.mode == AppMode::SceneNavigator || app.mode == AppMode::CharacterNavigator {
        let side_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(41),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(text_area);
        app.sidebar_area = side_chunks[0];
        let shadow_area = side_chunks[1];
        text_area = side_chunks[2];

        // Draw clean separator
        let dim_sep_color = Color::from(theme.ui.dim.clone());
        let sep_col = "│".repeat(shadow_area.height as usize);
        let sep_lines: Vec<Line> = sep_col.chars().map(|_| Line::from(Span::styled("│", Style::default().fg(dim_sep_color)))).collect();
        f.render_widget(Paragraph::new(sep_lines), shadow_area);
    }

    app.settings_area = Rect::default();
    if app.mode == AppMode::SettingsPane
        || app.mode == AppMode::ExportPane
    {
        let side_width: u16 = 34;
        let side_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(side_width),
            ])
            .split(text_area);
        text_area = side_chunks[0];
        let shadow_area = side_chunks[1];
        app.settings_area = side_chunks[2];

        // Draw clean separator
        let dim_sep_color = Color::from(theme.ui.dim.clone());
        let sep_lines: Vec<Line> = (0..shadow_area.height).map(|_| Line::from(Span::styled("│", Style::default().fg(dim_sep_color)))).collect();
        f.render_widget(Paragraph::new(sep_lines), shadow_area);
    }

    let height = text_area.height as usize;
    app.visible_height = height;
    let page_w = PAGE_WIDTH.min(text_area.width);
    let global_pad = text_area.width.saturating_sub(page_w) / 2;

    let mut pad_top = 0;

    if app.mode != AppMode::Home {
        let (vis_row, _vis_x) = find_visual_cursor(&app.layout, app.cursor_y, app.cursor_x);

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
    }

    let mut dark_gray_style = Style::default();
    if !app.config.no_color {
        dark_gray_style = dark_gray_style.fg(Color::from(theme.ui.dim.clone()));
    }

    let mut sug_style = Style::default();
    if !app.config.no_formatting {
        sug_style = sug_style.add_modifier(Modifier::DIM | Modifier::BOLD);
    }

    let mut page_num_style = Style::default();
    if !app.config.no_color {
        page_num_style = page_num_style.fg(Color::from(theme.ui.dim.clone()));
    }

    let current_view_mode = if app.mode == AppMode::Command || app.mode == AppMode::Search {
        app.previous_mode
    } else {
        app.mode
    };

    if current_view_mode != AppMode::IndexCards {
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

                    let mut bst = base_style(row.line_type, &app.config, &app.theme);
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
                            let to = if li == sel_el {
                                sel_ec.min(line_len)
                            } else {
                                line_len
                            };
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
    }

    if app.mode == AppMode::SceneNavigator {
        let border_color = theme
            .sidebar
            .border
            .clone()
            .map(Color::from)
            .unwrap_or(Color::DarkGray);
        let selected_bg = theme
            .sidebar
            .item_selected_bg
            .clone()
            .map(Color::from)
            .unwrap_or(mode_bg);
        let selected_fg = theme
            .sidebar
            .item_selected_fg
            .clone()
            .map(Color::from)
            .unwrap_or(Color::Black);
        let dim_color = theme
            .sidebar
            .item_dimmed
            .clone()
            .map(Color::from)
            .unwrap_or(Color::DarkGray);
        let header_color = theme
            .sidebar
            .section_header
            .clone()
            .map(Color::from)
            .unwrap_or(mode_bg);

        let items: Vec<ListItem> = app
            .scenes
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = i == app.selected_scene;
                let mut lines = Vec::new();

                let line_style = Style::default()
                    .fg(border_color)
                    .add_modifier(Modifier::DIM);

                if item.is_section {
                    // Section Header (ACT I, etc.)
                    let style = if is_selected {
                        Style::default()
                            .fg(selected_fg)
                            .bg(selected_bg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(header_color)
                            .add_modifier(Modifier::BOLD)
                    };

                    let prefix = if is_selected { " › " } else { "   " };
                    lines.push(Line::from(vec![
                        Span::styled(prefix, style),
                        Span::styled(item.label.to_uppercase(), style),
                    ]));

                    // Add a connecting line start below the section if the next item is a scene
                    if i + 1 < app.scenes.len() && !app.scenes[i + 1].is_section {
                        lines.push(Line::from(vec![
                            Span::styled("   ", Style::default()),
                            Span::styled("│", line_style),
                        ]));
                    } else {
                        lines.push(Line::from(""));
                    }
                } else {
                    // Scene Item
                    let mut base_style = if is_selected {
                        Style::default()
                            .fg(selected_fg)
                            .bg(selected_bg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    if let Some(c) = &theme.ui.foreground
                        && !is_selected
                    {
                        base_style = base_style.fg(Color::from(c.clone()));
                    }

                    if let Some(c) = item.color
                        && !is_selected
                    {
                        base_style = base_style.fg(c);
                    }

                    let dim_style = if is_selected {
                        base_style
                    } else {
                        Style::default().fg(dim_color).add_modifier(Modifier::DIM)
                    };

                    let prefix = if is_selected { " › " } else { "   " };

                    // Determine if this is the last scene in the section
                    let is_last_in_section =
                        i + 1 == app.scenes.len() || app.scenes[i + 1].is_section;
                    let connector = if is_last_in_section {
                        "└─ "
                    } else {
                        "├─ "
                    };

                    let s_tag = if let Some(ref s) = item.scene_num {
                        format!("{}. ", s)
                    } else {
                        String::new()
                    };

                    // Line 1: Scene Heading
                    lines.push(Line::from(vec![
                        Span::styled(prefix, base_style),
                        Span::styled(connector, line_style),
                        Span::styled(s_tag, base_style.add_modifier(Modifier::BOLD)),
                        Span::styled(item.label.clone(), base_style),
                    ]));

                    // Line 2+: Wrapped Synopses or placeholder
                    let syn_line_char = if is_last_in_section { "   " } else { "│  " };
                    if !item.synopses.is_empty() {
                        // Wrapping logic for each synopsis
                        for syn in &item.synopses {
                            let mut current_line = String::new();
                            let max_syn_w = 34; // Fits within the new 42-char wide navigator width

                            for word in syn.split_whitespace() {
                                if current_line.len() + word.len() + 1 > max_syn_w {
                                    lines.push(Line::from(vec![
                                        Span::styled("   ", Style::default()),
                                        Span::styled(syn_line_char, line_style),
                                        Span::styled(
                                            current_line.clone(),
                                            dim_style.add_modifier(Modifier::ITALIC),
                                        ),
                                    ]));
                                    current_line = word.to_string();
                                } else {
                                    if !current_line.is_empty() {
                                        current_line.push(' ');
                                    }
                                    current_line.push_str(word);
                                }
                            }
                            if !current_line.is_empty() {
                                lines.push(Line::from(vec![
                                    Span::styled("   ", Style::default()),
                                    Span::styled(syn_line_char, line_style),
                                    Span::styled(
                                        current_line,
                                        dim_style.add_modifier(Modifier::ITALIC),
                                    ),
                                ]));
                            }
                        }
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled("   ", Style::default()),
                            Span::styled(syn_line_char, line_style),
                            Span::styled("no synopsis", dim_style.add_modifier(Modifier::ITALIC)),
                        ]));
                    }

                    // Spacer/Separator
                    if !is_last_in_section {
                        lines.push(Line::from(vec![
                            Span::styled("   ", Style::default()),
                            Span::styled(syn_line_char, line_style),
                            Span::styled(
                                "─".repeat(30),
                                Style::default()
                                    .fg(Color::DarkGray)
                                    .add_modifier(Modifier::DIM),
                            ),
                        ]));
                    } else {
                        lines.push(Line::from(""));
                    }
                }

                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items).highlight_style(Style::default());
        f.render_stateful_widget(
            list,
            app.sidebar_area.inner(ratatui::layout::Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut app.navigator_state,
        );
    }

    if app.mode == AppMode::CharacterNavigator {
        let border_color = theme
            .sidebar
            .border
            .clone()
            .map(Color::from)
            .unwrap_or(Color::DarkGray);
        let selected_bg = theme
            .sidebar
            .item_selected_bg
            .clone()
            .map(Color::from)
            .unwrap_or(mode_bg);
        let selected_fg = theme
            .sidebar
            .item_selected_fg
            .clone()
            .map(Color::from)
            .unwrap_or(Color::Black);
        let dim_color = theme
            .sidebar
            .item_dimmed
            .clone()
            .map(Color::from)
            .unwrap_or(Color::DarkGray);
        let header_color = theme
            .sidebar
            .section_header
            .clone()
            .map(Color::from)
            .unwrap_or(mode_bg);

        let items: Vec<ListItem> = app
            .ensemble_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = i == app.selected_ensemble_idx;
                let mut lines = Vec::new();

                let line_style = Style::default()
                    .fg(border_color)
                    .add_modifier(Modifier::DIM);
                let base_style = if is_selected {
                    Style::default()
                        .fg(selected_fg)
                        .bg(selected_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let dim_style = if is_selected {
                    base_style
                } else {
                    Style::default().fg(dim_color).add_modifier(Modifier::DIM)
                };

                let prefix = if is_selected { " › " } else { "   " };

                match item {
                    EnsembleItem::CharacterHeader(char_idx) => {
                        let char_item = &app.character_stats[*char_idx];
                        lines.push(Line::from(vec![
                            Span::styled(prefix, base_style),
                            Span::styled(
                                char_item.name.clone(),
                                Style::default()
                                    .fg(header_color)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]));
                    }
                    EnsembleItem::Stat(text, hint, is_last) => {
                        let connector = if text.is_empty() {
                            "│"
                        } else if *is_last {
                            "└─ "
                        } else {
                            "├─ "
                        };

                        let mut spans = vec![
                            Span::styled("   ", Style::default()),
                            Span::styled(connector, line_style),
                            Span::styled(text.clone(), dim_style.add_modifier(Modifier::ITALIC)),
                        ];

                        if let Some(h) = hint {
                            spans.push(Span::styled(
                                format!(" {}", h),
                                line_style.add_modifier(Modifier::ITALIC),
                            ));
                        }

                        lines.push(Line::from(spans));
                    }
                    EnsembleItem::SceneLink(name, _, _) => {
                        lines.push(Line::from(vec![
                            Span::styled(prefix, base_style),
                            Span::styled("│  └─ ", line_style),
                            Span::styled(name.clone(), dim_style.add_modifier(Modifier::ITALIC)),
                        ]));
                    }
                    EnsembleItem::Separator => {
                        lines.push(Line::from(""));
                    }
                }

                ListItem::new(lines)
            })
            .collect();

        let title = format!(" [ Ensemble ({}) ]", app.character_stats.len());
        let list = List::new(items)
            .block(Block::default().title(Span::styled(title, Style::default().fg(Color::from(theme.ui.dim.clone())))))
            .highlight_style(Style::default());

        f.render_stateful_widget(
            list,
            app.sidebar_area.inner(ratatui::layout::Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut app.ensemble_state,
        );
    }

    if app.mode == AppMode::SettingsPane {
        let settings = vec![
            ("Typewriter Mode", &app.config.strict_typewriter_mode),
            ("Auto-Save", &app.config.auto_save),
            ("Autocomplete", &app.config.autocomplete),
            ("Auto-Breaks", &app.config.auto_paragraph_breaks),
            ("Focus Mode", &app.config.focus_mode),
            ("Theme", &false), // Not a toggle
        ];

        let theme_name = &app.config.theme;
        let items: Vec<ListItem> = settings
            .into_iter()
            .enumerate()
            .map(|(i, (label, value))| {
                let is_selected = i == app.selected_setting;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(mode_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let (icon, icon_style) = if label == "Theme" {
                    if app.config.use_nerd_fonts {
                        (
                            "󰔎 ",
                            Style::default().fg(Color::from(theme.ui.normal_mode_bg.clone())),
                        )
                    } else {
                        (
                            "• ",
                            Style::default().fg(Color::from(theme.ui.normal_mode_bg.clone())),
                        )
                    }
                } else if *value {
                    if app.config.use_nerd_fonts {
                        ("󰄬 ", Style::default().fg(Color::Green))
                    } else {
                        ("✓ ", Style::default().fg(Color::Green))
                    }
                } else {
                    if app.config.use_nerd_fonts {
                        ("󰄱 ", Style::default().fg(Color::DarkGray))
                    } else {
                        ("[ ] ", Style::default().fg(Color::DarkGray))
                    }
                };

                let line = if label == "Theme" {
                    Line::from(vec![
                        Span::styled(if is_selected { " › " } else { "   " }, style),
                        Span::styled(icon, if is_selected { style } else { icon_style }),
                        Span::styled(format!("{}: {}", label, theme_name), style),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled(if is_selected { " › " } else { "   " }, style),
                        Span::styled(icon, if is_selected { style } else { icon_style }),
                        Span::styled(label, style),
                    ])
                };

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items);
        f.render_widget(
            list,
            app.settings_area.inner(ratatui::layout::Margin {
                horizontal: 0,
                vertical: 1,
            }),
        );
    }

    if app.mode == AppMode::ExportPane {
        let format_label = match app.config.export_format.as_str() {
            "pdf" => "PDF",
            "fountain" => "Fountain",
            "fdx" => "FDX (Coming Soon)",
            _ => "PDF",
        };

        let report_label = match app.config.report_format.as_str() {
            "csv_scene" => "Scene List (CSV)",
            "csv_char" => "Character Report (CSV)",
            "csv_location" => "Location Report (CSV)",
            "csv_notes" => "Notes & Markers (CSV)",
            "csv_breakdown" => "Script Breakdown (CSV)",
            "txt_dialogue" => "Dialogue Only (TXT)",
            _ => "Scene List (CSV)",
        };

        let header_style = Style::default().fg(mode_bg).add_modifier(Modifier::BOLD);

        let mut visual_items = Vec::new();

        let render_item = |idx: usize, label: &str, app: &App| -> ListItem {
            let is_selected = idx == app.selected_export_option;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(mode_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(if is_selected { " › " } else { "   " }, style),
                Span::styled(label.to_string(), style),
            ]))
        };

        visual_items.push(ListItem::new(Line::from(vec![Span::styled(
            "  [ Screenplay Export ]",
            header_style,
        )])));
        visual_items.push(render_item(0, &format!(" Format: {}", format_label), app));

        visual_items.push(render_item(
            1,
            &format!(" Paper: {}", app.config.paper_size.to_uppercase()),
            app,
        ));
        visual_items.push(render_item(
            2,
            &format!(
                " Bold Headings: {}",
                if app.config.export_bold_scene_headings {
                    "[X]"
                } else {
                    "[ ]"
                }
            ),
            app,
        ));
        visual_items.push(render_item(
            3,
            &format!(
                " Scene Numbers: Both Sides: {}",
                if app.config.mirror_scene_numbers != crate::config::MirrorOption::Off {
                    "[X]"
                } else {
                    "[ ]"
                }
            ),
            app,
        ));

        visual_items.push(ListItem::new(Line::from(vec![Span::raw("")])));
        visual_items.push(render_item(4, " [ EXPORT SCREENPLAY ]", app));

        visual_items.push(ListItem::new(Line::from(Span::raw(""))));
        visual_items.push(ListItem::new(Line::from(Span::styled(
            "  [ Production Reports ]",
            header_style,
        ))));
        visual_items.push(render_item(5, &format!(" Type: {}", report_label), app));

        visual_items.push(ListItem::new(Line::from(Span::raw(""))));
        visual_items.push(render_item(6, " [ EXPORT REPORT ]", app));

        let list = List::new(visual_items);
        f.render_widget(
            list,
            app.settings_area.inner(ratatui::layout::Margin {
                horizontal: 0,
                vertical: 1,
            }),
        );
    }

    if app.mode == AppMode::Shortcuts {
        // ── Cheat Sheet: 3-column grid ──────────────────────────────────
        let modal_area = panes::centered_rect(92, 90, area);
        f.render_widget(ratatui::widgets::Clear, modal_area);

        let dim_color = Color::from(theme.ui.dim.clone());
        let key_style = Style::default().fg(mode_bg).add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(Color::Gray);
        let hdr_style = Style::default().fg(mode_bg).add_modifier(Modifier::BOLD);
        let sep_style = Style::default().fg(dim_color);

        // Helper: build lines for one category from registry
        let build_section = |title: &str, shortcuts: &[shortcuts::Shortcut]| -> Vec<Line<'static>> {
            let mut lines = Vec::new();
            lines.push(Line::from(Span::styled(format!(" [ {} ]", title), hdr_style)));
            for shortcut in shortcuts {
                let k = shortcut.key.trim();
                lines.push(Line::from(vec![
                    Span::styled(format!("  {:<14}", k), key_style),
                    Span::styled(shortcut.desc, desc_style),
                ]));
            }
            lines.push(Line::from(""));
            lines
        };

        let all_shortcuts = shortcuts::get_all_shortcuts();
        
        // Group by category while preserving order of first appearance
        let mut categories: Vec<&str> = Vec::new();
        for s in &all_shortcuts {
            if !categories.contains(&s.category) {
                categories.push(s.category);
            }
        }

        // Define column assignments (can be tweaked as needed)
        // Col1: 1, 2, 3 | Col2: 4, 5 | Col3: 6, 7, 8
        let mut col1: Vec<Line> = Vec::new();
        let mut col2: Vec<Line> = Vec::new();
        let mut col3: Vec<Line> = Vec::new();

        for (i, cat) in categories.iter().enumerate() {
            let cat_shortcuts: Vec<shortcuts::Shortcut> = all_shortcuts.iter()
                .filter(|s| s.category == *cat)
                .cloned()
                .collect();
            
            let section_lines = build_section(cat, &cat_shortcuts);
            if i < 3 {
                col1.extend(section_lines);
            } else if i < 5 {
                col2.extend(section_lines);
            } else {
                col3.extend(section_lines);
            }
        }

        // Render the block border
        let block = Block::default()
            .title(Span::styled(
                " [ Cheat Sheet ] ",
                Style::default().fg(mode_bg).add_modifier(Modifier::BOLD),
            ))
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(dim_color));

        let inner = block.inner(modal_area);
        f.render_widget(block, modal_area);

        // Split inner into 3 columns with separators
        let col_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Ratio(1, 3),
                Constraint::Length(1),
                Constraint::Ratio(1, 3),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(inner);

        // Render separator columns
        let sep_lines: Vec<Line> = (0..col_chunks[1].height)
            .map(|_| Line::from(Span::styled("\u{2502}", sep_style)))
            .collect();
        f.render_widget(Paragraph::new(sep_lines.clone()), col_chunks[1]);
        f.render_widget(Paragraph::new(sep_lines), col_chunks[3]);

        // Render the three columns
        f.render_widget(Paragraph::new(col1), col_chunks[0]);
        f.render_widget(Paragraph::new(col2), col_chunks[2]);
        f.render_widget(Paragraph::new(col3), col_chunks[4]);
    }



    // ── Footer rendering (Zen Style) ────────────────────────────────────────
    if footer_area.height > 0 {
        let dim_color = Color::from(theme.ui.dim.clone());
        let sep = " | ";
        let sep_style = Style::default().fg(dim_color);

        let mut spans = Vec::new();

        // Opening bracket
        spans.push(Span::styled("[ ", sep_style));

        // Mode label (title-case, calmer than SCREAMING)
        spans.push(Span::styled(
            mode_str.trim(),
            Style::default().fg(mode_bg).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(sep, sep_style));

        // Filename + dirty/lock indicators
        let fname = app
            .file
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "New Script".to_string());
        let dirty_str = if app.dirty { "*" } else { "" };
        let lock_str = if app.config.production_lock {
            " 🔒"
        } else {
            ""
        };
        spans.push(Span::styled(
            format!("{}{}{}", fname, dirty_str, lock_str),
            Style::default().fg(mode_bg),
        ));
        
        // Saved indicator
        if let Some(time) = app.save_indicator_timer {
            let elapsed = time.elapsed().as_secs_f32();
            if elapsed < 2.0 {
                spans.push(Span::styled("  ✓ Saved", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
            }
        }

        spans.push(Span::styled(sep, sep_style));

        // Center content: command, search, status, or hint
        if app.mode == AppMode::Command {
            let cmd_style = if app.command_error {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::BOLD)
            };
            spans.push(Span::styled("/", cmd_style));
            spans.push(Span::styled(&app.command_input, cmd_style));

            if !app.command_input.is_empty() && !app.command_error {
                let commands = app.get_command_completions();
                if let Some(first_match) = commands.iter().find(|&c| c.starts_with(&app.command_input) && c != &app.command_input) {
                    let remainder = &first_match[app.command_input.len()..];
                    spans.push(Span::styled(remainder.to_string(), Style::default().fg(dim_color)));
                }
            }

            if app.command_input.is_empty() && !app.command_error {
                spans.push(Span::styled(
                    " type a command...",
                    Style::default().fg(dim_color),
                ));
            }
        } else if matches!(
            app.mode,
            AppMode::Search | AppMode::PromptSave | AppMode::PromptFilename | AppMode::ReplaceOne | AppMode::ReplaceAll
        ) {
            match app.mode {
                AppMode::Search => {
                    let prompt_base = if app.last_search.is_empty() {
                        "Search: ".to_string()
                    } else {
                        format!("Search [{}]: ", app.last_search)
                    };
                    
                    let mut count_msg = String::new();
                    if !app.search_matches.is_empty() {
                        let cur = app.current_match_idx.map(|idx| idx + 1).unwrap_or(0);
                        count_msg = format!(" [{}/{}]", cur, app.search_matches.len());
                    }
                    
                    spans.push(Span::raw(format!("{}{}", prompt_base, app.search_query)));
                    if !count_msg.is_empty() {
                        spans.push(Span::styled(count_msg, Style::default().fg(dim_color)));
                    }
                    spans.push(Span::styled(" [Alt+↑/↓] Navigate", Style::default().fg(dim_color)));
                }
                AppMode::ReplaceOne => spans.push(Span::raw(format!("Replace: {} ", app.command_input))),
                AppMode::ReplaceAll => spans.push(Span::raw(format!("Replace All: {} ", app.command_input))),
                AppMode::PromptSave => spans.push(Span::raw("Save modified script? (y/n/c) ")),
                AppMode::PromptFilename => {
                    spans.push(Span::raw(format!("Filename: {} ", app.filename_input)))
                }
                _ => {}
            }
        } else if app.status_msg.is_some() || app.show_search_highlight {
            if let Some(msg) = &app.status_msg {
                let style = if app.command_error {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default()
                        .fg(dim_color)
                        .add_modifier(Modifier::ITALIC)
                };
                spans.push(Span::styled(msg, style));
            }
            
            if app.show_search_highlight && !app.search_matches.is_empty() {
                spans.push(Span::styled(" [Alt+↑/↓] Nav [r] Replace [R] Replace All", Style::default().fg(dim_color)));
            }
        } else {
            spans.push(Span::styled(
                "F1 Reference",
                Style::default().fg(dim_color),
            ));
        }

        // Sprint progress (if active)
        if let Some(GoalType::Sprint {
            start_time,
            duration,
            start_words,
            ..
        }) = &app.active_goal
        {
            let elapsed = start_time.elapsed();
            let pct = (elapsed.as_secs_f64() / duration.as_secs_f64()).min(1.0);
            let bar_width = 8;
            let filled = (pct * bar_width as f64) as usize;
            let empty = bar_width - filled;

            let remaining = duration.saturating_sub(elapsed);
            let rem_min = remaining.as_secs() / 60;
            let rem_sec = remaining.as_secs() % 60;

            let current_words = app.total_word_count();
            let words_written = current_words.saturating_sub(*start_words);

            let sprint_msg = format!(
                " | Sprint [{}{}] {:02}:{:02} +{}w",
                "█".repeat(filled),
                "░".repeat(empty),
                rem_min,
                rem_sec,
                words_written
            );
            spans.push(Span::styled(sprint_msg, Style::default().fg(mode_bg)));
        }

        // Right-side info: word count, line count, cursor position
        let mut right_spans = Vec::new();

        let current_context_mode = if app.mode == AppMode::Command || app.mode == AppMode::Search {
            app.previous_mode
        } else {
            app.mode
        };

        if current_context_mode == AppMode::IndexCards {
            let cards_len = app.extract_scene_cards().len();
            right_spans.push(Span::styled(sep, sep_style));
            right_spans.push(Span::styled(
                format!("{} Scenes", cards_len),
                Style::default().fg(dim_color),
            ));
        } else {
            let word_count = app.total_word_count();
            let pos_str = format!("Ln {}, Col {}", app.cursor_y + 1, app.cursor_x + 1);

            right_spans.push(Span::styled(sep, sep_style));
            right_spans.push(Span::styled(
                format!("{} words", word_count),
                Style::default().fg(dim_color),
            ));
            right_spans.push(Span::styled(sep, sep_style));
            right_spans.push(Span::styled(
                pos_str,
                Style::default().fg(dim_color),
            ));
        }
        // Closing bracket
        right_spans.push(Span::styled(" ]", sep_style));

        let left_width: usize = spans
            .iter()
            .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
            .sum();
        let right_width: usize = right_spans
            .iter()
            .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
            .sum();
        let total_width = footer_area.width as usize;

        if total_width > left_width + right_width {
            let pad_len = total_width - left_width - right_width;
            spans.push(Span::raw(" ".repeat(pad_len)));
        }

        spans.extend(right_spans);
        f.render_widget(Paragraph::new(Line::from(spans)), footer_area);

        // Cursor Handling for Footer Modes
        if matches!(app.mode, AppMode::Search | AppMode::Command | AppMode::PromptSave | AppMode::PromptFilename | AppMode::ReplaceOne | AppMode::ReplaceAll) && footer_area.height > 0 {
            // Calculate prefix width of the left side content
            let fname = app.file.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "New Script".to_string());
            let dirty_str = if app.dirty { "*" } else { "" };
            let lock_str = if app.config.production_lock { " 🔒" } else { "" };
            
            let prefix_w = 2 // "[ "
                + UnicodeWidthStr::width(mode_str.trim())
                + 3 // " | "
                + UnicodeWidthStr::width(fname.as_str()) + UnicodeWidthStr::width(dirty_str) + UnicodeWidthStr::width(lock_str)
                + 3; // " | "
            
            let (input_prefix, input_content) = match app.mode {
                AppMode::Search => {
                    let pb = if app.last_search.is_empty() { "Search: ".to_string() } else { format!("Search [{}]: ", app.last_search) };
                    (pb, app.search_query.clone())
                }
                AppMode::ReplaceOne => ("Replace: ".to_string(), app.command_input.clone()),
                AppMode::ReplaceAll => ("Replace All: ".to_string(), app.command_input.clone()),
                AppMode::Command => ("/".to_string(), app.command_input.clone()),
                AppMode::PromptFilename => ("Filename: ".to_string(), app.filename_input.clone()),
                AppMode::PromptSave => ("Save modified script? (y/n/c) ".to_string(), "".to_string()),
                _ => (String::new(), String::new()),
            };
            
            let cur_x = footer_area.x + (prefix_w + UnicodeWidthStr::width(input_prefix.as_str()) + UnicodeWidthStr::width(input_content.as_str())) as u16;
            f.set_cursor_position((cur_x, footer_area.y));
        }
    }

    // -- Screen Blink Effect --
    if app.flash_timer.is_some() {
        f.render_widget(
            Block::default().style(Style::default().bg(Color::White)),
            area,
        );
    }

    // -- Cursor Handling --
    if app.mode != AppMode::Command && app.mode != AppMode::Home {
        let (vis_row, vis_x) = find_visual_cursor(&app.layout, app.cursor_y, app.cursor_x);

        match app.mode {
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

    // -- Minimalist Home Screen --
    if app.mode == AppMode::Home {
        draw_home(f, app);
    }

    if app.mode == AppMode::FilePicker {
        draw_file_picker(f, app, area);
    }

    if app.mode == AppMode::Snapshots {
        draw_snapshots(f, app);
    }

    if app.mode == AppMode::SprintStat {
        draw_sprint_stats(f, app);
    }

    if app.mode == AppMode::XRay {
        draw_xray(f, app);
    }

    if current_view_mode == AppMode::IndexCards {
        draw_index_cards(f, app, text_area);
    }
}




