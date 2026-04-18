use crate::app::App;
use crate::types::LineType;
use crate::formatting::StringCaseExt;

impl App {
    pub fn get_command_completions(&self) -> Vec<String> {
        let mut commands: Vec<String> = vec![
            "w", "ww", "q", "q!", "wq", "ex",
            "renum", "clearnum", "locknum", "unlocknum", "injectnum",
            "search", "export",
            "ud", "rd", "copy", "cut", "paste", "pos",
            "selectall", "home", "o", "bn", "bp", "new", "newfile", "addtitle",
            "snap", "sprint", "cancelsprint", "sprintstat", "xray",
            "set", "theme", "t"
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let set_options = [
            "markup", "pagenums", "scenenums", "contd", "typewriter", 
            "autosave", "autocomplete", "autobreaks", "focus"
        ];
        for opt in set_options {
            commands.push(format!("set {}", opt));
        }

        let themes = self.theme_manager.list_themes();
        for t in themes {
            commands.push(format!("theme {}", t));
            commands.push(format!("t {}", t));
        }

        commands
    }
}


impl crate::app::App {
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
}
