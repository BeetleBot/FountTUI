use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
};
use std::collections::HashSet;
use unicode_width::UnicodeWidthStr;
use crate::{
    app::{App, AppMode, EnsembleItem},
    formatting::{RenderConfig, StringCaseExt, render_inline},
    layout::{find_visual_cursor, strip_sigils},
    types::{PAGE_WIDTH, base_style, LineType},
    theme::{HexColor},
};

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
            AppMode::Normal => (" NORMAL ", Color::from(theme.ui.normal_mode_bg.clone())),
            AppMode::Command => (" COMMAND ", Color::from(theme.ui.command_mode_bg.clone())),
            AppMode::SceneNavigator => (" NAVIGATOR ", Color::from(theme.ui.navigator_mode_bg.clone())),
            AppMode::SettingsPane => (" SETTINGS ", Color::from(theme.ui.settings_mode_bg.clone())),
            AppMode::ExportPane => (" EXPORT ", Color::from(theme.ui.normal_mode_bg.clone())),
            AppMode::Shortcuts => (" LEGEND ", Color::from(theme.ui.normal_mode_bg.clone())),
            AppMode::Search => (" SEARCH ", Color::from(theme.ui.search_mode_bg.clone())),
            AppMode::Home => (" HOME ", Color::from(theme.ui.normal_mode_bg.clone())),
            AppMode::FilePicker => (" FILE ", Color::from(theme.ui.normal_mode_bg.clone())),
            _ => (" PROMPT ", Color::from(theme.ui.command_mode_bg.clone())),
        };


    let is_prompt = app.mode != AppMode::Normal;
    let has_status = app.status_msg.is_some();

    let show_bottom = !app.config.focus_mode || is_prompt || has_status;

    let _in_command_mode = app.mode == AppMode::Command;
    let footer_height = if show_bottom { 1 } else { 0 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(footer_height),
        ])
        .split(area);

    let (mut text_area, footer_area) = (chunks[0], chunks[1]);

    app.sidebar_area = Rect::default();
    if app.mode == AppMode::SceneNavigator || app.mode == AppMode::CharacterNavigator {
        let side_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(41), Constraint::Length(1), Constraint::Min(0)])
            .split(text_area);
        app.sidebar_area = side_chunks[0];
        let shadow_area = side_chunks[1];
        text_area = side_chunks[2];

        // Draw shadow
        let shadow_color = theme.ui.shadow_color.clone().unwrap_or(HexColor("black".into()));
        f.render_widget(Block::default().style(Style::default().bg(Color::from(shadow_color))), shadow_area);

        let sidebar_block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(Color::from(theme.ui.dim.clone())));
        f.render_widget(sidebar_block, app.sidebar_area);
    }

    app.settings_area = Rect::default();
    if app.mode == AppMode::SettingsPane || app.mode == AppMode::Shortcuts || app.mode == AppMode::ExportPane {
        let side_width = if app.mode == AppMode::Shortcuts { 41 } else { 34 };
        let side_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(1), Constraint::Length(side_width)])
            .split(text_area);
        text_area = side_chunks[0];
        let shadow_area = side_chunks[1];
        app.settings_area = side_chunks[2];

        // Draw shadow (left of panel)
        let shadow_color = theme.ui.shadow_color.clone().unwrap_or(HexColor("black".into()));
        f.render_widget(Block::default().style(Style::default().bg(Color::from(shadow_color))), shadow_area);

        let settings_block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(mode_bg));
        f.render_widget(settings_block, app.settings_area);
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
        let border_color = theme.sidebar.border.clone().map(Color::from).unwrap_or(Color::DarkGray);
        let selected_bg = theme.sidebar.item_selected_bg.clone().map(Color::from).unwrap_or(mode_bg);
        let selected_fg = theme.sidebar.item_selected_fg.clone().map(Color::from).unwrap_or(Color::Black);
        let dim_color = theme.sidebar.item_dimmed.clone().map(Color::from).unwrap_or(Color::DarkGray);
        let header_color = theme.sidebar.section_header.clone().map(Color::from).unwrap_or(mode_bg);

        let items: Vec<ListItem> = app
            .scenes
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = i == app.selected_scene;
                let mut lines = Vec::new();
                
                let line_style = Style::default().fg(border_color).add_modifier(Modifier::DIM);

                if item.is_section {
                    // Section Header (ACT I, etc.)
                    let style = if is_selected {
                        Style::default().fg(selected_fg).bg(selected_bg).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(header_color).add_modifier(Modifier::BOLD)
                    };
                    
                    let prefix = if is_selected { " ⟫ " } else { "   " };
                    lines.push(Line::from(vec![
                        Span::styled(prefix, style),
                        Span::styled(item.label.to_uppercase(), style),
                    ]));
                    
                    // Add a connecting line start below the section if the next item is a scene
                    if i + 1 < app.scenes.len() && !app.scenes[i+1].is_section {
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
                        Style::default().fg(selected_fg).bg(selected_bg).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    
                    if let Some(c) = &theme.ui.foreground {
                        if !is_selected {
                            base_style = base_style.fg(Color::from(c.clone()));
                        }
                    }

                    if let Some(c) = item.color {
                        if !is_selected {
                            base_style = base_style.fg(c);
                        }
                    }
                    
                    let dim_style = if is_selected {
                        base_style
                    } else {
                        Style::default().fg(dim_color).add_modifier(Modifier::DIM)
                    };

                    let prefix = if is_selected { " ⟫ " } else { "   " };
                    
                    // Determine if this is the last scene in the section
                    let is_last_in_section = i + 1 == app.scenes.len() || app.scenes[i+1].is_section;
                    let connector = if is_last_in_section { "└─ " } else { "├─ " };

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
                                        Span::styled(current_line.clone(), dim_style.add_modifier(Modifier::ITALIC)),
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
                                    Span::styled(current_line, dim_style.add_modifier(Modifier::ITALIC)),
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
                            Span::styled("─".repeat(30), Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)),
                        ]));
                    } else {
                        lines.push(Line::from(""));
                    }
                }
                
                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items).highlight_style(Style::default());
        f.render_stateful_widget(list, app.sidebar_area.inner(ratatui::layout::Margin { horizontal: 0, vertical: 1 }), &mut app.navigator_state);
    }

    if app.mode == AppMode::CharacterNavigator {
        let border_color = theme.sidebar.border.clone().map(Color::from).unwrap_or(Color::DarkGray);
        let selected_bg = theme.sidebar.item_selected_bg.clone().map(Color::from).unwrap_or(mode_bg);
        let selected_fg = theme.sidebar.item_selected_fg.clone().map(Color::from).unwrap_or(Color::Black);
        let dim_color = theme.sidebar.item_dimmed.clone().map(Color::from).unwrap_or(Color::DarkGray);
        let header_color = theme.sidebar.section_header.clone().map(Color::from).unwrap_or(mode_bg);

        let items: Vec<ListItem> = app
            .ensemble_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = i == app.selected_ensemble_idx;
                let mut lines = Vec::new();
                
                let line_style = Style::default().fg(border_color).add_modifier(Modifier::DIM);
                let base_style = if is_selected {
                    Style::default().fg(selected_fg).bg(selected_bg).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let dim_style = if is_selected {
                    base_style
                } else {
                    Style::default().fg(dim_color).add_modifier(Modifier::DIM)
                };

                let prefix = if is_selected { " ⟫ " } else { "   " };
                
                match item {
                    EnsembleItem::CharacterHeader(char_idx) => {
                        let char_item = &app.character_stats[*char_idx];
                        lines.push(Line::from(vec![
                            Span::styled(prefix, base_style),
                            Span::styled(char_item.name.clone(), Style::default().fg(header_color).add_modifier(Modifier::BOLD)),
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
                            spans.push(Span::styled(format!(" {}", h), line_style.add_modifier(Modifier::ITALIC)));
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

        let title = format!(" [ ENSEMBLE ({}) ] ", app.character_stats.len());
        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
            )
            .highlight_style(Style::default());

        f.render_stateful_widget(list, app.sidebar_area.inner(ratatui::layout::Margin { horizontal: 0, vertical: 1 }), &mut app.ensemble_state);
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
                    Style::default().fg(Color::Black).bg(mode_bg).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                
                let (icon, icon_style) = if label == "Theme" {
                    ("󰔎 ", Style::default().fg(Color::from(theme.ui.normal_mode_bg.clone())))
                } else if *value { 
                    ("󰄬 ", Style::default().fg(Color::Green)) 
                } else { 
                    ("󰄱 ", Style::default().fg(Color::DarkGray)) 
                };

                let line = if label == "Theme" {
                    Line::from(vec![
                        Span::styled(if is_selected { " ⟫ " } else { "   " }, style),
                        Span::styled(icon, if is_selected { style } else { icon_style }),
                        Span::styled(format!("{}: {}", label, theme_name), style),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled(if is_selected { " ⟫ " } else { "   " }, style),
                        Span::styled(icon, if is_selected { style } else { icon_style }),
                        Span::styled(label, style),
                    ])
                };

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items);
        f.render_widget(list, app.settings_area.inner(ratatui::layout::Margin { horizontal: 0, vertical: 1 }));
    }

    if app.mode == AppMode::ExportPane {
        let format_label = match app.config.export_format.as_str() {
            "pdf" => "PDF",
            "fountain" => "Fountain",
            "fdx" => "FDX (Coming Soon)",
            _ => "PDF"
        };
        
        let report_label = match app.config.report_format.as_str() {
            "csv_scene" => "Scene List (CSV)",
            "csv_char" => "Character Report (CSV)",
            _ => "Scene List (CSV)"
        };

        let header_style = Style::default().fg(mode_bg).add_modifier(Modifier::BOLD);
        
        let mut visual_items = Vec::new();

        let render_item = |idx: usize, label: &str, app: &App| -> ListItem {
            let is_selected = idx == app.selected_export_option;
            let style = if is_selected {
                Style::default().fg(Color::Black).bg(mode_bg).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(if is_selected { " ⟫ " } else { "   " }, style),
                Span::styled(label.to_string(), style),
            ]))
        };

        visual_items.push(ListItem::new(Line::from(vec![Span::styled("  ── SCREENPLAY EXPORT ──", header_style)])));
        visual_items.push(render_item(0, &format!(" Format: {}", format_label), app));
        
        visual_items.push(render_item(1, &format!(" Paper: {}", app.config.paper_size.to_uppercase()), app));
        visual_items.push(render_item(2, &format!(" Bold Headings: {}", if app.config.export_bold_scene_headings { "[X]" } else { "[ ]" }), app));
        
        visual_items.push(ListItem::new(Line::from(vec![Span::raw("")])));
        visual_items.push(render_item(3, " [ EXPORT SCREENPLAY ]", app));
        
        visual_items.push(ListItem::new(Line::from(vec![Span::raw("")])));
        visual_items.push(ListItem::new(Line::from(vec![Span::styled("  ── PRODUCTION REPORTS ──", header_style)])));
        visual_items.push(render_item(4, &format!(" Type: {}", report_label), app));
        
        visual_items.push(ListItem::new(Line::from(vec![Span::raw("")])));
        visual_items.push(render_item(5, " [ EXPORT REPORT ]", app));

        let list = List::new(visual_items);
        f.render_widget(list, app.settings_area.inner(ratatui::layout::Margin { horizontal: 0, vertical: 1 }));
    }

    if app.mode == AppMode::Shortcuts {
        let categories = vec![
            (" GLOBAL KEYS ", vec![
                ("^H / ^L  ", "Scenes / Ensemble"),
                ("^P / ^E  ", "Settings / Export"),
                ("/         ", "Help / Command mode"),
                ("F1        ", "Toggle This Panel"),
            ]),
            (" EDITING ", vec![
                ("^A / ^C  ", "Select / Copy"),
                ("^X / ^V  ", "Cut / Paste"),
                ("Tab       ", "Indent / Assist"),
                ("Enter     ", "Add Line"),
                ("Shift+Arr ", "Select Text"),
            ]),
            (" COMMANDS (/) ", vec![
                ("/w / /ww    ", "Save / Save As"),
                ("/o [path]   ", "Open script"),
                ("/new        ", "New script"),
                ("/bn / /bp   ", "Buffer Next / Prev"),
                ("/q / /q!    ", "Close / Force Close"),
                ("/wq / /ex   ", "Save&Close / Exit"),
                ("/ud / /rd   ", "Undo / Redo"),
                ("/copy / /cut", "Copy / Cut"),
                ("/paste      ", "Paste"),
                ("/selectall  ", "Select All"),
                ("/search     ", "Global Search"),
                ("/renum      ", "Renumber Scenes"),
                ("/clearnum   ", "Clear Numbers"),
                ("/injectnum  ", "Tag Scene (#)"),
                ("/locknum    ", "Lock Numbering"),
                ("/unlocknum  ", "Unlock Numbering"),
                ("/addtitle   ", "Insert Title Page"),
                ("/export     ", "Export PDF"),
                ("/[line]     ", "Jump to Line"),
                ("/s[num]     ", "Jump to Scene"),
                ("/pos / /home", "Cursor Info / Home"),
            ]),
            (" ZEN SETS (/set) ", vec![
                ("focus     ", "Zen Mode (Clean UI)"),
                ("typewriter", "Lock cursor center"),
                ("markup    ", "Hide syntax markers"),
                ("pagenums  ", "Show page counts"),
                ("scenenums ", "Show scene numbers"),
                ("contd     ", "Auto (CONT'D) tags"),
                ("autosave  ", "Save every 30s"),
            ]),
        ];

        let mut items = Vec::new();
        for (idx, (cat, shortcuts)) in categories.iter().enumerate() {
            if idx > 0 {
                items.push(ListItem::new(""));
            }
            
            let header_str = format!(" ━━━ {} ", cat.trim());
            let header_line = header_str.clone() + &"━".repeat(40usize.saturating_sub(header_str.len()));
            
            items.push(ListItem::new(Line::from(vec![
                Span::styled(header_line, Style::default().fg(mode_bg).add_modifier(Modifier::BOLD)),
            ])));
            
            items.push(ListItem::new(""));

            for (key, desc) in shortcuts {
                let key_clean = key.trim();
                let key_padded = format!("  {:<16} ", key_clean);
                
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(key_padded, Style::default().fg(mode_bg).add_modifier(Modifier::BOLD)),
                    Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(*desc, Style::default().fg(Color::Gray)),
                ])));
            }
        }

        let list = List::new(items);
        f.render_stateful_widget(list, app.settings_area.inner(ratatui::layout::Margin { horizontal: 0, vertical: 1 }), &mut app.shortcuts_state);
    }

    // ── Footer rendering ──────────────────────────────────────────────────
    if footer_area.height > 0 {
        let mut spans = Vec::new();

        spans.push(Span::styled(mode_str, Style::default().fg(mode_bg).add_modifier(Modifier::BOLD)));
        spans.push(Span::raw(" │ "));

        let fname = app.file.as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "[No Name]".to_string());
        let dirty_str = if app.dirty { " [+]" } else { "" };
        let lock_str = if app.config.production_lock { " (L)" } else { "" };
        spans.push(Span::styled(format!("{}{}{}", fname, dirty_str, lock_str), Style::default().fg(mode_bg)));
        spans.push(Span::raw(" │ "));

        if app.mode == AppMode::Command {
            let cmd_style = if app.command_error {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::BOLD)
            };
            spans.push(Span::styled("/", cmd_style));
            spans.push(Span::styled(&app.command_input, cmd_style));
            
            if app.command_input.is_empty() && !app.command_error {
                spans.push(Span::styled(" type a command...", Style::default().fg(Color::DarkGray)));
            }
        } else if matches!(app.mode, AppMode::Search | AppMode::PromptSave | AppMode::PromptFilename) {
            match app.mode {
                AppMode::Search => {
                    let prompt_base = if app.last_search.is_empty() {
                        "SEARCH: ".to_string()
                    } else {
                        format!("SEARCH [{}]: ", app.last_search)
                    };
                    spans.push(Span::raw(format!("{}{}", prompt_base, app.search_query)));
                }
                AppMode::PromptSave => spans.push(Span::raw("SAVE MODIFIED SCRIPT? (Y/N/C) ")),
                AppMode::PromptFilename => spans.push(Span::raw(format!("FILENAME: {} ", app.filename_input))),
                _ => {}
            }
        } else if let Some(msg) = &app.status_msg {
            let style = if app.command_error {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
            };
            spans.push(Span::styled(msg, style));
        } else {
            spans.push(Span::styled("COMMANDS [F1]", Style::default().fg(Color::DarkGray)));
        }

        let word_count = app.total_word_count();
        let line_count = app.lines.len();
        let pos_str = format!("{}:{}", app.cursor_y + 1, app.cursor_x + 1);

        let mut right_spans = Vec::new();
        right_spans.push(Span::raw(" │ "));
        right_spans.push(Span::styled(format!("{}W {}L", word_count, line_count), Style::default().fg(Color::DarkGray)));
        right_spans.push(Span::raw(" │ "));
        right_spans.push(Span::styled(pos_str, Style::default().add_modifier(Modifier::BOLD)));

        let left_width: usize = spans.iter().map(|s| UnicodeWidthStr::width(s.content.as_ref())).sum();
        let right_width: usize = right_spans.iter().map(|s| UnicodeWidthStr::width(s.content.as_ref())).sum();
        let total_width = footer_area.width as usize;

        if total_width > left_width + right_width {
            let pad_len = total_width - left_width - right_width;
            spans.push(Span::raw(" ".repeat(pad_len)));
        }
        
        spans.extend(right_spans);
        f.render_widget(Paragraph::new(Line::from(spans)), footer_area);

        if app.mode == AppMode::Command {
            let mode_w = UnicodeWidthStr::width(mode_str);
            let fname_w = UnicodeWidthStr::width(fname.as_str()) + UnicodeWidthStr::width(dirty_str);
            let cur_x = footer_area.x + (mode_w + 3 + fname_w + 3 + 1 + UnicodeWidthStr::width(app.command_input.as_str())) as u16;
            f.set_cursor_position((cur_x, footer_area.y));
        }
    }

    // -- Cursor Handling --
    if app.mode != AppMode::Command && app.mode != AppMode::Home {
        let (vis_row, vis_x) = find_visual_cursor(&app.layout, app.cursor_y, app.cursor_x);

        match app.mode {
            AppMode::Normal => {
                let cur_screen_y = text_area.y + pad_top as u16 + (vis_row.saturating_sub(app.scroll)) as u16;
                let cur_screen_x = text_area.x + global_pad + vis_x;
                if cur_screen_y < text_area.y + text_area.height {
                    f.set_cursor_position((cur_screen_x, cur_screen_y));
                }
            }
            AppMode::Search if footer_area.height > 0 => {
                let prompt_base = if app.last_search.is_empty() {
                    "Search: ".to_string()
                } else {
                    format!("Search [{}]: ", app.last_search)
                };
                let query_w = unicode_width::UnicodeWidthStr::width(prompt_base.as_str())
                    + unicode_width::UnicodeWidthStr::width(app.search_query.as_str());
                let center_start = (footer_area.width as usize).saturating_sub(query_w) / 2;
                let cur_screen_x = footer_area.x + center_start as u16 + unicode_width::UnicodeWidthStr::width(prompt_base.as_str()) as u16;
                f.set_cursor_position((cur_screen_x, footer_area.y));
            }
            AppMode::PromptSave | AppMode::PromptFilename if footer_area.height > 0 => {
                let prompt_base = if app.mode == AppMode::PromptSave { "Save changes? (y/n): " } else { "File Name to Write: " };
                let input = if app.mode == AppMode::PromptSave { "" } else { app.filename_input.as_str() };
                let query_w = unicode_width::UnicodeWidthStr::width(prompt_base) + unicode_width::UnicodeWidthStr::width(input);
                let center_start = (footer_area.width as usize).saturating_sub(query_w) / 2;
                let cur_screen_x = footer_area.x + center_start as u16 + unicode_width::UnicodeWidthStr::width(prompt_base) as u16;
                f.set_cursor_position((cur_screen_x, footer_area.y));
            }
            _ => {}
        }
    }

    // -- Floating Home Screen --
    if app.mode == AppMode::Home {
        let panel_w = 66u16;
        let panel_h = 24u16;
        let x = area.x + area.width.saturating_sub(panel_w) / 2;
        let y = area.y + area.height.saturating_sub(panel_h) / 2;
        let panel = Rect { x, y, width: panel_w.min(area.width), height: panel_h.min(area.height) };

        f.render_widget(ratatui::widgets::Clear, panel);

        let outer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        f.render_widget(outer_block, panel);

        let inner = Rect {
            x: panel.x + 1,
            y: panel.y + 1,
            width: panel.width.saturating_sub(2),
            height: panel.height.saturating_sub(2),
        };

        let w = inner.width as usize;
        let accent = Color::LightCyan;
        let dim = Color::DarkGray;
        let sel_bg = mode_bg;

        let logo_style = Style::default().fg(accent).add_modifier(Modifier::BOLD);
        let dim_style = Style::default().fg(dim);
        
        let center = |text: &str| -> String {
            let text_w = unicode_width::UnicodeWidthStr::width(text);
            let pad = (w.saturating_sub(text_w)) / 2;
            format!("{}{}", " ".repeat(pad), text)
        };

        let mut home_lines = Vec::new();
        home_lines.push(Line::from(""));

        for row in &[
            "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557} \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557} \u{2588}\u{2588}\u{2557}   \u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2588}\u{2557}   \u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}",
            "\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{2550}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}  \u{2588}\u{2588}\u{2551}\u{255a}\u{2550}\u{2550}\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{255d}",
            "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2557}  \u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2554}\u{2588}\u{2588}\u{2557} \u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}   ",
            "\u{2588}\u{2588}\u{2554}\u{2550}\u{2550}\u{255d}  \u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}\u{2588}\u{2588}\u{2551}\u{255a}\u{2588}\u{2588}\u{2557}\u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}   ",
            "\u{2588}\u{2588}\u{2551}     \u{255a}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2554}\u{255d}\u{255a}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2554}\u{255d}\u{2588}\u{2588}\u{2551} \u{255a}\u{2588}\u{2588}\u{2588}\u{2588}\u{2551}   \u{2588}\u{2588}\u{2551}   ",
            "\u{255a}\u{2550}\u{255d}      \u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}  \u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d} \u{255a}\u{2550}\u{255d}  \u{255a}\u{2550}\u{2550}\u{2550}\u{255d}   \u{255a}\u{2550}\u{255d}   ",
        ] {
            home_lines.push(Line::from(Span::styled(center(row), logo_style)));
        }

        home_lines.push(Line::from(""));
        home_lines.push(Line::from(Span::styled(center("Screenwriting in the terminal. Distraction-free."), dim_style)));
        home_lines.push(Line::from(""));
        home_lines.push(Line::from(Span::styled(center(&"\u{2500}".repeat(42)), dim_style)));
        home_lines.push(Line::from(""));

        let menu = vec![
            ("New Script", "N", "Start a blank fountain screenplay"),
            ("Open File", "O", "Browse for a .fountain script"),
            ("Tutorial", "T", "Getting started guide"),
            ("Exit", "Q", "Quit Fount"),
        ];

        for (i, (label, key, hint)) in menu.iter().enumerate() {
            let is_sel = i == app.home_selected;
            let display = format!("[{}]  {:<12} \u{2014}  {}", key, label, hint);
            let centered = center(&display);
            if is_sel {
                let s = Style::default().fg(Color::Black).bg(sel_bg).add_modifier(Modifier::BOLD);
                home_lines.push(Line::from(Span::styled(centered, s)));
            } else {
                home_lines.push(Line::from(Span::styled(centered, dim_style)));
            }
            home_lines.push(Line::from(""));
        }

        home_lines.push(Line::from(Span::styled(center(&"\u{2500}".repeat(42)), dim_style)));
        home_lines.push(Line::from(""));
        home_lines.push(Line::from(Span::styled(center("github.com/BeetleBot/Fount"), Style::default().fg(dim).add_modifier(Modifier::DIM))));

        f.render_widget(Paragraph::new(home_lines), inner);
    }

    if app.mode == AppMode::FilePicker {
        draw_file_picker(f, app, area);
    }
}

