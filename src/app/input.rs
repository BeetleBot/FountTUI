use std::io;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use std::path::PathBuf;
use crate::app::{App, AppMode};

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

                                    if let Some(path) = rfd::FileDialog::new()
                                        .set_file_name(default_name)
                                        .add_filter(ext, &[ext][..])
                                        .save_file()
                                    {
                                        let fountain_text = self.lines.join("\n");
                                        let result = match self.config.export_format.as_str() {
                                            "fountain" => self.export_fountain(&path),
                                            _ => {
                                                let paper_size = if self.config.paper_size.to_lowercase() == "letter" {
                                                    crate::pdf::LETTER
                                                } else {
                                                    crate::pdf::A4
                                                };
                                                crate::pdf::export_to_pdf(&fountain_text, &path, paper_size, self.config.export_bold_scene_headings)
                                            }
                                        };

                                        match result {
                                            Ok(_) => self.set_status(&format!("Exported to {}", path.display())),
                                            Err(e) => self.set_status(&format!("Error exporting: {}", e)),
                                        }
                                        self.mode = AppMode::Normal;
                                    } else {
                                        self.set_status("Export cancelled.");
                                    }
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

                                    if let Some(path) = rfd::FileDialog::new()
                                        .set_file_name(default_name)
                                        .add_filter(ext, &[ext][..])
                                        .save_file()
                                    {
                                        let result = match self.config.report_format.as_str() {
                                            "csv_char" => self.export_character_csv(&path),
                                            _ => self.export_scene_csv(&path),
                                        };
                                        match result {
                                            Ok(_) => self.set_status(&format!("Exported to {}", path.display())),
                                            Err(e) => self.set_status(&format!("Error exporting: {}", e)),
                                        }
                                        self.mode = AppMode::Normal;
                                    } else {
                                        self.set_status("Export cancelled.");
                                    }
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
                        KeyCode::Enter | KeyCode::Char(' ') => {
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
                                    // Open File via native picker
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Fountain scripts", &["fountain"][..])
                                        .pick_file()
                                    {
                                        let content = std::fs::read_to_string(&path)
                                            .unwrap_or_default()
                                            .replace('\t', "    ");
                                        let lines: Vec<String> = if content.trim().is_empty() {
                                            vec![String::new()]
                                        } else {
                                            content.lines().map(str::to_string).collect()
                                        };
                                        let new_buf = crate::app::BufferState {
                                            lines,
                                            file: Some(path.clone()),
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
                                        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                                        self.set_status(&format!("Opened: {}", name));
                                        *text_changed = true;
                                        *cursor_moved = true;
                                    }
                                }
                                2 => {
                                    // Tutorial — placeholder
                                    self.mode = AppMode::Normal;
                                    self.set_status("Tutorial coming soon!");
                                }
                                3 => {
                                    // Exit
                                    return Ok(true);
                                }
                                _ => {}
                            }
                        }
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
                                "w", "q", "q!", "wq",
                                "renum", "clearnum", "locknum", "unlocknum",
                                "set", "search",
                                "u", "undo", "redo", "copy", "cut", "paste", "pos",
                                "injectnum", "selectall", "s",
                                "home", "new",
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
                            self.execute_command(text_changed, cursor_moved, update_target_x)?;
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
                            // Use :w to save
                        }
                        KeyCode::Char('h') if ctrl => {
                            self.open_scene_navigator();
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
                        KeyCode::Char('z') if ctrl => {}
                        KeyCode::Char('r') if ctrl => {}
                        KeyCode::Char('e') if ctrl => {
                            self.mode = AppMode::ExportPane;
                            self.selected_export_option = 0;
                        }
                        KeyCode::Char('k') if ctrl => {}
                        KeyCode::Char('u') if ctrl => {}
                        KeyCode::Char('w') if ctrl => {}
                        KeyCode::Char('c') if ctrl => {}
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
