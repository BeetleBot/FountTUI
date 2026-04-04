use std::{collections::HashSet, fs, io, path::PathBuf};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use unicode_width::UnicodeWidthStr;

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};

use crate::{
    config::{Cli, Config},
    formatting::{RenderConfig, StringCaseExt, render_inline},
    layout::{VisualRow, build_layout, find_visual_cursor, strip_sigils},
    parser::Parser,
    types::{LineType, PAGE_WIDTH, base_style},
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

    pub sidebar_area: Rect,

    pub settings_area: Rect,

    pub navigator_state: ListState,
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
            sidebar_area: Rect::default(),
            settings_area: Rect::default(),
            navigator_state: ListState::default(),
        };

        let mut first_buf = std::mem::take(&mut app.buffers[0]);
        app.swap_buffer(&mut first_buf);

        app.parse_document();
        app.update_autocomplete();
        app.update_layout();
        app.target_visual_x = app.current_visual_x();
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

    pub fn update_layout(&mut self) {
        self.layout = build_layout(&self.lines, &self.types, self.cursor_y, &self.config);
    }

}

pub mod ui;
pub mod editor;
pub mod input;

#[cfg(test)]
mod tests;