fn draw_file_picker(f: &mut Frame, app: &mut App, area: Rect) {
    let state = if let Some(ref mut s) = app.file_picker { s } else { return; };
    
    let block_w = 70u16.min(area.width);
    let block_h = 24u16.min(area.height);
    let x = area.x + (area.width - block_w) / 2;
    let y = area.y + (area.height - block_h) / 2;
    let block_area = Rect::new(x, y, block_w, block_h);

    f.render_widget(Clear, block_area);
    
    let title = match state.action {
        crate::app::FilePickerAction::Open => " Open File ",
        crate::app::FilePickerAction::Save => " Save As ",
        crate::app::FilePickerAction::ExportReport => " Export Report ",
        crate::app::FilePickerAction::ExportScript => " Export Script ",
    };

    let block = Block::default()
        .title(Span::styled(title, Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(block, block_area);

    let inner_area = block_area.inner(ratatui::layout::Margin { horizontal: 2, vertical: 1 });
    
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
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(dir_str, Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))
    ])), layout[0]);

    // 2. List of items
    let items_len = state.items.len();
    let selected_idx = state.list_state.selected().unwrap_or(0);
    let mut display_items: Vec<ListItem> = state.items.iter().enumerate().map(|(i, path)| {
        let is_selected = i == selected_idx;
        let is_dir = path.is_dir();
        
        let name = if let Some(parent) = state.current_dir.parent() {
            if path == parent {
                ".. (Parent Directory)".to_string()
            } else {
                path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "/".to_string())
            }
        } else {
            path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "/".to_string())
        };

        let (icon, color) = if is_dir { (" ", Color::LightBlue) } else { ("󰈙 ", Color::White) };
        
        let style = if is_selected {
            Style::default().bg(Color::LightMagenta).fg(Color::Black)
        } else {
            Style::default().fg(color)
        };

        ListItem::new(Line::from(vec![
            Span::styled(if is_selected { " ⟫ " } else { "   " }, style),
            Span::styled(icon, style),
            Span::styled(name, style),
        ]))
    }).collect();

    // Add virtual item for custom filename if in Save mode
    if state.action != crate::app::FilePickerAction::Open && !state.filename_input.is_empty() {
        let is_selected = selected_idx == items_len;
        let style = if is_selected {
            Style::default().bg(Color::LightGreen).fg(Color::Black)
        } else {
            Style::default().fg(Color::LightGreen)
        };
        display_items.push(ListItem::new(Line::from(vec![
            Span::styled(if is_selected { " ⟫ " } else { "   " }, style),
            Span::styled("󰒓 ", style),
            Span::styled(format!("Confirm: {}", state.filename_input), style.add_modifier(Modifier::BOLD)),
        ])));
    }

    let list = List::new(display_items).highlight_style(Style::default());
    f.render_stateful_widget(list, layout[1], &mut state.list_state);

    // 3. Input Label
    if state.action != crate::app::FilePickerAction::Open {
        f.render_widget(Paragraph::new(Line::from(vec![
            Span::styled(" Filename: ", Style::default().fg(Color::DarkGray))
        ])), layout[2]);

        // 4. Filename Input
        let input_style = Style::default().fg(Color::White).bg(Color::Rgb(30, 30, 30));
        f.render_widget(Paragraph::new(Line::from(vec![
            Span::styled(format!("  {}", state.filename_input), input_style)
        ])).block(Block::default().borders(Borders::NONE)), layout[3]);
        
        // Cursor for input
        let cursor_pos = layout[3].x + 2 + UnicodeWidthStr::width(state.filename_input.as_str()) as u16;
        f.set_cursor_position((cursor_pos, layout[3].y));
    }
}
