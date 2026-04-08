use std::{fs, io};

use crate::app::{App, AppMode, LastEdit, NavigatorItem};
use crate::formatting::StringCaseExt;
use crate::layout::find_visual_cursor;
use crate::types::LineType;


impl App {


    pub fn current_visual_x(&self) -> u16 {
        let (_, vis_x) = find_visual_cursor(&self.layout, self.cursor_y, self.cursor_x);
        vis_x
    }

    pub fn update_autocomplete(&mut self) {
        let pending_tab_suggestion = self.suggestion.take();
        let mut matched = false;

        if !self.config.autocomplete {
            return;
        }

        if self.cursor_y >= self.lines.len() {
            return;
        }

        let line = &self.lines[self.cursor_y];
        let char_count = line.chars().count();

        if self.cursor_x != char_count || char_count == 0 {
            return;
        }

        let upper_line = line.to_uppercase_1to1();

        if let Some(sug) = pending_tab_suggestion {
            let upper_trim = upper_line.trim_start();
            let full_text = format!("{}{}", upper_trim, sug);

            if self.characters.contains(&full_text) || self.locations.contains(&full_text) {
                self.suggestion = Some(sug);
                if self.characters.contains(&full_text) {
                    self.types[self.cursor_y] = LineType::Character;
                } else if self.locations.contains(&full_text) {
                    self.types[self.cursor_y] = LineType::SceneHeading;
                }
                return;
            }
        }

        let is_char_type = matches!(
            self.types.get(self.cursor_y),
            Some(LineType::Character) | Some(LineType::DualDialogueCharacter)
        );

        if is_char_type || upper_line.starts_with('@') {
            let input = upper_line.trim_start_matches('@').trim_start();
            if !input.is_empty() {
                let best_match = self
                    .characters
                    .iter()
                    .filter(|c| c.starts_with(input) && c.len() > input.len())
                    .min_by_key(|c| c.len());
                if let Some(c) = best_match {
                    self.suggestion = Some(c[input.len()..].to_string());
                    return;
                }
            }
        }

        let is_scene_type = self.types.get(self.cursor_y) == Some(&LineType::SceneHeading);

        if is_scene_type || upper_line.starts_with('.') {
            let mut input = upper_line.trim_start();

            if input.starts_with('.') && !input.starts_with("..") {
                input = &input[1..];
            } else {
                let prefixes = [
                    "INT. ",
                    "EXT. ",
                    "EST. ",
                    "INT/EXT. ",
                    "I/E. ",
                    "E/I. ",
                    "I./E. ",
                    "E./I. ",
                    "INT ",
                    "EXT ",
                    "EST ",
                    "INT/EXT ",
                    "I/E ",
                    "E/I ",
                ];
                for p in prefixes {
                    if let Some(rest) = input.strip_prefix(p) {
                        input = rest;
                        matched = true;
                        break;
                    }
                }
                if !matched && let Some((_, rest)) = input.split_once(". ") {
                    input = rest;
                }
            }

            input = input.trim_start();

            if !input.is_empty() {
                let mut best_match: Option<&String> = None;
                for loc in &self.locations {
                    if loc.starts_with(input)
                        && loc.len() > input.len()
                        && (best_match.is_none() || loc.len() < best_match.unwrap().len())
                    {
                        best_match = Some(loc);
                    }
                }
                if let Some(loc) = best_match {
                    self.suggestion = Some(loc[input.len()..].to_string());
                }
            }
        }
    }

    pub fn save(&mut self) -> io::Result<()> {
        if self.is_tutorial {
            self.set_status("Cannot save the tutorial buffer. Press Ctrl+X to exit.");
            return Ok(());
        }
        if let Some(ref p) = self.file {
            let mut content = self.lines.join("\n");
            if !content.ends_with('\n') {
                content.push('\n');
            }
            fs::write(p, content)?;
            self.dirty = false;
            self.set_status(&format!("Wrote {} lines", self.lines.len()));

            // Trigger snapshot on manual save
            self.trigger_snapshot();
        }
        Ok(())
    }

