use std::{collections::HashSet, fs, io, path::PathBuf};

use ratatui::{
    layout::Rect,
    style::Color,
    widgets::ListState,
};


use crate::{
    config::{Cli, Config},
    formatting::StringCaseExt,
    layout::{VisualRow, build_layout, find_visual_cursor, strip_sigils},
    parser::Parser,
    types::LineType,
};

#[derive(Clone)]

pub struct HistoryState {
    pub lines: Vec<String>,

    pub cursor_y: usize,

    pub cursor_x: usize,
}

#[derive(PartialEq, Clone, Default)]

pub enum LastEdit {
    #[default]
    None,

    Insert,

    Delete,

    Cut,

    Other,
}

#[derive(PartialEq, Debug)]
pub enum AppMode {
    Normal,

    Search,

    PromptSave,

    PromptFilename,

    SceneNavigator,

    SettingsPane,

    Shortcuts,

    ExportPane,

    Command,

    Home,
}

#[derive(Clone, Default)]
pub struct BufferState {
    pub lines: Vec<String>,

    pub types: Vec<LineType>,

    pub layout: Vec<VisualRow>,

    pub file: Option<PathBuf>,

    pub dirty: bool,

    pub cursor_y: usize,

    pub cursor_x: usize,

    pub target_visual_x: u16,

    pub scroll: usize,

    pub characters: HashSet<String>,

    pub locations: HashSet<String>,

    pub undo_stack: Vec<HistoryState>,

    pub redo_stack: Vec<HistoryState>,

    pub last_edit: LastEdit,
}

pub struct App {
    pub config: Config,

    pub buffers: Vec<BufferState>,

    pub current_buf_idx: usize,

    pub has_multiple_buffers: bool,

    pub lines: Vec<String>,

    pub types: Vec<LineType>,

    pub layout: Vec<VisualRow>,

    pub file: Option<PathBuf>,

    pub dirty: bool,

    pub cursor_y: usize,

    pub cursor_x: usize,

    pub target_visual_x: u16,

    pub visible_height: usize,

    pub scroll: usize,

    pub characters: HashSet<String>,

    pub locations: HashSet<String>,

    pub suggestion: Option<String>,

    pub undo_stack: Vec<HistoryState>,

    pub redo_stack: Vec<HistoryState>,

    pub last_edit: LastEdit,

    pub mode: AppMode,

    pub exit_after_save: bool,

    pub filename_input: String,

    pub status_msg: Option<String>,

    pub cut_buffer: Option<String>,

    pub search_query: String,

    pub last_search: String,

    pub show_search_highlight: bool,

    pub compiled_search_regex: Option<regex::Regex>,

    pub scenes: Vec<(usize, String, Option<String>, Vec<String>, Option<Color>)>,

    pub selected_scene: usize,

    pub selected_setting: usize,

    pub selected_export_option: usize,

    pub sidebar_area: Rect,

    pub settings_area: Rect,

    pub navigator_state: ListState,
    
    pub command_input: String,

    pub command_error: bool,

    pub selection_anchor: Option<(usize, usize)>,

    pub home_selected: usize,
}

impl Drop for App {
    fn drop(&mut self) {
        #[cfg(not(test))]
        if std::thread::panicking() {
            self.emergency_save();
        }
    }
}

impl App {
    pub fn new(cli: Cli) -> Self {
        let config = Config::load(&cli);

        let mut files = Vec::new();
        if cli.files.is_empty() {
            files.push(None);
        } else {
            let mut seen = std::collections::HashSet::new();
            for path in cli.files.clone() {
                let normalized = path.canonicalize().unwrap_or_else(|_| path.clone());
                if seen.insert(normalized) {
                    files.push(Some(path));
                }
            }
        }

        let mut buffers = Vec::new();
        for path in files {
            let mut is_new_or_empty = false;
            let lines = match &path {
                Some(p) if p.exists() => {
                    let text = fs::read_to_string(p)
                        .unwrap_or_default()
                        .replace('\t', "    ");
                    if text.trim().is_empty() {
                        is_new_or_empty = true;
                        vec![String::new()]
                    } else {
                        let ls: Vec<String> = text.lines().map(str::to_string).collect();
                        if ls.is_empty() {
                            vec![String::new()]
                        } else {
                            ls
                        }
                    }
                }
                _ => {
                    is_new_or_empty = true;
                    vec![String::new()]
                }
            };

            let mut buf = BufferState {
                lines,
                file: path,
                ..Default::default()
            };

            if is_new_or_empty && config.auto_title_page {
                buf.lines = vec![
                    "Title: Untitled".to_string(),
                    "Credit: Written by".to_string(),
                    "Author: ".to_string(),
                    "Draft date: ".to_string(),
                    "Contact: ".to_string(),
                    "".to_string(),
                    "".to_string(),
                ];
                buf.cursor_y = buf.lines.len() - 1;
                buf.dirty = true;
            } else if config.goto_end {
                buf.cursor_y = buf.lines.len().saturating_sub(1);
                buf.cursor_x = buf.lines[buf.cursor_y].chars().count();
            }
            buffers.push(buf);
        }

        let has_multiple_buffers = buffers.len() > 1;

        let mut app = Self {
            config,
            buffers,
            current_buf_idx: 0,
            has_multiple_buffers,

            lines: Vec::new(),
            types: Vec::new(),
            layout: Vec::new(),
            file: None,
            dirty: false,
            cursor_y: 0,
            cursor_x: 0,
            target_visual_x: 0,
            visible_height: 0,
            scroll: 0,
            characters: HashSet::new(),
            locations: HashSet::new(),
            suggestion: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_edit: LastEdit::None,

            mode: AppMode::Normal,
            exit_after_save: false,
            filename_input: String::new(),

            status_msg: None,
            cut_buffer: None,
            search_query: String::new(),
            last_search: String::new(),
            show_search_highlight: false,
            compiled_search_regex: None,
            scenes: Vec::new(),
            selected_scene: 0,
            selected_setting: 0,
            selected_export_option: 0,
            sidebar_area: Rect::default(),
            settings_area: Rect::default(),
            navigator_state: ListState::default(),
            command_input: String::new(),
            command_error: false,
            selection_anchor: None,
            home_selected: 0,
        };

        let mut first_buf = std::mem::take(&mut app.buffers[0]);
        app.swap_buffer(&mut first_buf);

        app.parse_document();
        app.update_autocomplete();
        app.update_layout();
        app.target_visual_x = app.current_visual_x();

        // Show home screen if opened without a file argument
        if cli.files.is_empty() {
            app.mode = AppMode::Home;
        }

        app
    }

