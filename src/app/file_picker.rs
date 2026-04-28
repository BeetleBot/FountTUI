use std::path::{Path, PathBuf};
use std::fs;
use crate::app::{App, FilePickerState, FilePickerAction, AppMode};
use ratatui::widgets::ListState;
use directories::UserDirs;

impl App {
    pub fn open_file_picker(&mut self, action: FilePickerAction, filter: Vec<String>, initial_filename: Option<String>) {
        let current_dir = self.file.as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| {
                UserDirs::new()
                    .map(|u| u.home_dir().to_path_buf())
                    .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
            });

        let items = get_dir_items(&current_dir);
        let mut list_state = ListState::default();
        if !items.is_empty() {
            list_state.select(Some(0));
        }
        
        self.file_picker = Some(FilePickerState {
            current_dir,
            items,
            list_state,
            action,
            filename_input: initial_filename.unwrap_or_default(),
            extension_filter: filter,
        });
        self.mode = AppMode::FilePicker;
    }

    pub fn file_picker_enter(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        let (selected_path, action, is_dir) = if let Some(ref mut state) = self.file_picker {
            let items_len = state.items.len();
            let selected_idx = state.list_state.selected().unwrap_or(0);
            
            if selected_idx < items_len {
                let p = state.items[selected_idx].clone();
                let is_d = p.is_dir();
                (Some(p), state.action.clone(), is_d)
            } else if (state.action == FilePickerAction::Save || state.action == FilePickerAction::ExportReport || state.action == FilePickerAction::ExportScript || state.action == FilePickerAction::ExportSprints) 
               && !state.filename_input.is_empty() {
                let p = state.current_dir.join(&state.filename_input);
                (Some(p), state.action.clone(), false)
            } else {
                (None, state.action.clone(), false)
            }
        } else {
            return Ok(false);
        };

        if let Some(path) = selected_path {
            if is_dir {
                if let Some(ref mut state) = self.file_picker {
                    state.current_dir = path;
                    state.items = get_dir_items(&state.current_dir);
                    state.list_state.select(Some(0));
                }
                Ok(false)
            } else {
                if (action == FilePickerAction::Save || action == FilePickerAction::ExportReport || action == FilePickerAction::ExportScript || action == FilePickerAction::ExportSprints)
                    && let Some(ref mut state) = self.file_picker {
                        let selected_idx = state.list_state.selected().unwrap_or(0);
                        // If we clicked a file in save mode, fill the input
                        if selected_idx < state.items.len() {
                            state.filename_input = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                            return Ok(false);
                        }
                    }
                self.handle_file_picker_choice(path)
            }
        } else {
            Ok(false)
        }
    }

    pub fn handle_file_picker_choice(&mut self, path: PathBuf) -> Result<bool, Box<dyn std::error::Error>> {
        let action = if let Some(ref s) = self.file_picker { s.action.clone() } else { return Ok(false); };
        self.file_picker = None;
        self.mode = AppMode::Normal;

        match action {
            FilePickerAction::Open => {
                let content = std::fs::read_to_string(&path)?;
                let lines: Vec<String> = content.replace('\t', "    ").lines().map(str::to_string).collect();
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
                self.parse_document();
                self.update_autocomplete();
                self.update_layout();
                let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                self.set_status(&format!("Opened: {}", name));
            }
            FilePickerAction::Save => {
                self.save_as(path)?;
            }
            FilePickerAction::ExportScript => {
                let result = match self.config.export_format.as_str() {
                    "fountain" => self.export_fountain(&path),
                    _ => self.export_pdf(&path),
                };
                match result {
                    Ok(_) => self.set_status(&format!("Exported to {}", path.display())),
                    Err(e) => self.set_status(&format!("Error exporting: {}", e)),
                }
            }
            FilePickerAction::ExportReport => {
                let result = match self.config.report_format.as_str() {
                    "csv_char" => self.export_character_csv(&path),
                    "csv_location" => self.export_location_csv(&path),
                    "csv_notes" => self.export_note_csv(&path),
                    "csv_breakdown" => self.export_breakdown_csv(&path),
                    "txt_dialogue" => self.export_dialogue_txt(&path),
                    _ => self.export_scene_csv(&path),
                };
                match result {
                    Ok(_) => self.set_status(&format!("Exported to {}", path.display())),
                    Err(e) => self.set_status(&format!("Error exporting: {}", e)),
                }
            }
            FilePickerAction::ExportSprints => {
                if let Err(e) = self.sprint_manager.export_csv(&path) {
                    self.set_error(&format!("Export failed: {}", e));
                } else {
                    self.set_status(&format!("Exported sprint data to {}", path.display()));
                }
            }
        }
        Ok(false)
    }
}

pub fn get_dir_items(path: &Path) -> Vec<PathBuf> {
    let mut items = Vec::new();
    
    // Add parent directory ".." if it exists
    if let Some(parent) = path.parent() {
        items.push(parent.to_path_buf());
    }

    if let Ok(entries) = fs::read_dir(path) {
        let mut entries_vec: Vec<_> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                !name.starts_with('.')
            })
            .collect();
        
        // Sort: directories first, then files
        entries_vec.sort_by(|a, b| {
            let a_is_dir = a.is_dir();
            let b_is_dir = b.is_dir();
            if a_is_dir && !b_is_dir {
                std::cmp::Ordering::Less
            } else if !a_is_dir && b_is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.file_name().cmp(&b.file_name())
            }
        });
        
        items.extend(entries_vec);
    }
    
    items
}
