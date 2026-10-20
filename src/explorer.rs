//! Workspace file tree explorer.

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    #[allow(dead_code)]
    pub path: PathBuf,
    pub is_dir: bool,
    pub depth: usize,
}

impl FileEntry {
    pub fn icon(&self) -> &'static str {
        if self.is_dir {
            return "📁 ";
        }
        if self.name.ends_with(".rs") {
            "🦀 "
        } else if self.name.ends_with(".toml") {
            "⚙️ "
        } else if self.name.ends_with(".md") {
            "📝 "
        } else if self.name.ends_with(".png") || self.name.ends_with(".jpg") || self.name.ends_with(".ico") {
            "🖼️ "
        } else if self.name.ends_with(".json") {
            "📋 "
        } else if self.name.ends_with(".lock") {
            "🔒 "
        } else {
            "📄 "
        }
    }
}

pub struct FileExplorer {
    pub root: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected: usize,
}

impl FileExplorer {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        let mut explorer = Self {
            root: root.as_ref().to_path_buf(),
            entries: Vec::new(),
            selected: 0,
        };
        explorer.refresh();
        explorer
    }

    pub fn refresh(&mut self) {
        self.entries.clear();
        self.scan_dir(&self.root.clone(), 0);
    }

    fn scan_dir(&mut self, dir: &Path, depth: usize) {
        if depth > 3 {
            return;
        }
        if let Ok(read_dir) = fs::read_dir(dir) {
            let mut items: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
            items.sort_by_key(|e| (!e.path().is_dir(), e.file_name()));

            for entry in items {
                let path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();

                // Skip heavy directories and internal VCS metadata
                if file_name == "target" || file_name == ".git" {
                    continue;
                }

                let is_dir = path.is_dir();
                self.entries.push(FileEntry {
                    name: file_name,
                    path: path.clone(),
                    is_dir,
                    depth,
                });

                // Recurse into directories
                if is_dir && depth < 2 {
                    self.scan_dir(&path, depth + 1);
                }
            }
        }
    }

    pub fn next(&mut self) {
        if !self.entries.is_empty() {
            self.selected = (self.selected + 1).min(self.entries.len() - 1);
        }
    }

    pub fn prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    #[allow(dead_code)]
    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.selected)
    }
}