    pub fn swap_buffer(&mut self, other: &mut BufferState) {
        std::mem::swap(&mut self.lines, &mut other.lines);
        std::mem::swap(&mut self.types, &mut other.types);
        std::mem::swap(&mut self.layout, &mut other.layout);
        std::mem::swap(&mut self.file, &mut other.file);
        std::mem::swap(&mut self.dirty, &mut other.dirty);
        std::mem::swap(&mut self.cursor_y, &mut other.cursor_y);
        std::mem::swap(&mut self.cursor_x, &mut other.cursor_x);
        std::mem::swap(&mut self.target_visual_x, &mut other.target_visual_x);
        std::mem::swap(&mut self.scroll, &mut other.scroll);
        std::mem::swap(&mut self.characters, &mut other.characters);
        std::mem::swap(&mut self.locations, &mut other.locations);
        std::mem::swap(&mut self.undo_stack, &mut other.undo_stack);
        std::mem::swap(&mut self.redo_stack, &mut other.redo_stack);
        std::mem::swap(&mut self.last_edit, &mut other.last_edit);
    }

    pub fn switch_buffer(&mut self, next_idx: usize) {
        if self.buffers.len() <= 1 || next_idx == self.current_buf_idx {
            return;
        }

        let mut current_state = BufferState::default();

        self.swap_buffer(&mut current_state);
        self.buffers[self.current_buf_idx] = current_state;
        self.current_buf_idx = next_idx;

        let mut next_state = std::mem::take(&mut self.buffers[self.current_buf_idx]);

        self.swap_buffer(&mut next_state);

        self.parse_document();
        self.update_autocomplete();
        self.update_layout();
        self.target_visual_x = self.current_visual_x();

        let file_name = self
            .file
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "New Buffer".to_string());

        let line_count = self.lines.len();
        let line_word = if line_count == 1 { "line" } else { "lines" };

