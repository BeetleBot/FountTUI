use crate::app::{App, AppMode, planning::PlanningProject};
use std::fs;
use std::path::PathBuf;

impl App {
    pub fn save_planning_project(&mut self) {
        let project = match self.planning.project.as_mut() {
            Some(p) => p,
            _ => return,
        };

        if project.file_path.is_none() {
            // If no file path, we need to ask for one.
            // For now, we'll use a default path or open the file picker.
            let default_name = format!("{}.StoryStruct", project.title.replace(" ", "_"));
            self.open_file_picker(
                crate::app::FilePickerAction::Save,
                vec!["StoryStruct".to_string()],
                Some(default_name),
            );
            return;
        }

        let path = project.file_path.as_ref().unwrap();
        match serde_json::to_string_pretty(&project) {
            Ok(json) => {
                if fs::write(path, json).is_ok() {
                    let path_str = path.display().to_string();
                    self.planning.is_dirty = false;
                    self.set_status(&format!("Saved to {}", path_str));
                } else {
                    self.set_error("Failed to write to file.");
                }
            }
            Err(_) => self.set_error("Failed to serialize project."),
        }
    }

    pub fn import_planning_to_fountain(&mut self) {
        // 1. First save the .StoryStruct as requested
        self.save_planning_project();

        let project = match self.planning.project.as_ref() {
            Some(p) => p,
            _ => return,
        };

        // 2. Generate Fountain content
        let mut lines = Vec::new();
        lines.push(format!("Title: {}", project.title));
        lines.push("Author: BeetleBot".to_string());
        lines.push("".to_string());

        for step in &project.steps {
            lines.push(format!("# {}", step.name.to_uppercase()));
            if !step.content.trim().is_empty() {
                // Prepend = to every line of content for Fountain synopsis
                for content_line in step.content.lines() {
                    lines.push(format!("= {}", content_line));
                }
            }
            lines.push("".to_string());
        }

        // 3. Create a new buffer with this content
        let new_buf = crate::app::BufferState {
            lines,
            dirty: true,
            ..Default::default()
        };
        self.buffers.push(new_buf);
        let new_idx = self.buffers.len() - 1;
        self.has_multiple_buffers = self.buffers.len() > 1;
        self.switch_buffer(new_idx);
        
        // 4. Set mode to normal editor
        self.mode = AppMode::Normal;
        self.set_status("Imported to Screenplay! Happy writing.");
    }

    pub fn load_planning_project(&mut self, path: PathBuf) {
        match fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str::<PlanningProject>(&content) {
                    Ok(mut project) => {
                        project.file_path = Some(path.clone());
                        self.planning.project = Some(project);
                        self.mode = AppMode::PlanningStudio;
                        self.set_status(&format!("Loaded {}", path.display()));
                    }
                    Err(e) => self.set_error(&format!("Failed to parse StoryStruct: {}", e)),
                }
            }
            Err(e) => self.set_error(&format!("Failed to read file: {}", e)),
        }
    }
}