    pub fn trigger_snapshot(&mut self) {
        if let Some(ref p) = self.file {
            if let Err(e) = self.snapshot_manager.create_snapshot(p, &self.lines) {
                self.set_status(&format!("Snapshot failed: {}", e));
            } else {
                self.last_snapshot_time = Some(std::time::Instant::now());
            }
        }
    }

    pub fn restore_snapshot(&mut self, index: usize, in_new_buffer: bool) -> io::Result<()> {
        if index >= self.snapshots.len() {
            return Ok(());
        }

        let snapshot_path = self.snapshots[index].path.clone();
        let snapshot_display_time = self.snapshots[index].display_time();
        let content = fs::read_to_string(&snapshot_path)?;

        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        if lines.is_empty() {
            lines = vec![String::new()];
        }

        let buf_name = if let Some(ref p) = self.file {
            p.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "unnamed".to_string())
        } else {
            "unnamed".to_string()
        };

        if in_new_buffer {
            let new_buf = crate::app::BufferState {
                lines,
                dirty: true,
                ..Default::default()
            };
            self.buffers.push(new_buf);
            let new_idx = self.buffers.len() - 1;
            self.has_multiple_buffers = true;
            self.switch_buffer(new_idx);
            self.set_status(&format!(
                "Opened snapshot of {} from {} in a new buffer",
                buf_name, snapshot_display_time
            ));
        } else {
            self.save_state(true); // Save current for undo
            self.lines = lines;
            self.cursor_y = 0;
            self.cursor_x = 0;
            self.dirty = true;
            self.parse_document();
            self.update_autocomplete();
            self.update_layout();
            self.set_status(&format!(
                "Replaced current buffer with snapshot from {}",
                snapshot_display_time
            ));
        }