        self.set_status(&format!("{} -- {} {}", file_name, line_count, line_word));
    }

    pub fn switch_next_buffer(&mut self) {
        let next = (self.current_buf_idx + 1) % self.buffers.len();
        self.switch_buffer(next);
    }

    pub fn switch_prev_buffer(&mut self) {
        let prev = if self.current_buf_idx == 0 {
            self.buffers.len() - 1
        } else {
            self.current_buf_idx - 1
        };
        self.switch_buffer(prev);
    }

    pub fn close_current_buffer(&mut self) -> bool {
        if self.buffers.len() <= 1 {
            return true;
        }

        self.buffers.remove(self.current_buf_idx);
        if self.current_buf_idx >= self.buffers.len() {
            self.current_buf_idx = self.buffers.len() - 1;
        }

        let mut dummy = BufferState::default();
        self.swap_buffer(&mut dummy);

        let mut next_state = std::mem::take(&mut self.buffers[self.current_buf_idx]);
        self.swap_buffer(&mut next_state);

        self.parse_document();
        self.update_autocomplete();
        self.update_layout();
        self.target_visual_x = self.current_visual_x();

        let file_name = self
            .file
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "New Buffer".to_string());
        let line_count = self.lines.len();
        let line_word = if line_count == 1 { "line" } else { "lines" };
        self.set_status(&format!("{} -- {} {}", file_name, line_count, line_word));

        false
    }

    #[allow(dead_code)]
    pub fn emergency_save(&mut self) {
        let mut to_save = Vec::new();
        to_save.push((self.file.clone(), &self.lines, self.dirty));

        for (i, buf) in self.buffers.iter().enumerate() {
            if i != self.current_buf_idx {
                to_save.push((buf.file.clone(), &buf.lines, buf.dirty));
            }
        }

        for (file, lines, dirty) in to_save {
            if !dirty || lines.is_empty() || (lines.len() == 1 && lines[0].is_empty()) {
                continue;
            }

            let dir = file
                .as_ref()
                .and_then(|p| p.parent())
                .filter(|p| !p.as_os_str().is_empty())
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| {
                    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
                });

            let base_name = file
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "fount".to_string());

            let mut backup_path = dir.join(format!("{}.save", base_name));
            let mut counter = 1;

            while backup_path.exists() && counter <= 1000 {
                backup_path = dir.join(format!("{}.save.{}", base_name, counter));
                counter += 1;
            }

            if counter <= 1000 {
                let content = lines.join("\n");
                let _ = std::fs::write(&backup_path, content);
            }
        }
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status_msg = Some(msg.to_string());
    }

    pub fn clear_status(&mut self) {
        self.status_msg = None;
    }

    pub fn update_search_regex(&mut self) {
        let active_query = if self.search_query.is_empty() {
            &self.last_search
        } else {
            &self.search_query
        };

        if active_query.is_empty() {
            self.compiled_search_regex = None;
        } else {
            self.compiled_search_regex = regex::RegexBuilder::new(&regex::escape(active_query))
                .case_insensitive(true)
                .build()
                .ok();
        }
    }

    pub fn report_cursor_position(&mut self) {
        if self.lines.is_empty() {
            self.set_status("line 1/1 (100%), col 1/1 (100%), char 1/1 (100%)");
            return;
        }

        let total_lines = self.lines.len().max(1);
        let cur_line = self.cursor_y + 1;
        let line_pct = (cur_line as f64 / total_lines as f64 * 100.0) as usize;

        let current_line_text = self
            .lines
            .get(self.cursor_y)
            .map(|s| s.as_str())
            .unwrap_or("");
        let total_cols = current_line_text.chars().count() + 1;
        let cur_col = self.cursor_x + 1;
        let col_pct = (cur_col as f64 / total_cols as f64 * 100.0) as usize;

        let total_chars: usize = self
            .lines
            .iter()
            .map(|l| l.chars().count() + 1)
            .sum::<usize>()
            .max(1);

        let cur_char = self.lines[..self.cursor_y]
            .iter()
            .map(|l| l.chars().count() + 1)
            .sum::<usize>()
            + self.cursor_x
            + 1;

        let char_pct = (cur_char as f64 / total_chars as f64 * 100.0) as usize;

        let msg = format!(
            "line {}/{} ({}%), col {}/{} ({}%), char {}/{} ({}%)",
            cur_line,
            total_lines,
            line_pct,
            cur_col,
            total_cols,
            col_pct,
            cur_char,
            total_chars,
            char_pct
        );
        self.set_status(&msg);
    }

    pub fn total_word_count(&self) -> usize {
        self.lines
            .iter()
            .map(|l| l.split_whitespace().count())
            .sum()
    }

    pub fn total_page_count(&self) -> usize {
        self.layout
            .iter()
            .filter_map(|r| r.page_num)
            .last()
            .unwrap_or(1)
    }

    pub fn current_page_number(&self) -> usize {
        let (vis_row_idx, _) = find_visual_cursor(&self.layout, self.cursor_y, self.cursor_x);
        for i in (0..=vis_row_idx).rev() {
            if let Some(p) = self.layout[i].page_num {
                return p;
            }
        }
        1
    }

    pub fn open_scene_navigator(&mut self) {
        self.scenes.clear();
        let mut current_scene: Option<(usize, String, Option<String>, Vec<String>, Option<Color>)> =
            None;
        let mut last_color: Option<Color> = None;

        for row in &self.layout {
            if row.line_type == LineType::Note {
                last_color = row.override_color;
            }

            if row.line_type == LineType::SceneHeading {
                if let Some(s) = current_scene.take() {
                    self.scenes.push(s);
                }
                let heading = strip_sigils(&row.raw_text, row.line_type).to_uppercase_1to1();
                let color = row.override_color.or(last_color);
                current_scene = Some((
                    row.line_idx,
                    heading,
                    row.scene_num.clone(),
                    Vec::new(),
                    color,
                ));
                last_color = None;
            } else if row.line_type == LineType::Synopsis {
                if let Some(ref mut s) = current_scene {
                    let note_text = strip_sigils(&row.raw_text, row.line_type).to_string();
                    if !note_text.is_empty() {
                        s.3.push(note_text);
                    }
                }
                last_color = None;
            } else if row.line_type != LineType::Empty {
                last_color = None;
            }

            if let Some(ref mut s) = current_scene {
                if s.4.is_none() {
                    if let Some(c) = row.override_color {
                        s.4 = Some(c);
                    } else if let Some(c) = row.fmt.note_color.values().next() {
                        s.4 = Some(*c);
                    }
                }
            }
        }
        if let Some(s) = current_scene {
            self.scenes.push(s);
        }

        if self.scenes.is_empty() {
            self.set_status("No scenes found");
        } else {
            self.mode = AppMode::SceneNavigator;
            self.selected_scene = 0;
            for (idx, (line_idx, _, _, _, _)) in self.scenes.iter().enumerate() {
                if *line_idx <= self.cursor_y {
                    self.selected_scene = idx;
                } else {
                    break;
                }
            }
            self.navigator_state.select(Some(self.selected_scene));
        }
    }

    pub fn cut_line(&mut self) {
        if self.last_edit != LastEdit::Cut {
            self.save_state(true);
        }

        if self.cursor_y < self.lines.len() {
            let cut_line = self.lines.remove(self.cursor_y);

            if self.last_edit == LastEdit::Cut {
                if let Some(buf) = &mut self.cut_buffer {
                    buf.push('\n');
                    buf.push_str(&cut_line);
                }
            } else {
                self.cut_buffer = Some(cut_line);
            }
            self.last_edit = LastEdit::Cut;

            if self.lines.is_empty() {
                self.lines.push(String::new());
            }
            if self.cursor_y >= self.lines.len() {
                self.cursor_y = self.lines.len().saturating_sub(1);
                self.cursor_x = self.line_len(self.cursor_y);
            } else {
                self.cursor_x = 0;
            }
            self.dirty = true;
        }
    }

    pub fn paste_line(&mut self) {
        if let Some(cut_buf) = self.cut_buffer.clone() {
            self.save_state(true);
            let lines_to_paste: Vec<&str> = cut_buf.split('\n').collect();
            for (i, l) in lines_to_paste.iter().enumerate() {
                self.lines
                    .insert(self.cursor_y + i, l.replace('\t', "    "));
            }
            self.cursor_y += lines_to_paste.len();
            self.cursor_x = 0;
            self.dirty = true;
            self.last_edit = LastEdit::Other;
        }
    }

    pub fn execute_search(&mut self) {
        if self.search_query.is_empty() {
            self.search_query = self.last_search.clone();
        }
        if self.search_query.is_empty() {
            self.mode = AppMode::Normal;
            self.set_status("Cancelled");
            self.show_search_highlight = false;
            self.compiled_search_regex = None;
            return;
        }
        self.last_search = self.search_query.clone();
        self.update_search_regex();

        let re = self.compiled_search_regex.as_ref().unwrap();

        let mut wrapped = false;
        let mut found = false;
        let start_y = self.cursor_y;
        let start_char_x = self.cursor_x;

        for i in 0..=self.lines.len() {
            let y = (start_y + i) % self.lines.len();
            let line = &self.lines[y];

            for mat in re.find_iter(line) {
                let char_idx = line[..mat.start()].chars().count();

                if i == 0 && char_idx <= start_char_x {
                    continue;
                }

                if i == self.lines.len() && char_idx > start_char_x {
                    continue;
                }

                self.cursor_y = y;
                self.cursor_x = char_idx;
                found = true;

                if y < start_y || (y == start_y && i > 0) {
                    wrapped = true;
                }
                break;
            }
            if found {
                break;
            }
        }

        self.mode = AppMode::Normal;

        if !found {
            self.set_status(&format!("\"{}\" not found", self.search_query));
            self.show_search_highlight = false;
        } else {
            self.show_search_highlight = true;
            if wrapped {
                self.set_status("Search Wrapped");
            } else {
                self.clear_status();
            }
        }

        self.search_query.clear();
    }

    pub fn save_state(&mut self, force: bool) {
        let state = HistoryState {
            lines: self.lines.clone(),
            cursor_y: self.cursor_y,
            cursor_x: self.cursor_x,
        };
        if force
            || self
                .undo_stack
                .last()
                .is_none_or(|last| last.lines != state.lines)
        {
            self.undo_stack.push(state);
            if self.undo_stack.len() > 640 {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some(state) = self.undo_stack.pop() {
            self.redo_stack.push(HistoryState {
                lines: self.lines.clone(),
                cursor_y: self.cursor_y,
                cursor_x: self.cursor_x,
            });
            self.lines = state.lines;
            self.cursor_y = state.cursor_y;
            self.cursor_x = state.cursor_x;
            self.dirty = true;
            self.last_edit = LastEdit::None;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(state) = self.redo_stack.pop() {
            self.undo_stack.push(HistoryState {
                lines: self.lines.clone(),
                cursor_y: self.cursor_y,
                cursor_x: self.cursor_x,
            });
            self.lines = state.lines;
            self.cursor_y = state.cursor_y;
            self.cursor_x = state.cursor_x;
            self.dirty = true;
            self.last_edit = LastEdit::None;
            true
        } else {
            false
        }
    }

    pub fn parse_document(&mut self) {
        self.types = Parser::parse(&self.lines);
        self.characters.clear();
        self.locations.clear();

        for (i, t) in self.types.iter().enumerate() {
            if *t == LineType::Character || *t == LineType::DualDialogueCharacter {
                let full_name = self.lines[i]
                    .trim_start_matches('@')
                    .trim_end_matches('^')
                    .trim();
                let name = if let Some(idx) = full_name.find('(') {
                    full_name[..idx].trim()
                } else {
                    full_name
                };
                if !name.is_empty() {
                    self.characters.insert(name.to_uppercase_1to1());
                }
            } else if *t == LineType::SceneHeading {
                let scene = self.lines[i].trim().to_uppercase_1to1();
                let mut loc_str = scene.as_str();
                let mut matched = false;

                if loc_str.starts_with('.') && !loc_str.starts_with("..") {
                    loc_str = &loc_str[1..];
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
                        if let Some(rest) = loc_str.strip_prefix(p) {
                            loc_str = rest;
                            matched = true;
                            break;
                        }
                    }
                    if !matched && let Some((_, rest)) = loc_str.split_once(". ") {
                        loc_str = rest;
                    }
                }

                let mut final_loc = loc_str.trim().to_string();

                if final_loc.ends_with('#')
                    && let Some(idx) = final_loc.rfind(" #")
                {
                    final_loc.truncate(idx);
                    final_loc = final_loc.trim().to_string();
                }

                if !final_loc.is_empty() {
                    self.locations.insert(final_loc);
                }
            }
        }
    }

    /// Strips a trailing `#num#` tag from a scene heading line if present.
    fn strip_scene_number_from_line(line: &str) -> &str {
        let trimmed = line.trim_end();
        if trimmed.ends_with('#') {
            if let Some(open) = trimmed[..trimmed.len() - 1].rfind('#') {
                let inner = &trimmed[open + 1..trimmed.len() - 1];
                if !inner.is_empty() && !inner.contains(' ') {
                    return trimmed[..open].trim_end();
                }
            }
        }
        line
    }

    /// Calculates what scene-number integer a given scene index should receive,
    /// walking all scene headings up to `target_idx` and applying the same
    /// cascading logic used by the full renumber pass.
    fn compute_scene_number_for(&self, target_line_idx: usize) -> usize {
        let mut count = 1usize;
        for i in 0..self.lines.len() {
            if self.types[i] != LineType::SceneHeading {
                continue;
            }
            let base = Self::strip_scene_number_from_line(&self.lines[i]);
            // Check if a non-integer override is already present
            let existing: Option<&str> = {
                let t = self.lines[i].trim_end();
                if t.ends_with('#') {
                    t[..t.len()-1].rfind('#').and_then(|o| {
                        let inner = &t[o+1..t.len()-1];
                        if !inner.is_empty() && !inner.contains(' ') { Some(inner) } else { None }
                    })
                } else { None }
            };
            if i == target_line_idx {
                // If the existing tag is non-integer (custom lock like 14B) keep count as-is
                if let Some(num) = existing {
                    if !num.chars().all(|c| c.is_ascii_digit()) {
                        return count; // will be ignored by caller
                    }
                }
                return count;
            }
            // Advance count only for scenes that consume an integer slot
            let _ = base;
            if let Some(num) = existing {
                if num.chars().all(|c| c.is_ascii_digit()) {
                    count += 1;
                }
                // non-integer overrides don't consume a slot
            } else {
                count += 1;
            }
        }
        count
    }

    /// Numbers ALL scene headings chronologically. Respects non-integer custom
    /// overrides (e.g. `14B`) — those scenes keep their tag, subsequent integer
    /// scenes are re-indexed. Ignores `production_lock`; this is an explicit
    /// on-demand action.
    pub fn renumber_all_scenes(&mut self) {
        let mut count = 1usize;
        let mut changed = false;

        for i in 0..self.lines.len() {
            if self.types[i] != LineType::SceneHeading {
                continue;
            }
            let base = Self::strip_scene_number_from_line(&self.lines[i]).to_string();
            // Detect existing custom (non-integer) tag
            let existing_custom: Option<String> = {
                let t = self.lines[i].trim_end();
                if t.ends_with('#') {
                    t[..t.len()-1].rfind('#').and_then(|o| {
                        let inner = &t[o+1..t.len()-1];
                        if !inner.is_empty() && !inner.contains(' ') && !inner.chars().all(|c| c.is_ascii_digit()) {
                            Some(inner.to_string())
                        } else { None }
                    })
                } else { None }
            };

            let new_line = if let Some(custom) = existing_custom {
                // Keep custom tag, don't consume an integer slot
                format!("{} #{}#", base, custom)
            } else {
                let n = count;
                count += 1;
                format!("{} #{}#", base, n)
            };

            if self.lines[i] != new_line {
                self.lines[i] = new_line;
                changed = true;
            }
        }

        if changed {
            self.parse_document();
        }
    }

    /// Injects or updates the scene number **only for the line the cursor is on**.
    /// Does nothing if the current line is not a scene heading.
    /// Respects `production_lock`: if locked, this call is still allowed (it's
    /// triggered explicitly by the user). 
    /// Unlike `renumber_all_scenes`, this only touches one line.
    pub fn inject_current_scene_number(&mut self) {
        self.inject_scene_number_tag(None);
    }

    /// Inject a specific scene number tag `#tag#` or auto-compute one.
    pub fn inject_scene_number_tag(&mut self, tag: Option<&str>) {
        let y = self.cursor_y;
        if y >= self.types.len() || self.types[y] != LineType::SceneHeading {
            self.set_status("Not a scene heading");
            return;
        }
        let base = Self::strip_scene_number_from_line(&self.lines[y]).to_string();
        let label = if let Some(t) = tag {
            t.to_string()
        } else {
            // Preserve existing non-integer custom tag
            let existing_custom: Option<String> = {
                let t = self.lines[y].trim_end();
                if t.ends_with('#') {
                    t[..t.len()-1].rfind('#').and_then(|o| {
                        let inner = &t[o+1..t.len()-1];
                        if !inner.is_empty() && !inner.contains(' ') && !inner.chars().all(|c| c.is_ascii_digit()) {
                            Some(inner.to_string())
                        } else { None }
                    })
                } else { None }
            };
            existing_custom.unwrap_or_else(|| self.compute_scene_number_for(y).to_string())
        };

        let new_line = format!("{} #{}#", base, label);
        if self.lines[y] != new_line {
            self.lines[y] = new_line;
            self.parse_document();
            self.set_status(&format!("Scene #{} injected", label));
        } else {
            self.set_status("Scene already numbered");
        }
    }

    /// Removes `#num#` tags from ALL scene headings. Always allowed, even with
    /// `production_lock` on.
    pub fn strip_all_scene_numbers(&mut self) {
        let mut changed = false;
        for i in 0..self.lines.len() {
            if self.types[i] != LineType::SceneHeading {
                continue;
            }
            let base = Self::strip_scene_number_from_line(&self.lines[i]);
            if self.lines[i].trim_end() != base {
                self.lines[i] = base.to_string();
                changed = true;
            }
        }
        if changed {
            self.parse_document();
            self.set_status("All scene numbers cleared");
        } else {
            self.set_status("No scene numbers found");
        }
    }

    pub fn update_layout(&mut self) {
        self.layout = build_layout(&self.lines, &self.types, self.cursor_y, &self.config);
    }

    // ── Selection Helpers ────────────────────────────────────────────────────

    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    /// Returns (start, end) in document order, where each is (line, char).
    pub fn selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        let anchor = self.selection_anchor?;
        let cursor = (self.cursor_y, self.cursor_x);
        if anchor <= cursor {
            Some((anchor, cursor))
        } else {
            Some((cursor, anchor))
        }
    }

    pub fn selected_text(&self) -> String {
        let Some(((sl, sc), (el, ec))) = self.selection_range() else {
            return String::new();
        };
        if sl == el {
            // Single line
            let line = &self.lines[sl];
            let chars: Vec<char> = line.chars().collect();
            let sc = sc.min(chars.len());
            let ec = ec.min(chars.len());
            chars[sc..ec].iter().collect()
        } else {
            let mut result = String::new();
            // First partial line
            let first: Vec<char> = self.lines[sl].chars().collect();
            let sc = sc.min(first.len());
            result.push_str(&first[sc..].iter().collect::<String>());
            result.push('\n');
            // Middle lines
            for li in (sl + 1)..el {
                result.push_str(&self.lines[li]);
                result.push('\n');
            }
            // Last partial line
            let last: Vec<char> = self.lines[el].chars().collect();
            let ec = ec.min(last.len());
            result.push_str(&last[..ec].iter().collect::<String>());
            result
        }
    }

    /// Delete the selected region and place cursor at selection start.
    /// Returns true if anything was deleted.
    pub fn delete_selection(&mut self) -> bool {
        let Some(((sl, sc), (el, ec))) = self.selection_range() else {
            return false;
        };
        self.selection_anchor = None;

        if sl == el {
            let chars: Vec<char> = self.lines[sl].chars().collect();
            let sc = sc.min(chars.len());
            let ec = ec.min(chars.len());
            let new_line: String = chars[..sc].iter().chain(chars[ec..].iter()).collect();
            self.lines[sl] = new_line;
        } else {
            let prefix: String = self.lines[sl].chars().take(sc).collect();
            let suffix: String = self.lines[el].chars().skip(ec).collect();
            self.lines[sl] = format!("{}{}", prefix, suffix);
            self.lines.drain((sl + 1)..=el);
            self.types.drain((sl + 1)..=el);
        }

        self.cursor_y = sl;
        self.cursor_x = sc;
        true
    }

    /// Select entire document.
    pub fn select_all(&mut self) {
        self.selection_anchor = Some((0, 0));
        let last_line = self.lines.len().saturating_sub(1);
        self.cursor_y = last_line;
        self.cursor_x = self.lines[last_line].chars().count();
    }

    pub fn copy_to_clipboard(&mut self) {
        let text = self.selected_text();
        if text.is_empty() {
            return;
        }
        match arboard::Clipboard::new() {
            Ok(mut cb) => {
                if let Err(e) = cb.set_text(text) {
                    self.set_status(&format!("Clipboard error: {}", e));
                } else {
                    self.set_status("Copied to clipboard");
                }
            }
            Err(e) => self.set_status(&format!("Clipboard unavailable: {}", e)),
        }
    }

    pub fn cut_to_clipboard(&mut self) -> bool {
        let text = self.selected_text();
        if text.is_empty() {
            return false;
        }
        match arboard::Clipboard::new() {
            Ok(mut cb) => {
                let _ = cb.set_text(text);
            }
            Err(_) => {}
        }
        self.delete_selection()
    }

    pub fn paste_from_clipboard(&mut self) {
        match arboard::Clipboard::new() {
            Ok(mut cb) => {
                match cb.get_text() {
                    Ok(text) => {
                        // If selection active, replace it first
                        if self.selection_anchor.is_some() {
                            self.delete_selection();
                        }
                        // Insert text at cursor, handling multi-line paste
                        let mut first = true;
                        for part in text.split('\n') {
                            if !first {
                                self.insert_newline(false);
                            }
                            for ch in part.chars() {
                                self.insert_char(ch);
                            }
                            first = false;
                        }
                    }
                    Err(e) => self.set_status(&format!("Paste error: {}", e)),
                }
            }
            Err(e) => self.set_status(&format!("Clipboard unavailable: {}", e)),
        }
    }

    /// Helper to save the current buffer to a new path.
    pub fn save_as(&mut self, path: PathBuf) -> io::Result<()> {
        let content = self.lines.join("\n");
        fs::write(&path, content)?;
        self.file = Some(path);
        self.dirty = false;
        self.set_status(&format!("Saved as {}", self.file.as_ref().unwrap().display()));
        Ok(())
    }

    /// Central dispatcher for the ":" command palette.
    /// Returns Ok(true) if the command should trigger an application exit.
    pub fn execute_command(
        &mut self,
        text_changed: &mut bool,
        cursor_moved: &mut bool,
        update_target_x: &mut bool,
    ) -> io::Result<bool> {
        let input = self.command_input.trim().to_string();
        self.command_input.clear();
        self.mode = AppMode::Normal;
        self.command_error = false;

        if input.is_empty() {
            return Ok(false);
        }

        // 1. Numeric jump (e.g. :50)
        if let Ok(line_num) = input.parse::<usize>() {
            self.cursor_y = (line_num.saturating_sub(1)).min(self.lines.len().saturating_sub(1));
            self.cursor_x = 0;
            *update_target_x = true;
            *cursor_moved = true;
            return Ok(false);
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts[0];
        let args = &parts[1..];

        match cmd {
            "w" => {
                if let Some(path_str) = args.get(0) {
                    self.save_as(PathBuf::from(path_str))?;
                } else if self.file.is_some() {
                    self.save()?;
                } else {
                    self.set_error("No filename. Use :w <file>");
                }
            }
            "q" => {
                if self.dirty {
                    self.set_error("Unsaved changes. Use :q! or :wq");
                } else if self.close_current_buffer() {
                    return Ok(true);
                }
            }
            "q!" => {
                if self.close_current_buffer() {
                    return Ok(true);
                }
            }
            "wq" => {
                if self.file.is_some() {
                    self.save()?;
                    if self.close_current_buffer() {
                        return Ok(true);
                    }
                } else if let Some(path_str) = args.get(0) {
                    self.save_as(PathBuf::from(path_str))?;
                    if self.close_current_buffer() {
                        return Ok(true);
                    }
                } else {
                    self.set_error("No filename");
                }
            }
            "set" => {
                if args.len() >= 2 {
                    let opt = args[0];
                    let val_str = args[1].to_lowercase();
                    let val = val_str == "on" || val_str == "true";
                    match opt {
                        "markup"       => self.config.hide_markup = !val,
                        "pagenums"     => self.config.show_page_numbers = val,
                        "scenenums"    => self.config.show_scene_numbers = val,
                        "contd"        => self.config.auto_contd = val,
                        "typewriter"   => self.config.strict_typewriter_mode = val,
                        "autosave"     => self.config.auto_save = val,
                        "autocomplete" => self.config.autocomplete = val,
                        "autobreaks"   => self.config.auto_paragraph_breaks = val,
                        "focus"        => self.config.focus_mode = val,
                        _ => self.set_error(&format!("Unknown option: {}", opt)),
                    }
                    *text_changed = true;
                } else if args.len() == 1 {
                    // Toggle syntax: :set focus
                    let opt = args[0];
                    match opt {
                        "markup"       => self.config.hide_markup = !self.config.hide_markup,
                        "pagenums"     => self.config.show_page_numbers = !self.config.show_page_numbers,
                        "scenenums"    => self.config.show_scene_numbers = !self.config.show_scene_numbers,
                        "contd"        => self.config.auto_contd = !self.config.auto_contd,
                        "typewriter"   => self.config.strict_typewriter_mode = !self.config.strict_typewriter_mode,
                        "autosave"     => self.config.auto_save = !self.config.auto_save,
                        "autocomplete" => self.config.autocomplete = !self.config.autocomplete,
                        "autobreaks"   => self.config.auto_paragraph_breaks = !self.config.auto_paragraph_breaks,
                        "focus"        => self.config.focus_mode = !self.config.focus_mode,
                        _ => self.set_error(&format!("Unknown option: {}", opt)),
                    }
                    *text_changed = true;
                } else {
                    self.set_error("Usage: /set <option> [on/off]");
                }
            }
            "u" | "undo" => {
                if self.undo() {
                    self.set_status("Undo applied");
                    *update_target_x = true;
                    *text_changed = true;
                    *cursor_moved = true;
                } else {
                    self.set_error("Nothing to undo");
                }
            }
            "redo" => {
                if self.redo() {
                    self.set_status("Redo applied");
                    *update_target_x = true;
                    *text_changed = true;
                    *cursor_moved = true;
                } else {
                    self.set_error("Nothing to redo");
                }
            }
            "pos" => {
                self.report_cursor_position();
            }
            // :injectnum  (auto) or  :injectnum14B  (custom tag)
            s if s.starts_with("injectnum") => {
                let tag_part = &s[9..];
                if tag_part.is_empty() {
                    self.inject_scene_number_tag(None);
                } else {
                    self.inject_scene_number_tag(Some(tag_part));
                }
                *text_changed = true;
            }
            // Jump to line number: /123
            s if s.chars().all(|c| c.is_ascii_digit()) => {
                if let Ok(line_num) = s.parse::<usize>() {
                    let target = line_num.saturating_sub(1);
                    self.cursor_y = target.min(self.lines.len().saturating_sub(1));
                    self.cursor_x = 0;
                    *cursor_moved = true;
                    *update_target_x = true;
                }
            }
            // Jump to scene number: /s50
            s if s.starts_with('s') && s[1..].chars().all(|c| c.is_ascii_digit()) => {
                let scene_num_str = &s[1..];
                if let Ok(num) = scene_num_str.parse::<usize>() {
                    if let Some(pos) = self.scenes.iter().position(|(_, _, s_num, _, _)| {
                        s_num.as_ref().map(|n: &String| n.trim_matches('#').parse::<usize>().unwrap_or(0)) == Some(num)
                    }) {
                        let line_idx = self.scenes[pos].0;
                        self.cursor_y = line_idx;
                        self.cursor_x = 0;
                        *cursor_moved = true;
                        *update_target_x = true;
                        self.set_status(&format!("Jumped to scene {}", num));
                    } else {
                        self.set_error(&format!("Scene {} not found", num));
                    }
                }
            }
            "renum" => {
                self.renumber_all_scenes();
                self.set_status("Scenes renumbered");
                *text_changed = true;
            }
            "clearnum" => {
                self.strip_all_scene_numbers();
                self.set_status("All the scene numbers are cleared now");
                *text_changed = true;
            }
            "locknum" => {
                self.config.production_lock = true;
                self.set_status("Production lock ENABLED");
            }
            "unlocknum" => {
                self.config.production_lock = false;
                self.set_status("Production lock DISABLED");
            }
            "search" => {
                if let Some(query) = args.get(0) {
                    self.search_query = query.to_string();
                    self.last_search = query.to_string();
                    self.show_search_highlight = true;
                    self.update_search_regex();
                    self.mode = AppMode::Search;
                    self.set_status(&format!("Searching: {}", query));
                } else {
                    self.search_query.clear();
                    self.show_search_highlight = true;
                    self.update_search_regex();
                    self.mode = AppMode::Search;
                }
            }
            "copy" => {
                self.copy_to_clipboard();
            }
            "cut" => {
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
            "paste" => {
                self.paste_from_clipboard();
                *update_target_x = true;
                *text_changed = true;
                *cursor_moved = true;
            }
            "selectall" => {
                self.select_all();
                *cursor_moved = true;
            }
            "home" => {
                self.mode = AppMode::Home;
                self.home_selected = 0;
            }
            "newfile" | "new" => {
                let new_buf = BufferState {
                    lines: vec![String::new()],
                    ..Default::default()
                };
                self.buffers.push(new_buf);
                let new_idx = self.buffers.len() - 1;
                self.has_multiple_buffers = true;
                self.switch_buffer(new_idx);
                self.set_status("New buffer opened");
                *text_changed = true;
                *cursor_moved = true;
            }
            _ => {
                self.set_error(&format!("Unknown command: /{}", cmd));
            }
        }

        Ok(false)
    }

    pub fn export_fountain(&self, path: &std::path::Path) -> std::io::Result<()> {
        let content = self.lines.join("\n");
        std::fs::write(path, content)
    }

    pub fn export_scene_csv(&self, path: &std::path::Path) -> std::io::Result<()> {
        let mut csv = String::new();
        csv.push_str("Scene Number,Int/Ext,Location,Time,Estimated Length (8ths)\n");

        let mut current_scene = None;
        let mut scene_lines = 0;
        let mut scenes_data = Vec::new();

        for row in &self.layout {
            if row.line_type == crate::types::LineType::SceneHeading {
                if let Some((s_num, heading)) = current_scene.take() {
                    scenes_data.push((s_num, heading, scene_lines));
                }
                
                let s_num = row.scene_num.clone().unwrap_or_default();
                let heading = crate::layout::strip_sigils(&row.raw_text, row.line_type).to_string();
                current_scene = Some((s_num, heading));
                scene_lines = 1;
            } else if current_scene.is_some() {
                scene_lines += 1;
            }
        }
        
        if let Some((s_num, heading)) = current_scene.take() {
            scenes_data.push((s_num, heading, scene_lines));
        }

        for (s_num, heading, visual_lines) in scenes_data {
            let eights_total = visual_lines as f32 / 7.0;
            let eights_rounded = eights_total.round() as usize;
            
            let full_pages = eights_rounded / 8;
            let remaining_eighths = eights_rounded % 8;
            
            let length_str = if full_pages > 0 && remaining_eighths > 0 {
                format!("{} {}/8", full_pages, remaining_eighths)
            } else if full_pages > 0 {
                format!("{}", full_pages)
            } else if remaining_eighths > 0 {
                format!("{}/8", remaining_eighths)
            } else {
                "1/8".to_string() 
            };

            let mut int_ext = String::new();
            let loc;
            let mut time = String::new();
            let h = heading.to_uppercase();
            if let Some((ie, rest)) = h.split_once('.') {
                int_ext = ie.trim().to_string();
                if let Some((l, t)) = rest.split_once('-') {
                    loc = l.trim().to_string();
                    time = t.trim().to_string();
                } else {
                    loc = rest.trim().to_string();
                }
            } else {
                loc = h;
            }

            csv.push_str(&format!("\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n", s_num, int_ext, loc, time, length_str));
        }

        std::fs::write(path, csv)
    }

    pub fn export_character_csv(&self, path: &std::path::Path) -> std::io::Result<()> {
        let mut csv = String::new();
        csv.push_str("Character,Dialogue Words,Scenes\n");

        let mut char_word_counts = std::collections::HashMap::new();
        let mut char_scenes = std::collections::HashMap::new();

        let mut current_scene = String::new();
        let mut current_char = String::new();

        for row in &self.layout {
            match row.line_type {
                crate::types::LineType::SceneHeading => {
                    if let Some(snum) = &row.scene_num {
                        current_scene = snum.clone();
                    } else {
                        current_scene = String::new();
                    }
                }
                crate::types::LineType::Character | crate::types::LineType::DualDialogueCharacter => {
                    let mut name = crate::layout::strip_sigils(&row.raw_text, row.line_type).trim().to_string();
                    if let Some(idx) = name.find('(') {
                        name = name[..idx].trim().to_string(); // Strip (V.O.) and (CONT'D)
                    }
                    current_char = name.to_uppercase();
                    if !current_scene.is_empty() {
                        let scenes: &mut std::collections::HashSet<String> = char_scenes.entry(current_char.clone()).or_default();
                        scenes.insert(current_scene.clone());
                    }
                }
                crate::types::LineType::Dialogue => {
                    let text = crate::layout::strip_sigils(&row.raw_text, row.line_type);
                    if !current_char.is_empty() {
                        let words = text.split_whitespace().count();
                        *char_word_counts.entry(current_char.clone()).or_insert(0) += words;
                    }
                }
                _ => {
                    if row.line_type != crate::types::LineType::Parenthetical {
                        current_char = String::new();
                    }
                }
            }
        }

        let mut sorted_chars: Vec<_> = char_word_counts.into_iter().collect();
        sorted_chars.sort_by(|a, b| b.1.cmp(&a.1));

        for (ch, words) in sorted_chars {
            let scenes = char_scenes.get(&ch).cloned().unwrap_or_default();
            let mut scene_list: Vec<_> = scenes.into_iter().collect();
            scene_list.sort();
            let scenes_str = scene_list.join(", ");
            csv.push_str(&format!("\"{}\",{},\"{}\"\n", ch, words, scenes_str));
        }

        std::fs::write(path, csv)
    }

    fn set_error(&mut self, msg: &str) {
        self.status_msg = Some(msg.to_string());
        self.command_error = true;
    }
}

pub mod ui;
pub mod editor;
pub mod input;

#[cfg(test)]
mod tests;
