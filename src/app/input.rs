use std::io;
use std::path::PathBuf;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use crate::app::{App, AppMode, EnsembleItem, FilePickerAction};

impl App {

    fn handle_mouse_cursor(
        &mut self,
        mouse_y: u16,
        mouse_x: u16,
        cursor_moved: &mut bool,
        update_target_x: &mut bool,
    ) {
        // Adjust for title bar (fixed at 1 line if not focus mode)
        let title_height = if !self.config.focus_mode { 1 } else { 0 };
        let vis_y = (mouse_y as usize).saturating_sub(title_height) + self.scroll;
        
        if let Some(row) = self.layout.get(vis_y) {
            self.cursor_y = row.line_idx;
            
            // Need to know if this is the last visual row for the logical line
            let is_last = self.layout.get(vis_y + 1).map(|next| next.line_idx != row.line_idx).unwrap_or(true);
            
            self.cursor_x = row.visual_to_logical_x(mouse_x, is_last);
            *cursor_moved = true;
            *update_target_x = true;
        }
    }

    pub fn handle_event(
        &mut self,
        ev: Event,
        update_target_x: &mut bool,
        text_changed: &mut bool,
        cursor_moved: &mut bool,
    ) -> io::Result<bool> {
        if let Event::Mouse(mouse_event) = ev {
            self.clear_status();
            if self.show_search_highlight {
                self.show_search_highlight = false;
            }

            match mouse_event.kind {
                MouseEventKind::ScrollUp => {
                    if self.mode == AppMode::SceneNavigator
                        && mouse_event.column < self.sidebar_area.x + self.sidebar_area.width
                    {
                        if self.selected_scene > 0 {
                            self.selected_scene -= 1;
                            self.navigator_state.select(Some(self.selected_scene));
                        }
                    } else {
                        self.clear_selection();
                        self.move_up();
                        *cursor_moved = true;
                    }
                }
                MouseEventKind::ScrollDown => {
                    if self.mode == AppMode::SceneNavigator
                        && mouse_event.column < self.sidebar_area.x + self.sidebar_area.width
                    {
                        if self.selected_scene < self.scenes.len() - 1 {
                            self.selected_scene += 1;
                            self.navigator_state.select(Some(self.selected_scene));
                        }
                    } else {
                        self.clear_selection();
                        self.move_down();
                        *cursor_moved = true;
                    }
                }
                MouseEventKind::Down(MouseButton::Left) => {
                    if self.mode == AppMode::Normal {
                        self.clear_selection();
                        self.handle_mouse_cursor(
                            mouse_event.row,
                            mouse_event.column,
                            cursor_moved,
                            update_target_x,
                        );
                        self.selection_anchor = Some((self.cursor_y, self.cursor_x));
                    } else if self.mode == AppMode::SettingsPane {
                        let x = mouse_event.column;
                        let y = mouse_event.row;
                        if x >= self.settings_area.x
                            && x < self.settings_area.x + self.settings_area.width
                            && y >= self.settings_area.y
                            && y < self.settings_area.y + self.settings_area.height
                        {
                            let rel_y = (y - self.settings_area.y) as usize;
                            if rel_y > 0 && rel_y <= 10 {
                                let setting_idx = rel_y - 1;
                                self.selected_setting = setting_idx;

                                if x >= self.settings_area.x + self.settings_area.width - 5 {
                                    let desc = match self.selected_setting {
                                        0 => {
                                            "Always center the cursor, even at the start of the file."
                                        }
                                        1 => "Periodically save the current buffer to disk.",
                                        2 => "Display scene numbers in the left margin.",
                                        3 => "Display page numbers in the right margin.",
                                        4 => "Hide Fountain markup unless the line is active.",
                                        5 => "Enable scene heading/character name completion.",
                                        6 => "Automatically append (CONT'D) to character names.",
                                        7 => "Insert paragraph breaks after screenplay elements.",
                                        8 => "Hide the UI bars for a distraction-free view.",
                                        _ => "",
                                    };
                                    if !desc.is_empty() {
                                        self.set_status(desc);
                                    }
                                } else {
                                    match self.selected_setting {
                                        0 => {
                                            self.config.strict_typewriter_mode =
                                                !self.config.strict_typewriter_mode
                                        }
                                        1 => self.config.auto_save = !self.config.auto_save,
                                        2 => {
                                            self.config.show_scene_numbers =
                                                !self.config.show_scene_numbers
                                        }
                                        3 => {
                                            self.config.show_page_numbers =
                                                !self.config.show_page_numbers
                                        }
                                        4 => self.config.hide_markup = !self.config.hide_markup,
                                        5 => self.config.autocomplete = !self.config.autocomplete,
                                        6 => self.config.auto_contd = !self.config.auto_contd,
                                        7 => {
                                            self.config.auto_paragraph_breaks =
                                                !self.config.auto_paragraph_breaks
                                        }
                                        8 => self.config.focus_mode = !self.config.focus_mode,
                                        _ => {}
                                    }
                                    *text_changed = true;
                                }
                            }
                        }
                    } else if self.mode == AppMode::SceneNavigator {
                        let x = mouse_event.column;
                        let y = mouse_event.row;
                        if x < self.sidebar_area.x + self.sidebar_area.width
                            && y >= self.sidebar_area.y
                            && y < self.sidebar_area.y + self.sidebar_area.height
                        {
                            let mut current_y = self.sidebar_area.y as usize + 1;
                            let offset = self.navigator_state.offset();
                            if self.mode == AppMode::SceneNavigator {
                                for i in offset..self.scenes.len() {
                                    let h = self.calculate_scene_height(&self.scenes[i]);
                                    if (y as usize) < current_y + h {
                                        self.selected_scene = i;
                                        self.navigator_state.select(Some(i));

                                        let line_idx = self.scenes[i].line_idx;
                                        self.cursor_y = line_idx;
                                        self.cursor_x = 0;
                                        *cursor_moved = true;
                                        *update_target_x = true;
                                        break;
                                    }
                                    current_y += h;
                                    if current_y >= (self.sidebar_area.y + self.sidebar_area.height) as usize {
                                        break;
                                    }
                                }
                            } else if self.mode == AppMode::CharacterNavigator {
                                for i in offset..self.ensemble_items.len() {
                                    let h = 1; // Flat list, 1 line per item
                                    if (y as usize) < current_y + h {
                                        match self.ensemble_items[i] {
                                            EnsembleItem::CharacterHeader(_) | EnsembleItem::SceneLink(..) => {
                                                self.selected_ensemble_idx = i;
                                                self.ensemble_state.select(Some(i));

                                                if let EnsembleItem::SceneLink(_, line_idx, _) = self.ensemble_items[i] {
                                                    self.cursor_y = line_idx;
                                                    self.cursor_x = 0;
                                                    self.mode = AppMode::Normal;
                                                    *cursor_moved = true;
                                                    *update_target_x = true;
                                                }
                                            }
                                            _ => {}
                                        }
                                        break;
                                    }
                                    current_y += h;
                                    if current_y >= (self.sidebar_area.y + self.sidebar_area.height) as usize {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                MouseEventKind::Drag(MouseButton::Left) => {
                    if self.mode == AppMode::Normal {
                        self.handle_mouse_cursor(
                            mouse_event.row,
                            mouse_event.column,
                            cursor_moved,
                            update_target_x,
                        );
                        *cursor_moved = true;
                    }
                }
                _ => {}
            }
            return Ok(false);
        }

        if let Event::Paste(text) = ev {
            if self.mode == AppMode::Normal {
                self.insert_str(&text);
                *text_changed = true;
                *cursor_moved = true;
                *update_target_x = true;
            }
            return Ok(false);
        }

        if let Event::Key(key) = ev {
            if key.kind != KeyEventKind::Press {
                return Ok(false);
            }

            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
            let shift = key.modifiers.contains(KeyModifiers::SHIFT);

            match self.mode {
                AppMode::Search => {
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                            self.set_status("Cancelled");
                            self.show_search_highlight = false;
                            self.search_query.clear();
                        }
                        KeyCode::Char('c') | KeyCode::Char('g') if ctrl => {
                            self.mode = AppMode::Normal;
                            self.set_status("Cancelled");
                            self.show_search_highlight = false;
                            self.search_query.clear();
                        }
                        KeyCode::Enter => {
                            self.execute_search();
                            *update_target_x = true;
                            *cursor_moved = true;
                        }
                        KeyCode::Backspace => {
                            self.search_query.pop();
                            self.update_search_regex();
                        }
                        KeyCode::Char(c) if !ctrl => {
                            self.search_query.push(c);
                            self.update_search_regex();
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::PromptSave => {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') if !ctrl => {
                            if self.file.is_some() && self.save().is_ok() {
                                if self.exit_after_save {
                                    self.close_current_buffer();
                                    return Ok(true);
                                }
                                self.mode = AppMode::Normal;
                                return Ok(false);
                            }
                            self.filename_input = self
                                .file
                                .as_ref()
                                .map(|p| p.to_string_lossy().into_owned())
                                .unwrap_or_default();
                            self.mode = AppMode::PromptFilename;
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') if !ctrl => {
                            if self.exit_after_save {
                                self.close_current_buffer();
                                return Ok(true);
                            }
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                            self.set_status("Cancelled");
                        }
                        KeyCode::Char('c') | KeyCode::Char('g') if ctrl => {
                            self.mode = AppMode::Normal;
                            self.set_status("Cancelled");
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::PromptFilename => {
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                            self.set_status("Cancelled");
                        }
                        KeyCode::Char('c') | KeyCode::Char('g') if ctrl => {
                            self.mode = AppMode::Normal;
                            self.set_status("Cancelled");
                        }
                        KeyCode::Enter => {
                            if !self.filename_input.trim().is_empty() {
                                self.file = Some(PathBuf::from(self.filename_input.trim()));
                                match self.save() {
                                    Ok(_) => {
                                        if self.exit_after_save {
                                            self.close_current_buffer();
                                            return Ok(true);
                                        }
                                        self.mode = AppMode::Normal;
                                    }
                                    Err(e) => {
                                        self.set_status(&format!("Error saving: {}", e));
                                        self.mode = AppMode::Normal;
                                    }
                                }
                            } else {
                                self.set_status("Cancelled");
                                self.mode = AppMode::Normal;
                            }
                        }
                        KeyCode::Backspace => {
                            self.filename_input.pop();
                        }
                        KeyCode::Char(c) if !ctrl => {
                            self.filename_input.push(c);
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::ExportPane => {
                    let options_count = 6;
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Char('c') | KeyCode::Char('e') | KeyCode::Char('g') if ctrl => {
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Char('h') if ctrl => {
                            self.open_scene_navigator();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.selected_export_option = self.selected_export_option.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.selected_export_option =
                                (self.selected_export_option + 1).min(options_count - 1);
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            match self.selected_export_option {
                                0 => {
                                    let formats = ["pdf", "fountain", "fdx"];
                                    if let Some(idx) = formats.iter().position(|&x| x == self.config.export_format.as_str()) {
                                        self.config.export_format = formats[(idx + 1) % formats.len()].to_string();
                                    } else {
                                        self.config.export_format = "pdf".to_string();
                                    }
                                }
                                1 => {
                                    if self.config.paper_size == "a4" {
                                        self.config.paper_size = "letter".to_string();
                                    } else {
                                        self.config.paper_size = "a4".to_string();
                                    }
                                }
                                2 => self.config.export_bold_scene_headings = !self.config.export_bold_scene_headings,
                                3 => {
                                    let (ext, default_name) = match self.config.export_format.as_str() {
                                        "pdf" => ("pdf", "screenplay.pdf"),
                                        "fountain" => ("fountain", "screenplay.fountain"),
                                        "fdx" => {
                                            self.set_status("FDX export is coming soon.");
                                            return Ok(false);
                                        },
                                        _ => ("pdf", "screenplay.pdf"),
                                    };

                                    self.open_file_picker(FilePickerAction::ExportScript, vec![ext.to_string()], Some(default_name.to_string()));
                                }
                                4 => {
                                    let formats = ["csv_scene", "csv_char"];
                                    if let Some(idx) = formats.iter().position(|&x| x == self.config.report_format.as_str()) {
                                        self.config.report_format = formats[(idx + 1) % formats.len()].to_string();
                                    } else {
                                        self.config.report_format = "csv_scene".to_string();
                                    }
                                }
                                5 => {
                                    let (ext, default_name) = match self.config.report_format.as_str() {
                                        "csv_scene" => ("csv", "scene_list.csv"),
                                        "csv_char" => ("csv", "character_report.csv"),
                                        _ => ("csv", "report.csv"),
                                    };

                                    self.open_file_picker(FilePickerAction::ExportReport, vec![ext.to_string()], Some(default_name.to_string()));
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::SceneNavigator => {
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                            self.set_status("Cancelled");
                        }
                        KeyCode::Char(h) if ctrl && h == 'h' => {
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Char('p') if ctrl => {
                            self.mode = AppMode::SettingsPane;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if self.selected_scene > 0 {
                                self.selected_scene -= 1;
                                self.navigator_state.select(Some(self.selected_scene));
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if self.selected_scene + 1 < self.scenes.len() {
                                self.selected_scene += 1;
                                self.navigator_state.select(Some(self.selected_scene));
                            }
                        }
                        KeyCode::Enter => {
                            let line_idx = self.scenes[self.selected_scene].line_idx;
                            self.cursor_y = line_idx;
                            self.cursor_x = 0;
                            self.mode = AppMode::Normal;
                            *cursor_moved = true;
                            *update_target_x = true;
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::CharacterNavigator => {
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let mut next = self.selected_ensemble_idx;
                            while next > 0 {
                                next -= 1;
                                match self.ensemble_items[next] {
                                    EnsembleItem::CharacterHeader(_) | EnsembleItem::SceneLink(..) => {
                                        self.selected_ensemble_idx = next;
                                        self.ensemble_state.select(Some(self.selected_ensemble_idx));
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let mut next = self.selected_ensemble_idx;
                            while next + 1 < self.ensemble_items.len() {
                                next += 1;
                                match self.ensemble_items[next] {
                                    EnsembleItem::CharacterHeader(_) | EnsembleItem::SceneLink(..) => {
                                        self.selected_ensemble_idx = next;
                                        self.ensemble_state.select(Some(self.selected_ensemble_idx));
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        KeyCode::Tab | KeyCode::Char(' ') => {
                            if let EnsembleItem::CharacterHeader(char_idx) = self.ensemble_items[self.selected_ensemble_idx] {
                                self.character_stats[char_idx].is_expanded = !self.character_stats[char_idx].is_expanded;
                                self.refresh_ensemble_list();
                                self.selected_ensemble_idx = self.ensemble_items.iter().position(|item| {
                                    if let EnsembleItem::CharacterHeader(idx) = item {
                                        *idx == char_idx
                                    } else {
                                        false
                                    }
                                }).unwrap_or(0);
                                self.ensemble_state.select(Some(self.selected_ensemble_idx));
                            }
                        }
                        KeyCode::Enter => {
                            match &self.ensemble_items[self.selected_ensemble_idx] {
                                EnsembleItem::CharacterHeader(char_idx) => {
                                    let item = &self.character_stats[*char_idx];
                                    if let Some((_, line_idx)) = item.appears_in_scenes.first() {
                                        self.cursor_y = *line_idx;
                                        self.cursor_x = 0;
                                        self.mode = AppMode::Normal;
                                        *cursor_moved = true;
                                        *update_target_x = true;
                                    }
                                }
                                EnsembleItem::SceneLink(_, line_idx, _) => {
                                    self.cursor_y = *line_idx;
                                    self.cursor_x = 0;
                                    self.mode = AppMode::Normal;
                                    *cursor_moved = true;
                                    *update_target_x = true;
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::SettingsPane => {
                    let settings_count = 6;
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Char('p') if ctrl => {
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Char('f') if ctrl => {}
                        KeyCode::Char('h') if ctrl => {
                            self.open_scene_navigator();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.selected_setting = self.selected_setting.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.selected_setting =
                                (self.selected_setting + 1).min(settings_count - 1);
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            match self.selected_setting {
                                0 => {
                                    self.config.strict_typewriter_mode =
                                        !self.config.strict_typewriter_mode;
                                    let _ = crate::config::Config::save_setting("strict_typewriter_mode", self.config.strict_typewriter_mode);
                                }
                                1 => {
                                    self.config.auto_save = !self.config.auto_save;
                                    let _ = crate::config::Config::save_setting("auto_save", self.config.auto_save);
                                }
                                2 => {
                                    self.config.autocomplete = !self.config.autocomplete;
                                    let _ = crate::config::Config::save_setting("autocomplete", self.config.autocomplete);
                                }
                                3 => {
                                    self.config.auto_paragraph_breaks =
                                        !self.config.auto_paragraph_breaks;
                                    let _ = crate::config::Config::save_setting("auto_paragraph_breaks", self.config.auto_paragraph_breaks);
                                }
                                4 => {
                                    self.config.focus_mode = !self.config.focus_mode;
                                    let _ = crate::config::Config::save_setting("focus_mode", self.config.focus_mode);
                                }
                                5 => {
                                    let themes = self.theme_manager.list_themes();
                                    if let Some(pos) = themes.iter().position(|t| t == &self.config.theme) {
                                        let next = (pos + 1) % themes.len();
                                        let name = &themes[next];
                                        if self.theme_manager.set_theme(name) {
                                            self.theme = self.theme_manager.current_theme.clone();
                                            self.config.theme = self.theme.name.clone();
                                            let _ = crate::config::Config::save_string_setting("theme", &self.theme.name);
                                            self.set_status(&format!("Theme set to {}", self.theme.name));
                                            self.update_layout();
                                        }
                                    } else {
                                        // Fallback if current theme name is not in list for some reason
                                        if !themes.is_empty() {
                                            let name = &themes[0];
                                            if self.theme_manager.set_theme(name) {
                                                self.theme = self.theme_manager.current_theme.clone();
                                                self.config.theme = self.theme.name.clone();
                                                let _ = crate::config::Config::save_string_setting("theme", &self.theme.name);
                                                self.set_status(&format!("Theme set to {}", self.theme.name));
                                                self.update_layout();
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                            *text_changed = true;
                        }
                        KeyCode::Char('?') | KeyCode::Char('h') => {
                            let desc = match self.selected_setting {
                                0 => "Always center the cursor, even at the start of the file.",
                                1 => "Periodically save the current buffer to disk.",
                                2 => "Suggest character names and scene prefixes.",
                                3 => "Insert paragraph breaks after screenplay elements.",
                                4 => "Hide the UI bars for a distraction-free view.",
                                _ => "",
                            };
                            if !desc.is_empty() {
                                self.set_status(desc);
                            }
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::Home => {
                    const HOME_ITEMS: usize = 4;
                    match key.code {
                        KeyCode::Esc => {
                            // If there's an actual file loaded, dismiss home
                            if self.file.is_some() || !self.lines.iter().all(|l| l.is_empty()) {
                                self.mode = AppMode::Normal;
                            }
                        }
                        KeyCode::Char('c') | KeyCode::Char('g') if ctrl => {
                            // Ctrl+C/G always dismisses
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.home_selected = self.home_selected.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.home_selected = (self.home_selected + 1).min(HOME_ITEMS - 1);
                        }
                        KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('\n') |
                        KeyCode::Char('n') | KeyCode::Char('N') |
                        KeyCode::Char('o') | KeyCode::Char('O') |
                        KeyCode::Char('t') | KeyCode::Char('T') |
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            match key.code {
                                KeyCode::Char('n') | KeyCode::Char('N') => self.home_selected = 0,
                                KeyCode::Char('o') | KeyCode::Char('O') => self.home_selected = 1,
                                KeyCode::Char('t') | KeyCode::Char('T') => self.home_selected = 2,
                                KeyCode::Char('q') | KeyCode::Char('Q') => self.home_selected = 3,
                                _ => {},
                            }
                            match self.home_selected {
                                0 => {
                                    // New File
                                    let new_buf = crate::app::BufferState {
                                        lines: vec![String::new()],
                                        ..Default::default()
                                    };
                                    self.buffers.push(new_buf);
                                    let new_idx = self.buffers.len() - 1;
                                    self.has_multiple_buffers = self.buffers.len() > 1;
                                    self.switch_buffer(new_idx);
                                    self.mode = AppMode::Normal;
                                    self.set_status("New buffer");
                                    *text_changed = true;
                                    *cursor_moved = true;
                                }
                                1 => {
                                    // Open File via TUI picker
                                    self.open_file_picker(FilePickerAction::Open, vec!["fountain".to_string()], None);
                                }
                                2 => {
                                    // Tutorial
                                    let tutorial_text = include_str!("../../assets/tutorial.fountain");
                                    let lines: Vec<String> = tutorial_text.lines().map(|s| s.to_string()).collect();
                                    let new_buf = crate::app::BufferState {
                                        lines,
                                        file: None,
                                        is_tutorial: true,
                                        ..Default::default()
                                    };
                                    self.buffers.push(new_buf);
                                    let new_idx = self.buffers.len() - 1;
                                    self.has_multiple_buffers = self.buffers.len() > 1;
                                    self.switch_buffer(new_idx);
                                    self.parse_document();
                                    self.update_autocomplete();
                                    self.update_layout();
                                    self.mode = AppMode::Normal;
                                    self.set_status("Tutorial loaded! Enjoy the show.");
                                    *text_changed = true;
                                    *cursor_moved = true;
                                }
                                3 => {
                                    // Exit App
                                    return Ok(true);
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::FilePicker => {
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                            self.file_picker = None;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if let Some(ref mut state) = self.file_picker {
                                let current = state.list_state.selected().unwrap_or(0);
                                if current > 0 {
                                    state.list_state.select(Some(current - 1));
                                }
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if let Some(ref mut state) = self.file_picker {
                                let current = state.list_state.selected().unwrap_or(0);
                                let max = state.items.len() + (if state.action != FilePickerAction::Open && !state.filename_input.is_empty() { 1 } else { 0 });
                                if current + 1 < max {
                                    state.list_state.select(Some(current + 1));
                                }
                            }
                        }
                        KeyCode::Enter => {
                            if self.file_picker_enter().map_err(|e| io::Error::other(e.to_string()))? {
                                return Ok(true);
                            }
                        }
                        KeyCode::Backspace => {
                            if let Some(ref mut state) = self.file_picker {
                                if state.action != FilePickerAction::Open {
                                    state.filename_input.pop();
                                } else {
                                    // Navigate up directory
                                    if let Some(parent) = state.current_dir.parent().map(|p| p.to_path_buf()) {
                                        state.current_dir = parent;
                                        state.items = crate::app::file_picker::get_dir_items(&state.current_dir);
                                        state.list_state.select(Some(0));
                                    }
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            if let Some(ref mut state) = self.file_picker
                                && state.action != FilePickerAction::Open {
                                    state.filename_input.push(c);
                                }
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::Snapshots => {
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let current = self.snapshot_list_state.selected().unwrap_or(0);
                            if current > 0 {
                                self.snapshot_list_state.select(Some(current - 1));
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let current = self.snapshot_list_state.selected().unwrap_or(0);
                            if current + 1 < self.snapshots.len() {
                                self.snapshot_list_state.select(Some(current + 1));
                            }
                        }
                        KeyCode::Enter => {
                            let selected = self.snapshot_list_state.selected().unwrap_or(0);
                            self.restore_snapshot(selected)?;
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::SprintStat => {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => self.mode = AppMode::Normal,
                        KeyCode::Up | KeyCode::Char('k') => {
                            let current = self.sprint_stats_state.selected().unwrap_or(0);
                            if current > 0 {
                                self.sprint_stats_state.select(Some(current - 1));
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let current = self.sprint_stats_state.selected().unwrap_or(0);
                            if current + 1 < self.sprint_history.len() {
                                self.sprint_stats_state.select(Some(current + 1));
                            }
                        }
                        KeyCode::Char('e') => self.export_sprint_data(),
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::Command => {
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                            self.command_input.clear();
                            self.command_error = false;
                        }
                        KeyCode::Tab => {
                            let commands = vec![
                                "w", "ww", "q", "q!", "wq", "ex",
                                "renum", "clearnum", "locknum", "unlocknum", "injectnum",
                                "set", "search", "export",
                                "ud", "rd", "copy", "cut", "paste", "pos",
                                "selectall", "home", "o", "bn", "bp", "new", "newfile", "addtitle",
                                "snap", "sprint", "cancelsprint", "sprintstat",
                            ];
                            let matches: Vec<&&str> = commands.iter()
                                .filter(|c| c.starts_with(&self.command_input))
                                .collect();
                            
                            if !matches.is_empty() {
                                // Basic cycling
                                let current = self.command_input.as_str();
                                if let Some(pos) = matches.iter().position(|m| **m == current) {
                                    self.command_input = matches[(pos + 1) % matches.len()].to_string();
                                } else {
                                    self.command_input = matches[0].to_string();
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            self.command_input.pop();
                            if self.command_input.is_empty() {
                                self.mode = AppMode::Normal;
                            }
                            self.command_error = false;
                        }
                        KeyCode::Enter => {
                            if self.execute_command(text_changed, cursor_moved, update_target_x)? {
                                return Ok(true);
                            }
                        }
                        KeyCode::Char(c) => {
                            self.command_input.push(c);
                            self.command_error = false;
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::Shortcuts => {
                    match key.code {
                        KeyCode::Esc | KeyCode::F(1) => {
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let i = self.shortcuts_state.selected().unwrap_or(0);
                            self.shortcuts_state.select(Some(i.saturating_sub(1)));
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let i = self.shortcuts_state.selected().unwrap_or(0);
                            self.shortcuts_state.select(Some(i.saturating_add(1)));
                        }
                        KeyCode::PageUp => {
                            let i = self.shortcuts_state.selected().unwrap_or(0);
                            self.shortcuts_state.select(Some(i.saturating_sub(10)));
                        }
                        KeyCode::PageDown => {
                            let i = self.shortcuts_state.selected().unwrap_or(0);
                            self.shortcuts_state.select(Some(i.saturating_add(10)));
                        }
                        KeyCode::Home => {
                            self.shortcuts_state.select(Some(0));
                        }
                        KeyCode::Char('h') if ctrl => {
                            self.open_scene_navigator();
                        }
                        KeyCode::Char('p') if ctrl => {
                            self.mode = AppMode::SettingsPane;
                            self.selected_setting = 0;
                        }
                        KeyCode::Char('f') if ctrl => {}
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::Normal => {
                    self.clear_status();

                    if self.show_search_highlight {
                        match key.code {
                            KeyCode::Char('w') if ctrl => {}
                            KeyCode::Char('c') if ctrl => {}
                            _ => {
                                self.show_search_highlight = false;
                                *text_changed = true;
                            }
                        }
                    }

                    match key.code {
                        KeyCode::Esc => {}


                        KeyCode::Left if ctrl => {
                            self.move_word_left();
                            *update_target_x = true;
                            *cursor_moved = true;
                        }
                        KeyCode::Right if ctrl => {
                            self.move_word_right();
                            *update_target_x = true;
                            *cursor_moved = true;
                        }

                        KeyCode::Backspace if ctrl => {
                            self.delete_word_back();
                            *update_target_x = true;
                            *text_changed = true;
                            *cursor_moved = true;
                        }
                        KeyCode::Delete if ctrl => {
                            self.delete_word_forward();
                            *update_target_x = true;
                            *text_changed = true;
                            *cursor_moved = true;
                        }

                        KeyCode::Char('h') if ctrl => {
                            self.open_scene_navigator();
                        }
                        KeyCode::Char('l') if ctrl => {
                            self.open_character_sidebar();
                        }
                        KeyCode::Char('p') if ctrl => {
                            self.mode = AppMode::SettingsPane;
                            self.selected_setting = 0;
                        }
                        KeyCode::Char('f') if ctrl => {}
                        KeyCode::Char('/') => {
                            self.mode = AppMode::Command;
                            self.command_input.clear();
                            self.command_error = false;
                        }
                        KeyCode::Char('e') if ctrl => {
                            self.mode = AppMode::ExportPane;
                            self.selected_export_option = 0;
                        }
                        KeyCode::Char('i') if ctrl && shift => {}

                        KeyCode::Char('a') if ctrl => {
                            self.select_all();
                            *cursor_moved = true;
                        }
                        KeyCode::Char('c') if ctrl => {
                            self.copy_to_clipboard();
                        }
                        KeyCode::Char('x') if ctrl => {
                            if self.selection_anchor.is_some() {
                                self.cut_to_clipboard();
                            } else {
                                self.cut_line();
                                self.set_status("Line cut");
                            }
                            *update_target_x = true;
                            *text_changed = true;
                            *cursor_moved = true;
                        }
                        KeyCode::Char('v') if ctrl => {
                            self.paste_from_clipboard();
                            *update_target_x = true;
                            *text_changed = true;
                            *cursor_moved = true;
                        }
                        KeyCode::F(1) => {
                            self.mode = AppMode::Shortcuts;
                        }
                        KeyCode::Up if shift => {
                            if self.selection_anchor.is_none() {
                                self.selection_anchor = Some((self.cursor_y, self.cursor_x));
                            }
                            self.move_up();
                            *cursor_moved = true;
                        }
                        KeyCode::Down if shift => {
                            if self.selection_anchor.is_none() {
                                self.selection_anchor = Some((self.cursor_y, self.cursor_x));
                            }
                            self.move_down();
                            *cursor_moved = true;
                        }
                        KeyCode::Left if shift => {
                            if self.selection_anchor.is_none() {
                                self.selection_anchor = Some((self.cursor_y, self.cursor_x));
                            }
                            self.move_left();
                            *update_target_x = true;
                            *cursor_moved = true;
                        }
                        KeyCode::Right if shift => {
                            if self.selection_anchor.is_none() {
                                self.selection_anchor = Some((self.cursor_y, self.cursor_x));
                            }
                            self.move_right();
                            *update_target_x = true;
                            *cursor_moved = true;
                        }
                        KeyCode::Home if shift => {
                            if self.selection_anchor.is_none() {
                                self.selection_anchor = Some((self.cursor_y, self.cursor_x));
                            }
                            self.move_home();
                            *update_target_x = true;
                            *cursor_moved = true;
                        }
                        KeyCode::End if shift => {
                            if self.selection_anchor.is_none() {
                                self.selection_anchor = Some((self.cursor_y, self.cursor_x));
                            }
                            self.move_end();
                            *update_target_x = true;
                            *cursor_moved = true;
                        }
                        KeyCode::Up => {
                            self.clear_selection();
                            self.move_up();
                            *cursor_moved = true;
                        }
                        KeyCode::Down => {
                            self.clear_selection();
                            self.move_down();
                            *cursor_moved = true;
                        }
                        KeyCode::Left => {
                            self.clear_selection();
                            self.move_left();
                            *update_target_x = true;
                            *cursor_moved = true;
                        }
                        KeyCode::Right => {
                            self.clear_selection();
                            self.move_right();
                            *update_target_x = true;
                            *cursor_moved = true;
                        }
                        KeyCode::PageUp => {
                            self.move_page_up();
                            *cursor_moved = true;
                        }
                        KeyCode::PageDown => {
                            self.move_page_down();
                            *cursor_moved = true;
                        }
                        KeyCode::Home => {
                            self.move_home();
                            *update_target_x = true;
                            *cursor_moved = true;
                        }
                        KeyCode::End => {
                            self.move_end();
                            *update_target_x = true;
                            *cursor_moved = true;
                        }

                        KeyCode::Enter => {
                            self.suggestion = None;
                            self.insert_newline(shift);
                            *update_target_x = true;
                            *text_changed = true;
                            *cursor_moved = true;
                        }
                        KeyCode::Backspace => {
                            if self.selection_anchor.is_some() {
                                self.delete_selection();
                                self.parse_document();
                            } else {
                                self.backspace();
                            }
                            *update_target_x = true;
                            *text_changed = true;
                            *cursor_moved = true;
                        }
                        KeyCode::Delete => {
                            if self.selection_anchor.is_some() {
                                self.delete_selection();
                                self.parse_document();
                            } else {
                                self.delete_forward();
                            }
                            *update_target_x = true;
                            *text_changed = true;
                            *cursor_moved = true;
                        }
                        KeyCode::Tab => {
                            self.handle_tab();
                            *update_target_x = true;
                            *text_changed = true;
                            *cursor_moved = true;
                        }
                        KeyCode::Char(c) if !ctrl => {
                            if self.selection_anchor.is_some() {
                                self.delete_selection();
                                self.parse_document();
                            }
                            self.insert_char(c);
                            *update_target_x = true;
                            *text_changed = true;
                            *cursor_moved = true;
                        }
                        _ => {}
                    }
                }
            }
        }
        
        Ok(false)
    }
}
