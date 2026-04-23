#[derive(Clone, Debug)]
pub struct Shortcut {
    pub category: String,
    pub key: String,
    pub desc: String,
}

pub fn get_all_shortcuts() -> Vec<Shortcut> {
    let content = include_str!("../../assets/shortcuts.txt");
    let mut shortcuts = Vec::new();
    let mut current_category = "General".to_string();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            current_category = line.trim_start_matches('#').trim().to_string();
            continue;
        }

        let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        if parts.len() >= 2 {
            shortcuts.push(Shortcut {
                category: current_category.clone(),
                key: parts[0].to_string(),
                desc: parts[1].to_string(),
            });
        }
    }
    shortcuts
}
