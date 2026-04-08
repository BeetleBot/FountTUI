use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "assets/structures/"]
struct Asset;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StructureStep {
    pub id: String,
    pub name: String,
    pub target: String,
    pub prompt: String,
    #[serde(default)]
    pub content: String,
    #[serde(default = "default_status")]
    pub status: String, // empty, draft, done
}

fn default_status() -> String {
    "empty".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoryStructure {
    pub id: String,
    pub name: String,
    pub mediums: Vec<String>,
    pub author: String,
    #[serde(rename = "type")]
    pub structure_type: String,
    pub complexity: String,
    pub description: String,
    pub best_for: String,
    pub avoid_if: String,
    pub steps: Vec<StructureStep>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlanningProject {
    pub id: String,
    pub structure_id: String,
    pub structure_name: String,
    pub steps: Vec<StructureStep>,
    pub title: String,
    pub file_path: Option<PathBuf>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

pub struct StructureRegistry {
    pub structures: HashMap<String, StoryStructure>,
}

impl StructureRegistry {
    pub fn new() -> Self {
        let mut registry = StructureRegistry {
            structures: HashMap::new(),
        };
        registry.load_all();
        registry
    }

    fn load_all(&mut self) {
        for file in Asset::iter() {
            if let Some(content) = Asset::get(file.as_ref()) {
                if let Ok(json_str) = std::str::from_utf8(content.data.as_ref()) {
                    if let Ok(structure) = serde_json::from_str::<StoryStructure>(json_str) {
                        // Only include screenplay structures (templates use lowercase "screenplay")
                        if structure.mediums.iter().any(|m| m.to_lowercase() == "screenplay") {
                            self.structures.insert(structure.id.clone(), structure);
                        }
                    }
                }
            }
        }
    }

    pub fn get_all(&self) -> Vec<StoryStructure> {
        let mut values: Vec<StoryStructure> = self.structures.values().cloned().collect();
        values.sort_by(|a, b| a.name.cmp(&b.name));
        values
    }
}

#[derive(Default)]
pub struct PlanningState {
    pub project: Option<PlanningProject>,
    pub selected_step_idx: usize,
    pub is_dirty: bool,
    pub registry: Option<StructureRegistry>,
}

impl PlanningState {
    pub fn new() -> Self {
        Self {
            registry: Some(StructureRegistry::new()),
            ..Default::default()
        }
    }
}
