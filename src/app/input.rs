use std::io;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use std::path::PathBuf;
use crate::app::{App, AppMode};

impl App {

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
                        self.move_down();
                        *cursor_moved = true;
                    }
                }
                MouseEventKind::Down(MouseButton::Left) => {
                    if self.mode == AppMode::SettingsPane {
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
                            let mut current_y = self.sidebar_area.y as usize;
                            let offset = self.navigator_state.offset();
                            for i in offset..self.scenes.len() {
                                let h = self.calculate_scene_height(&self.scenes[i].1, &self.scenes[i].3);
                                if (y as usize) < current_y + h {
                                    self.selected_scene = i;
                                    self.navigator_state.select(Some(i));

                                    let line_idx = self.scenes[i].0;
                                    self.cursor_y = line_idx;
                                    self.cursor_x = 0;
                                    *cursor_moved = true;
                                    *update_target_x = true;
                                    break;
                                }
                                current_y += h;
                                if current_y >= (self.sidebar_area.y + self.sidebar_area.height) as usize
                                {
                                    break;
                                }
                            }
                        }
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
                                if self.exit_after_save && self.close_current_buffer() {
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
                            if self.exit_after_save && self.close_current_buffer() {
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
                                        if self.exit_after_save && self.close_current_buffer() {
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
                    let options_count = 4;
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
                                    if self.config.export_format == "pdf" {
                                        self.config.export_format = "html".to_string();
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
                                    self.filename_input = self
                                        .file
                                        .as_ref()
                                        .map(|p| p.with_extension(&self.config.export_format).to_string_lossy().into_owned())
                                        .unwrap_or_else(|| format!("screenplay.{}", self.config.export_format));
                                    self.mode = AppMode::PromptExportFilename;
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::PromptExportFilename => {
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::ExportPane;
                            self.set_status("Cancelled");
                        }
                        KeyCode::Char('c') | KeyCode::Char('g') if ctrl => {
                            self.mode = AppMode::Normal;
                            self.set_status("Cancelled");
                        }
                        KeyCode::Enter => {
                            if !self.filename_input.trim().is_empty() {
                                let export_path = std::path::PathBuf::from(self.filename_input.trim());
                                let fountain_text = self.lines.join("\n");
                                
                                let result = if self.config.export_format == "html" {
                                    crate::pdf::export_to_html(&fountain_text, &export_path)
                                } else {
                                    let paper_size = if self.config.paper_size.to_lowercase() == "letter" {
                                        crate::pdf::LETTER
                                    } else {
                                        crate::pdf::A4
                                    };
                                    crate::pdf::export_to_pdf(&fountain_text, &export_path, paper_size, self.config.export_bold_scene_headings)
                                };

                                match result {
                                    Ok(_) => {
                                        self.set_status(&format!("Exported to {}", export_path.display()));
                                        self.mode = AppMode::Normal;
                                    }
                                    Err(e) => {
                                        self.set_status(&format!("Error exporting: {}", e));
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
                            self.selected_setting = 0;
                        }
                        KeyCode::Char('c') | KeyCode::Char('g') if ctrl => {
                            self.mode = AppMode::Normal;
                            self.set_status("Cancelled");
                        }
                        KeyCode::Up => {
                            if self.selected_scene > 0 {
                                self.selected_scene -= 1;
                                self.navigator_state.select(Some(self.selected_scene));
                            }
                        }
                        KeyCode::Down => {
                            if self.selected_scene < self.scenes.len() - 1 {
                                self.selected_scene += 1;
                                self.navigator_state.select(Some(self.selected_scene));
                            }
                        }
                        KeyCode::Enter => {
                            let line_idx = self.scenes[self.selected_scene].0;
                            self.cursor_y = line_idx;
                            self.cursor_x = 0;
                            *cursor_moved = true;
                            *update_target_x = true;
                        }
                        _ => {}
                    }
                    return Ok(false);
                }
                AppMode::SettingsPane => {
                    let settings_count = 5;
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Char('p') if ctrl => {
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Char('f') if ctrl => {
                            self.mode = AppMode::FormatPane;
                            self.selected_format_option = 0;
                        }
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
                                        !self.config.strict_typewriter_mode
                                }
                                1 => self.config.auto_save = !self.config.auto_save,
                                2 => self.config.autocomplete = !self.config.autocomplete,
                                3 => {
                                    self.config.auto_paragraph_breaks =
                                        !self.config.auto_paragraph_breaks
                                }
                                4 => self.config.focus_mode = !self.config.focus_mode,
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
                AppMode::FormatPane => {
                    // Navigable count = 7 (headers are skipped)
                    // nav idx -> action : 0=PageNums, 1=HideMarkup, 2=AutoContd,
                    //                     3=ProdLock, 4=Renumber, 5=ClearAll, 6=ShowSceneNums
                    const NAV_COUNT: usize = 7;
                    // Header positions in the rendered list: render_idx 0 and 4
                    // nav_idx maps to render_idx: +1 for idx>=0 (skip header at 0),
                    //                             +1 again for idx>=3 (skip header at render 4)
                    fn next_nav(cur: usize, dir: isize) -> usize {
                        let next = (cur as isize + dir).clamp(0, (NAV_COUNT - 1) as isize) as usize;
                        next
                    }
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Char('f') if ctrl => {
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Char('p') if ctrl => {
                            self.mode = AppMode::SettingsPane;
                            self.selected_setting = 0;
                        }
                        KeyCode::Char('h') if ctrl => {
                            self.open_scene_navigator();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.selected_format_option = next_nav(self.selected_format_option, -1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.selected_format_option = next_nav(self.selected_format_option, 1);
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            match self.selected_format_option {
                                0 => self.config.show_page_numbers = !self.config.show_page_numbers,
                                1 => self.config.hide_markup = !self.config.hide_markup,
                                2 => self.config.auto_contd = !self.config.auto_contd,
                                3 => {
                                    self.config.production_lock = !self.config.production_lock;
                                    let state = if self.config.production_lock { "ON" } else { "OFF" };
                                    self.set_status(&format!("Production Lock {}", state));
                                }
                                4 => {
                                    self.renumber_all_scenes();
                                    self.set_status("All scenes renumbered");
                                }
                                5 => {
                                    self.strip_all_scene_numbers();
                                }
                                6 => self.config.show_scene_numbers = !self.config.show_scene_numbers,
                                _ => {}
                            }
                            *text_changed = true;
                        }
                        KeyCode::Char('?') => {
                            let desc = match self.selected_format_option {
                                0 => "Display page numbers in the right margin.",
                                1 => "Hide Fountain markup except on the active line.",
                                2 => "Automatically append (CONT'D) to repeated characters.",
                                3 => "Lock: prevents automatic scene number changes while editing.",
                                4 => "Number all scenes chronologically (respects custom tags like 14B).",
                                5 => "Remove ALL scene number tags from the entire script.",
                                6 => "Display scene numbers in the left margin.",
                                _ => "",
                            };
                            if !desc.is_empty() { self.set_status(desc); }
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
                        KeyCode::Char('h') if ctrl => {
                            self.open_scene_navigator();
                        }
                        KeyCode::Char('p') if ctrl => {
                            self.mode = AppMode::SettingsPane;
                            self.selected_setting = 0;
                        }
                        KeyCode::Char('f') if ctrl => {
                            self.mode = AppMode::FormatPane;
                            self.selected_format_option = 0;
                        }
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
                        KeyCode::Char('x') if ctrl => {
                            if self.dirty {
                                self.exit_after_save = true;
                                self.mode = AppMode::PromptSave;
                            } else if self.close_current_buffer() {
                                return Ok(true);
                            }
                        }

                        KeyCode::Char('>') | KeyCode::Char('.') if ctrl => {
                            self.switch_next_buffer();
                            *update_target_x = true;
                            *text_changed = true;
                            *cursor_moved = true;
                        }
                        KeyCode::Char('<') | KeyCode::Char(',') if ctrl => {
                            self.switch_prev_buffer();
                            *update_target_x = true;
                            *text_changed = true;
                            *cursor_moved = true;
                        }

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

                        KeyCode::Char('s') if ctrl => {
                            if self.file.is_some() {
                                self.save()?;
                            } else {
                                self.filename_input.clear();
                                self.mode = AppMode::PromptFilename;
                                self.exit_after_save = false;
                            }
                        }
                        KeyCode::Char('h') if ctrl => {
                            self.open_scene_navigator();
                        }
                        KeyCode::Char('p') if ctrl => {
                            self.mode = AppMode::SettingsPane;
                            self.selected_setting = 0;
                        }
                        KeyCode::Char('f') if ctrl => {
                            self.mode = AppMode::FormatPane;
                            self.selected_format_option = 0;
                        }
                        KeyCode::Char('z') if ctrl => {
                            if self.undo() {
                                self.set_status("Undo applied");
                                *update_target_x = true;
                                *text_changed = true;
                                *cursor_moved = true;
                            } else {
                                self.set_status("Nothing to undo");
                            }
                        }
                        KeyCode::Char('r') if ctrl => {
                            if self.redo() {
                                self.set_status("Redo applied");
                                *update_target_x = true;
                                *text_changed = true;
                                *cursor_moved = true;
                            } else {
                                self.set_status("Nothing to redo");
                            }
                        }
                        KeyCode::Char('e') if ctrl => {
                            self.mode = AppMode::ExportPane;
                            self.selected_export_option = 0;
                        }
                        KeyCode::Char('k') if ctrl => {
                            self.cut_line();
                            *update_target_x = true;
                            *text_changed = true;
                            *cursor_moved = true;
                        }
                        KeyCode::Char('u') if ctrl => {
                            self.paste_line();
                            *update_target_x = true;
                            *text_changed = true;
                            *cursor_moved = true;
                        }
                        KeyCode::Char('w') if ctrl => {
                            self.mode = AppMode::Search;
                            self.search_query.clear();
                            self.show_search_highlight = true;
                            self.update_search_regex();
                        }
                        KeyCode::Char('c') if ctrl => {
                            self.report_cursor_position();
                        }
                        KeyCode::Char('i') if ctrl && shift => {
                            self.inject_current_scene_number();
                            *text_changed = true;
                        }

                        KeyCode::F(1) => {
                            self.mode = AppMode::Shortcuts;
                        }
                        KeyCode::Up => {
                            self.move_up();
                            *cursor_moved = true;
                        }
                        KeyCode::Down => {
                            self.move_down();
                            *cursor_moved = true;
                        }
                        KeyCode::Left => {
                            self.move_left();
                            *update_target_x = true;
                            *cursor_moved = true;
                        }
                        KeyCode::Right => {
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
                            self.backspace();
                            *update_target_x = true;
                            *text_changed = true;
                            *cursor_moved = true;
                        }
                        KeyCode::Delete => {
                            self.delete_forward();
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
