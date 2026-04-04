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
                    let settings_count = 9;
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = AppMode::Normal;
                        }
                        KeyCode::Char('p') if ctrl => {
                            self.mode = AppMode::Normal;
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
                                2 => {
                                    self.config.show_scene_numbers = !self.config.show_scene_numbers
                                }
                                3 => self.config.show_page_numbers = !self.config.show_page_numbers,
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
                        KeyCode::Char('?') | KeyCode::Char('h') => {
                            let desc = match self.selected_setting {
                                0 => "Always center the cursor, even at the start of the file.",
                                1 => "Periodically save the current buffer to disk.",
                                2 => "Display scene numbers in the left margin.",
                                3 => "Display page numbers in the right margin.",
                                4 => "Hide Fountain markup (headings, blocks) except for active line.",
                                5 => "Suggest character names and scene prefixes.",
                                6 => "Automatically append (CONT'D) to character names.",
                                7 => "Insert paragraph breaks after screenplay elements.",
                                8 => "Hide the UI bars for a distraction-free view.",
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
                        KeyCode::Char('h') if ctrl => {
                            self.open_scene_navigator();
                        }
                        KeyCode::Char('p') if ctrl => {
                            self.mode = AppMode::SettingsPane;
                            self.selected_setting = 0;
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
