use std::path::PathBuf;
use crate::app::{App, AppMode, FilePickerAction};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io;
use std::fs;

impl App {
    pub fn handle_panes(&mut self, key: KeyEvent, update_target_x: &mut bool, text_changed: &mut bool, cursor_moved: &mut bool) -> io::Result<bool> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let _shift = key.modifiers.contains(KeyModifiers::SHIFT);
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
                AppMode::ExportPane => {
                    let options_count = 7;
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
                                    if self.config.mirror_scene_numbers == crate::config::MirrorOption::Off {
                                        self.config.mirror_scene_numbers = crate::config::MirrorOption::ExportOnly;
                                        let _ = crate::config::Config::save_string_setting("mirror_scene_numbers", "export");
                                    } else {
                                        self.config.mirror_scene_numbers = crate::config::MirrorOption::Off;
                                        let _ = crate::config::Config::save_string_setting("mirror_scene_numbers", "off");
                                    }
                                }
                                4 => {
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
                                5 => {
                                    let formats = ["csv_scene", "csv_char"];
                                    if let Some(idx) = formats.iter().position(|&x| x == self.config.report_format.as_str()) {
                                        self.config.report_format = formats[(idx + 1) % formats.len()].to_string();
                                    } else {
                                        self.config.report_format = "csv_scene".to_string();
                                    }
                                }
                                6 => {
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
                AppMode::Home => {
                    let home_items = 4 + self.recent_files.len().min(5);
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
                            if self.home_selected > 0 {
                                self.home_selected -= 1;
                            } else {
                                self.home_selected = home_items.saturating_sub(1);
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.home_selected = (self.home_selected + 1) % home_items;
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
                                    let tutorial_text = include_str!("../../../assets/tutorial.fountain");
                                    let lines: Vec<String> = tutorial_text.lines().map(|s: &str| s.to_string()).collect();
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
                                _ => {
                                    // Recent Files
                                    let recent_idx = self.home_selected - 4;
                                    if recent_idx < self.recent_files.len() {
                                        let path = self.recent_files[recent_idx].clone();
                                        if let Ok(content) = fs::read_to_string(&path) {
                                            let lines: Vec<String> = content.replace('\t', "    ")
                                                .lines()
                                                .map(|s| s.to_string())
                                                .collect();
                                            let new_buf = crate::app::BufferState {
                                                lines: if lines.is_empty() { vec![String::new()] } else { lines },
                                                file: Some(path.clone()),
                                                ..Default::default()
                                            };
                                            self.buffers.push(new_buf);
                                            let new_idx = self.buffers.len() - 1;
                                            self.has_multiple_buffers = self.buffers.len() > 1;
                                            self.switch_buffer(new_idx);
                                            self.add_recent_file(path.clone());
                                            self.mode = AppMode::Normal;
                                            self.parse_document();
                                            self.update_autocomplete();
                                            self.update_layout();
                                            let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                                            self.set_status(&format!("Opened: {}", name));
                                            *text_changed = true;
                                            *cursor_moved = true;
                                        } else {
                                            self.set_status("Error opening recent file");
                                            self.recent_files.remove(recent_idx);
                                            self.save_recent_files();
                                        }
                                    }
                                }
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
                        KeyCode::Enter | KeyCode::Char('r') => {
                            let selected = self.snapshot_list_state.selected().unwrap_or(0);
                            self.restore_snapshot(selected, false)?;
                        }
                        KeyCode::Char('o') => {
                            let selected = self.snapshot_list_state.selected().unwrap_or(0);
                            self.restore_snapshot(selected, true)?;
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
            _ => {}
        }
        Ok(false)
    }
}