        self.mode = AppMode::Normal;
        Ok(())
    }

    pub fn line_len(&self, y: usize) -> usize {
        self.lines.get(y).map(|l| l.chars().count()).unwrap_or(0)
    }

    pub fn move_up(&mut self) {
        self.last_edit = LastEdit::Other;
        let (vis_row, _) = find_visual_cursor(&self.layout, self.cursor_y, self.cursor_x);
        if vis_row > 0 {
            let mut target_vi = vis_row - 1;
            while target_vi > 0 && self.layout[target_vi].is_phantom {
                target_vi -= 1;
            }
            self.jump_to_visual_row(target_vi, Some(false));
        } else {
            self.cursor_y = 0;
            self.cursor_x = 0;
        }
    }

    pub fn move_down(&mut self) {
        self.last_edit = LastEdit::Other;
        let (vis_row, _) = find_visual_cursor(&self.layout, self.cursor_y, self.cursor_x);
        if vis_row + 1 < self.layout.len() {
            let mut target_vi = vis_row + 1;
            while target_vi + 1 < self.layout.len() && self.layout[target_vi].is_phantom {
                target_vi += 1;
            }
            self.jump_to_visual_row(target_vi, Some(true));
        } else {
            self.cursor_y = self.lines.len().saturating_sub(1);
            self.cursor_x = self.line_len(self.cursor_y);
        }
    }

    pub fn move_left(&mut self) {
        self.last_edit = LastEdit::Other;
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.line_len(self.cursor_y);
        }
    }

    pub fn move_right(&mut self) {
        self.last_edit = LastEdit::Other;
        let max = self.line_len(self.cursor_y);
        if self.cursor_x < max {
            self.cursor_x += 1;
        } else if self.cursor_y + 1 < self.lines.len() {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }

    pub fn move_word_left(&mut self) {
        self.last_edit = LastEdit::Other;
        if self.cursor_x == 0 {
            self.move_left();
            return;
        }
        let chars: Vec<char> = self.lines[self.cursor_y].chars().collect();
        while self.cursor_x > 0 && chars[self.cursor_x - 1].is_whitespace() {
            self.cursor_x -= 1;
        }
        while self.cursor_x > 0 && !chars[self.cursor_x - 1].is_whitespace() {
            self.cursor_x -= 1;
        }
    }

    pub fn move_word_right(&mut self) {
        self.last_edit = LastEdit::Other;
        let chars: Vec<char> = self.lines[self.cursor_y].chars().collect();
        let max = chars.len();
        if self.cursor_x == max {
            self.move_right();
            return;
        }
        while self.cursor_x < max && chars[self.cursor_x].is_whitespace() {
            self.cursor_x += 1;
        }
        while self.cursor_x < max && !chars[self.cursor_x].is_whitespace() {
            self.cursor_x += 1;
        }
    }

    pub fn move_home(&mut self) {
        self.last_edit = LastEdit::Other;
        self.cursor_x = 0;
    }

    pub fn move_end(&mut self) {
        self.last_edit = LastEdit::Other;
        self.cursor_x = self.line_len(self.cursor_y);
    }

    pub fn move_page_up(&mut self) {
        self.last_edit = LastEdit::Other;
        let height = self.visible_height.max(1);
        let (vis_row, _) = find_visual_cursor(&self.layout, self.cursor_y, self.cursor_x);
        if vis_row > 0 {
            let mut target_vi = vis_row.saturating_sub(height);
            while target_vi > 0 && self.layout[target_vi].is_phantom {
                target_vi -= 1;
            }
            self.jump_to_visual_row(target_vi, None);
        } else {
            self.cursor_y = 0;
            self.cursor_x = 0;
        }
    }

    pub fn move_page_down(&mut self) {
        self.last_edit = LastEdit::Other;
        let height = self.visible_height.max(1);
        let (vis_row, _) = find_visual_cursor(&self.layout, self.cursor_y, self.cursor_x);
        if vis_row + 1 < self.layout.len() {
            let mut target_vi = (vis_row + height).min(self.layout.len().saturating_sub(1));
            while target_vi + 1 < self.layout.len() && self.layout[target_vi].is_phantom {
                target_vi += 1;
            }
            self.jump_to_visual_row(target_vi, None);
        } else {
            self.cursor_y = self.lines.len().saturating_sub(1);
            self.cursor_x = self.line_len(self.cursor_y);
        }
    }

    fn jump_to_visual_row(&mut self, target_vi: usize, snap_edge: Option<bool>) {
        let target_line_idx = self.layout[target_vi].line_idx;
        let changed_line = self.cursor_y != target_line_idx;

        let mut offset = 0;
        for i in (0..target_vi).rev() {
            if self.layout[i].line_idx == target_line_idx && !self.layout[i].is_phantom {
                offset += 1;
            } else if self.layout[i].line_idx != target_line_idx {
                break;
            }
        }

        self.cursor_y = target_line_idx;
        let mut final_vi = target_vi;

        if changed_line {
            self.update_layout();

            let new_rows: Vec<usize> = self
                .layout
                .iter()
                .enumerate()
                .filter(|(_, r)| !r.is_phantom && r.line_idx == target_line_idx)
                .map(|(i, _)| i)
                .collect();

            if !new_rows.is_empty() {
                if let Some(moving_down) = snap_edge {
                    if moving_down {
                        final_vi = *new_rows.first().unwrap();
                    } else {
                        final_vi = *new_rows.last().unwrap();
                    }
                } else {
                    final_vi = new_rows[offset.min(new_rows.len().saturating_sub(1))];
                }
            }
        }

        if final_vi < self.layout.len() {
            let target_row = &self.layout[final_vi];
            let is_last = target_row.char_end == self.line_len(target_row.line_idx);
            self.cursor_x = target_row
                .visual_to_logical_x(self.target_visual_x, is_last)
                .min(self.line_len(self.cursor_y));
        }
    }

    pub fn byte_of(&self, y: usize, cx: usize) -> usize {
        self.lines[y]
            .char_indices()
            .nth(cx)
            .map(|(b, _)| b)
            .unwrap_or(self.lines[y].len())
    }

    pub fn insert_char(&mut self, c: char) {
        if self.last_edit != LastEdit::Insert || c.is_whitespace() || ".,;?!()[]*\"'".contains(c) {
            self.save_state(true);
        }
        self.last_edit = LastEdit::Insert;

        let b = self.byte_of(self.cursor_y, self.cursor_x);
        let line = &self.lines[self.cursor_y];

        let next_char = line[b..].chars().next();
        let prev_char = if b > 0 {
            line[..b].chars().next_back()
        } else {
            None
        };

        let mut valid_left_quotes = 0;
        let mut prev_c_in_iter = ' ';
        for ch in line[..b].chars() {
            if ch == c && !(c == '\'' && prev_c_in_iter.is_alphanumeric()) {
                valid_left_quotes += 1;
            }
            prev_c_in_iter = ch;
        }

        let is_inside_string = valid_left_quotes % 2 != 0;
        let next_is_word = next_char.is_some_and(|nc| nc.is_alphanumeric());
        let prev_is_word = prev_char.is_some_and(|pc| pc.is_alphanumeric());

        let step_over = if c == '"' || c == '\'' {
            is_inside_string && next_char == Some(c)
        } else if c == ')' {
            next_char == Some(')')
        } else if c == ']' {
            next_char == Some(']')
        } else {
            false
        };

        if step_over {
            self.cursor_x += 1;
            self.dirty = true;
            return;
        }

        self.lines[self.cursor_y].insert(b, c);
        let new_b = b + c.len_utf8();
        self.cursor_x += 1;

        if true {
            if c == '(' {
                if !next_is_word {
                    self.lines[self.cursor_y].insert(new_b, ')');
                }
            } else if (c == '"' || c == '\'') && !is_inside_string {
                let is_apostrophe = c == '\'' && prev_is_word;
                if !is_apostrophe && !next_is_word {
                    self.lines[self.cursor_y].insert(new_b, c);
                }
            }
        }

        if c == '[' {
            if self.lines[self.cursor_y][..new_b].ends_with("[[") {
                self.lines[self.cursor_y].insert_str(new_b, "]]");
            }
        } else if c == '*' {
            if self.lines[self.cursor_y][..new_b].ends_with("/*") {
                self.lines[self.cursor_y].insert_str(new_b, "*/");
            } else if self.lines[self.cursor_y][..new_b].ends_with("**") {
                self.lines[self.cursor_y].insert_str(new_b, "**");
            }
        }

        self.dirty = true;
    }

    pub fn insert_str(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.save_state(true);
        self.last_edit = LastEdit::Other;

        let b = self.byte_of(self.cursor_y, self.cursor_x);
        let current_line = self.lines[self.cursor_y].clone();
        let prefix = current_line[..b].to_string();
        let suffix = current_line[b..].to_string();

        let mut lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
        if text.ends_with('\n') {
            lines.push(String::new());
        }

        if lines.is_empty() {
            return;
        }

        if lines.len() == 1 {
            self.lines[self.cursor_y] = format!("{}{}{}", prefix, lines[0], suffix);
            self.cursor_x += lines[0].chars().count();
        } else {
            self.lines[self.cursor_y] = format!("{}{}", prefix, lines[0]);
            let mut insert_idx = self.cursor_y + 1;
            for line in lines.iter().take(lines.len() - 1).skip(1) {
                self.lines.insert(insert_idx, line.clone());
                self.types.insert(insert_idx, LineType::Action);
                insert_idx += 1;
            }
            let last_line_content = lines.last().unwrap();
            self.lines
                .insert(insert_idx, format!("{}{}", last_line_content, suffix));
            self.types.insert(insert_idx, LineType::Action);

            self.cursor_y = insert_idx;
            self.cursor_x = last_line_content.chars().count();
        }
        self.dirty = true;
    }

    pub fn insert_newline(&mut self, is_shift: bool) {
        self.save_state(true);
        self.last_edit = LastEdit::Other;

        if is_shift {
            let b = self.byte_of(self.cursor_y, self.cursor_x);
            let tail = self.lines[self.cursor_y].split_off(b);
            self.lines.insert(self.cursor_y + 1, tail);
            self.cursor_y += 1;
            self.cursor_x = 0;
            self.dirty = true;
            return;
        }

        let t = self
            .types
            .get(self.cursor_y)
            .copied()
            .unwrap_or(LineType::Empty);

        let is_smart_element = matches!(
            t,
            LineType::Parenthetical | LineType::Character | LineType::DualDialogueCharacter
        );

        if is_smart_element {
            let b = self.byte_of(self.cursor_y, self.cursor_x);
            let line = &self.lines[self.cursor_y];
            let remainder = &line[b..];
            let trim_rem = remainder.trim();

            if trim_rem.is_empty() || trim_rem == ")" {
                self.lines.insert(self.cursor_y + 1, String::new());
                self.cursor_y += 1;
                self.cursor_x = 0;
                self.dirty = true;
                return;
            }
        }

        let b = self.byte_of(self.cursor_y, self.cursor_x);
        let tail = self.lines[self.cursor_y].split_off(b);
        let head_is_empty = self.lines[self.cursor_y].is_empty();

        let breaks_paragraph = matches!(
            t,
            LineType::Action
                | LineType::SceneHeading
                | LineType::Transition
                | LineType::Section
                | LineType::Synopsis
                | LineType::Shot
                | LineType::Boneyard
                | LineType::Dialogue
                | LineType::Centered
                | LineType::PageBreak
        );

        if self.config.auto_paragraph_breaks && breaks_paragraph && !head_is_empty {
            if tail.trim().is_empty() {
                self.lines.insert(self.cursor_y + 1, String::new());
                self.lines.insert(self.cursor_y + 2, String::new());
                self.cursor_y += 2;
            } else {
                self.lines.insert(self.cursor_y + 1, String::new());
                self.lines.insert(self.cursor_y + 2, String::new());
                self.lines.insert(self.cursor_y + 3, String::new());
                self.lines
                    .insert(self.cursor_y + 4, tail.trim_start().to_string());
                self.cursor_y += 2;
            }
        } else {
            self.lines.insert(self.cursor_y + 1, tail);
            self.cursor_y += 1;
        }

        self.cursor_x = 0;
        self.dirty = true;
    }

    pub fn handle_tab(&mut self) {
        if let Some(sug) = self.suggestion.take() {
            self.save_state(true);
            self.last_edit = LastEdit::Other;

            self.lines[self.cursor_y] = self.lines[self.cursor_y].to_uppercase_1to1();

            let b = self.byte_of(self.cursor_y, self.cursor_x);
            self.lines[self.cursor_y].insert_str(b, &sug);
            self.cursor_x += sug.chars().count();

            self.parse_document();
            let parsed_type = self.types[self.cursor_y];

            let line = self.lines[self.cursor_y].clone();
            let clean_line = crate::layout::strip_sigils(&line, parsed_type)
                .trim()
                .to_string();

            let is_char = self.characters.contains(&clean_line);
            let is_loc = self.locations.contains(&clean_line);

            if is_char
                && parsed_type != LineType::Character
                && parsed_type != LineType::DualDialogueCharacter
            {
                if !self.lines[self.cursor_y].starts_with('@') {
                    self.lines[self.cursor_y].insert(0, '@');
                    self.cursor_x += 1;
                }
            } else if is_loc
                && !is_char
                && parsed_type != LineType::SceneHeading
                && !self.lines[self.cursor_y].starts_with('.')
            {
                self.lines[self.cursor_y].insert(0, '.');
                self.cursor_x += 1;
            }

            self.dirty = true;
            return;
        }

        self.save_state(true);
        self.last_edit = LastEdit::Other;

        let lt = self.types[self.cursor_y];
        let line = self.lines[self.cursor_y].clone();
        let trim = line.trim();
        let prev_t = if self.cursor_y > 0 {
            self.types[self.cursor_y - 1]
        } else {
            LineType::Empty
        };

        if trim.is_empty() {
            if matches!(
                prev_t,
                LineType::Character
                    | LineType::DualDialogueCharacter
                    | LineType::Dialogue
                    | LineType::Parenthetical
            ) {
                self.lines[self.cursor_y] = "()".to_string();
                self.cursor_x = 1;
            } else {
                self.lines[self.cursor_y] = "@".to_string();
                self.cursor_x = 1;
            }
        } else if trim == "()" {
            self.lines[self.cursor_y] = "@".to_string();
            self.cursor_x = 1;
        } else if trim == "@" {
            self.lines[self.cursor_y] = ".".to_string();
            self.cursor_x = 1;
        } else if trim == "." {
            self.lines[self.cursor_y] = ">".to_string();
            self.cursor_x = 1;
        } else if trim == ">" {
            self.lines[self.cursor_y] = String::new();
            self.cursor_x = 0;
        } else if lt == LineType::Action {
            if line.starts_with('!')
                || line.starts_with('~')
                || line.starts_with('=')
                || line.starts_with('#')
            {
                let stripped = line.trim_start_matches(['!', '~', '=', '#']);
                self.lines[self.cursor_y] = stripped.to_string();
                self.cursor_x = self.cursor_x.saturating_sub(line.len() - stripped.len());
            } else if line.starts_with('.') {
                self.lines[self.cursor_y] = line.replacen('.', ">", 1);
            } else if !line.starts_with('@') {
                let upper_prefix = line.trim_start().to_uppercase_1to1();
                let mut best_match: Option<&String> = None;

                if !upper_prefix.is_empty() {
                    for c in &self.characters {
                        if c.starts_with(&upper_prefix)
                            && c.len() > upper_prefix.len()
                            && (best_match.is_none() || c.len() < best_match.unwrap().len())
                        {
                            best_match = Some(c);
                        }
                    }
                }

                if let Some(c) = best_match {
                    self.suggestion = Some(c[upper_prefix.len()..].to_string());
                } else {
                    self.lines[self.cursor_y].insert(0, '@');
                    self.cursor_x += 1;
                }
            }
        } else if matches!(
            lt,
            LineType::Shot | LineType::Lyrics | LineType::Synopsis | LineType::Section
        ) {
            let stripped = line.trim_start_matches(['!', '~', '=', '#']);
            self.lines[self.cursor_y] = stripped.to_string();
            self.cursor_x = self.cursor_x.saturating_sub(line.len() - stripped.len());
        } else if lt == LineType::Character || lt == LineType::DualDialogueCharacter {
            if line.starts_with('@') {
                self.lines[self.cursor_y] = line.replacen('@', ".", 1);
            } else {
                self.lines[self.cursor_y].insert(0, '.');
                self.cursor_x += 1;
            }
        } else if lt == LineType::Dialogue {
            self.lines[self.cursor_y] = format!("({})", trim);
            self.cursor_x = self.lines[self.cursor_y].chars().count() - 1;
        } else if lt == LineType::Parenthetical {
            if trim.starts_with('(') && trim.ends_with(')') {
                self.lines[self.cursor_y] = trim[1..trim.len() - 1].to_string();
                self.cursor_x = self.lines[self.cursor_y].chars().count();
            } else if line.starts_with('(') {
                let mut s = line.replacen('(', "", 1);
                if let Some(idx) = s.rfind(')') {
                    s.remove(idx);
                }
                self.lines[self.cursor_y] = s;
                self.cursor_x = self.cursor_x.saturating_sub(1);
            }
        } else if lt == LineType::SceneHeading {
            if line.starts_with('.') {
                self.lines[self.cursor_y] = line.replacen('.', ">", 1);
            } else {
                self.lines[self.cursor_y].insert(0, '>');
                self.cursor_x += 1;
            }
        } else if lt == LineType::Transition
            && line.starts_with('>')
            && let Some(stripped) = line.strip_prefix('>')
        {
            self.lines[self.cursor_y] = stripped.to_string();
            self.cursor_x = self.cursor_x.saturating_sub(1);
        } else if line.starts_with('!')
            || line.starts_with('~')
            || line.starts_with('=')
            || line.starts_with('#')
        {
            let stripped = line.trim_start_matches(['!', '~', '=', '#']);
            self.lines[self.cursor_y] = stripped.to_string();
            self.cursor_x = self.cursor_x.saturating_sub(line.len() - stripped.len());
        }
        self.dirty = true;
    }

    pub fn backspace(&mut self) {
        if self.last_edit != LastEdit::Delete {
            self.save_state(true);
        }
        self.last_edit = LastEdit::Delete;

        let max = self.line_len(self.cursor_y);
        if self.cursor_x > max {
            self.cursor_x = max;
        }

        if self.cursor_x > 0 {
            let line = &self.lines[self.cursor_y];
            let cx = self.cursor_x;

            if cx >= 1 && cx < line.chars().count() {
                let mut chars = line.chars().skip(cx - 1);
                if let (Some(c1), Some(c2)) = (chars.next(), chars.next())
                    && matches!((c1, c2), ('(', ')') | ('"', '"') | ('\'', '\''))
                {
                    let b_start = self.byte_of(self.cursor_y, cx - 1);
                    let b_end = self.byte_of(self.cursor_y, cx + 1);
                    self.lines[self.cursor_y].replace_range(b_start..b_end, "");
                    self.cursor_x -= 1;
                    self.dirty = true;
                    return;
                }
            }

            if cx >= 2 && cx + 1 < line.chars().count() {
                let mut chars = line.chars().skip(cx - 2);
                if let (Some(c1), Some(c2), Some(c3), Some(c4)) =
                    (chars.next(), chars.next(), chars.next(), chars.next())
                {
                    let arr = [c1, c2, c3, c4];
                    if matches!(
                        arr,
                        ['[', '[', ']', ']'] | ['/', '*', '*', '/'] | ['*', '*', '*', '*']
                    ) {
                        let b_start = self.byte_of(self.cursor_y, cx - 2);
                        let b_end = self.byte_of(self.cursor_y, cx + 2);
                        self.lines[self.cursor_y].replace_range(b_start..b_end, "");
                        self.cursor_x -= 2;
                        self.dirty = true;
                        return;
                    }
                }
            }

            let b = self.byte_of(self.cursor_y, self.cursor_x - 1);
            self.lines[self.cursor_y].remove(b);
            self.cursor_x -= 1;
            self.dirty = true;
        } else if self.cursor_y > 0 {
            let tail = self.lines.remove(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = self.line_len(self.cursor_y);
            self.lines[self.cursor_y].push_str(&tail);
            self.dirty = true;
        }
    }

    pub fn delete_forward(&mut self) {
        if self.last_edit != LastEdit::Delete {
            self.save_state(true);
        }
        self.last_edit = LastEdit::Delete;

        let max = self.line_len(self.cursor_y);
        if self.cursor_x > max {
            self.cursor_x = max;
        }

        let line = &self.lines[self.cursor_y];
        let cx = self.cursor_x;

        let mut chars = line.chars().skip(cx);
        let c1 = chars.next();
        let c2 = chars.next();
        let c3 = chars.next();
        let c4 = chars.next();

        if let (Some(a), Some(b)) = (c1, c2) {
            if matches!((a, b), ('(', ')') | ('"', '"') | ('\'', '\'')) {
                let b_start = self.byte_of(self.cursor_y, cx);
                let b_end = self.byte_of(self.cursor_y, cx + 2);
                self.lines[self.cursor_y].replace_range(b_start..b_end, "");
                self.dirty = true;
                return;
            }

            if let (Some(c), Some(d)) = (c3, c4)
                && matches!(
                    [a, b, c, d],
                    ['[', '[', ']', ']'] | ['/', '*', '*', '/'] | ['*', '*', '*', '*']
                )
            {
                let b_start = self.byte_of(self.cursor_y, cx);
                let b_end = self.byte_of(self.cursor_y, cx + 4);
                self.lines[self.cursor_y].replace_range(b_start..b_end, "");
                self.dirty = true;
                return;
            }
        }

        if self.cursor_x < max {
            let b = self.byte_of(self.cursor_y, self.cursor_x);
            self.lines[self.cursor_y].remove(b);
            self.dirty = true;
        } else if self.cursor_y + 1 < self.lines.len() {
            let next = self.lines.remove(self.cursor_y + 1);
            self.lines[self.cursor_y].push_str(&next);
            self.dirty = true;
        }
    }

    pub fn delete_word_back(&mut self) {
        let max = self.line_len(self.cursor_y);
        if self.cursor_x > max {
            self.cursor_x = max;
        }

        if self.cursor_x == 0 {
            self.backspace();
            return;
        }
        self.save_state(true);
        self.last_edit = LastEdit::Other;

        let mut chars: Vec<char> = self.lines[self.cursor_y].chars().collect();
        while self.cursor_x > 0 && chars[self.cursor_x - 1].is_whitespace() {
            self.cursor_x -= 1;
            chars.remove(self.cursor_x);
        }
        while self.cursor_x > 0 && !chars[self.cursor_x - 1].is_whitespace() {
            self.cursor_x -= 1;
            chars.remove(self.cursor_x);
        }
        self.lines[self.cursor_y] = chars.into_iter().collect();
        self.dirty = true;
    }

    pub fn delete_word_forward(&mut self) {
        let max = self.line_len(self.cursor_y);
        if self.cursor_x > max {
            self.cursor_x = max;
        }

        let mut chars: Vec<char> = self.lines[self.cursor_y].chars().collect();
        if self.cursor_x == chars.len() {
            self.delete_forward();
            return;
        }
        self.save_state(true);
        self.last_edit = LastEdit::Other;

        while self.cursor_x < chars.len() && chars[self.cursor_x].is_whitespace() {
            chars.remove(self.cursor_x);
        }
        while self.cursor_x < chars.len() && !chars[self.cursor_x].is_whitespace() {
            chars.remove(self.cursor_x);
        }
        self.lines[self.cursor_y] = chars.into_iter().collect();
        self.dirty = true;
    }

    pub fn calculate_scene_height(&self, item: &NavigatorItem) -> usize {
        if item.is_section {
            return 2; // Section name + spacer
        }

        let max_w: usize = 45; // Match the wider navigator sidebar
        let mut height: usize = 0;

        // Heading wrapping
        let mut current_line_len: usize = 0;
        let heading_indent: usize = 5; // prefix(3) + connector(2)
        for word in item.label.split_whitespace() {
            if current_line_len + word.len() + heading_indent + 1 > max_w {
                height += 1;
                current_line_len = 0;
            }
            if current_line_len > 0 {
                current_line_len += 1;
            }
            current_line_len += word.len();
        }
        if current_line_len > 0 || height == 0 {
            height += 1;
        }

        // Synopsis wrapping
        for syn in &item.synopses {
            let mut current_line_len: usize = 0;
            let max_syn_w = 34; // Sync with UI
            let mut syn_lines: usize = 0;
            for word in syn.split_whitespace() {
                if current_line_len + word.len() + 1 > max_syn_w {
                    syn_lines += 1;
                    current_line_len = word.len();
                } else {
                    if current_line_len > 0 {
                        current_line_len += 1;
                    }
                    current_line_len += word.len();
                }
            }
            if current_line_len > 0 {
                syn_lines += 1;
            }
            height += syn_lines;
        }

        if item.synopses.is_empty() {
            height += 1; // "no synopsis" placeholder height
        }

        height += 1; // Empty separator line or ending spacer
        height
    }

}
