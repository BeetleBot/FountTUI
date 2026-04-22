use crate::app::{App, AppMode, EnsembleItem};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io;

impl App {
    pub fn handle_navigation(&mut self, key: KeyEvent, update_target_x: &mut bool, _text_changed: &mut bool, cursor_moved: &mut bool) -> io::Result<bool> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let _shift = key.modifiers.contains(KeyModifiers::SHIFT);
        match self.mode {
                AppMode::SceneNavigator => {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('h') => {
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
                        KeyCode::Enter | KeyCode::Char('l') => {
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
                        KeyCode::Esc | KeyCode::Char('h') => {
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
                        KeyCode::Enter | KeyCode::Char('l') => {
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
            _ => {}
        }
        Ok(false)
    }
}
