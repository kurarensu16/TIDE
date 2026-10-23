//! Workspace file tree explorer with interactive directory expand/collapse.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub depth: usize,
    pub is_expanded: bool,
}

impl FileEntry {
    pub fn icon_styled(&self, theme: crate::icons::IconTheme) -> (&'static str, ratatui::style::Color) {
        if self.is_dir {
            crate::icons::folder_icon(self.is_expanded, theme)
        } else {
            crate::icons::file_icon(&self.name, theme)
        }
    }

    #[allow(dead_code)]
    pub fn icon(&self) -> &'static str {
        self.icon_styled(crate::icons::IconTheme::NerdFont).0
    }
}

pub struct FileExplorer {
    pub root: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected: usize,
    pub scroll: usize,
    pub expanded_paths: HashSet<PathBuf>,
}

impl FileExplorer {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        let root_buf = root.as_ref().to_path_buf();
        let mut expanded_paths = HashSet::new();

        // Expand 'src' by default for convenience
        let src_dir = root_buf.join("src");
        if src_dir.is_dir() {
            expanded_paths.insert(src_dir);
        }

        // Expand '.tide' and '.tide/worktrees' by default if present
        let tide_dir = root_buf.join(".tide");
        if tide_dir.is_dir() {
            expanded_paths.insert(tide_dir.clone());
            let wt_dir = tide_dir.join("worktrees");
            if wt_dir.is_dir() {
                expanded_paths.insert(wt_dir);
            }
        }

        let mut explorer = Self {
            root: root_buf,
            entries: Vec::new(),
            selected: 0,
            scroll: 0,
            expanded_paths,
        };
        explorer.refresh();
        explorer
    }

    pub fn refresh(&mut self) {
        let current_selected_path = self.entries.get(self.selected).map(|e| e.path.clone());
        self.entries.clear();
        let root = self.root.clone();
        self.scan_dir(&root, 0);

        // Restore selection to the same path if still visible, or clamp
        if let Some(target) = current_selected_path {
            if let Some(pos) = self.entries.iter().position(|e| e.path == target) {
                self.selected = pos;
            } else {
                self.selected = self.selected.min(self.entries.len().saturating_sub(1));
            }
        } else {
            self.selected = self.selected.min(self.entries.len().saturating_sub(1));
        }
    }

    fn scan_dir(&mut self, dir: &Path, depth: usize) {
        // Prevent infinite recursion loops with symlinks
        if depth > 20 {
            return;
        }
        if let Ok(read_dir) = fs::read_dir(dir) {
            let mut items: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
            // Sort directories first, then alphabetically
            items.sort_by_key(|e| (!e.path().is_dir(), e.file_name()));

            for entry in items {
                let path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();

                // Skip heavy directories and internal VCS metadata
                if file_name == "target" || file_name == ".git" || file_name == "node_modules" {
                    continue;
                }

                let is_dir = path.is_dir();
                let is_expanded = is_dir && self.expanded_paths.contains(&path);

                self.entries.push(FileEntry {
                    name: file_name,
                    path: path.clone(),
                    is_dir,
                    depth,
                    is_expanded,
                });

                // Only recurse into directories if they are currently expanded
                if is_expanded {
                    self.scan_dir(&path, depth + 1);
                }
            }
        }
    }

    pub fn effective_scroll(&self, view_height: usize) -> usize {
        if view_height == 0 || self.entries.is_empty() {
            return 0;
        }
        let mut scroll = self.scroll;
        if self.selected < scroll {
            scroll = self.selected;
        } else if self.selected >= scroll + view_height {
            scroll = self.selected + 1 - view_height;
        }
        let max_scroll = self.entries.len().saturating_sub(view_height);
        scroll.min(max_scroll)
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

    pub fn toggle_selected(&mut self) -> bool {
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.is_dir {
                let path = entry.path.clone();
                if self.expanded_paths.contains(&path) {
                    self.expanded_paths.remove(&path);
                } else {
                    self.expanded_paths.insert(path);
                }
                self.refresh();
                return true;
            }
        }
        false
    }

    pub fn expand_selected(&mut self) -> bool {
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.is_dir {
                let path = entry.path.clone();
                if !self.expanded_paths.contains(&path) {
                    self.expanded_paths.insert(path);
                    self.refresh();
                    return true;
                } else if self.selected + 1 < self.entries.len() && self.entries[self.selected + 1].depth > entry.depth {
                    self.selected += 1;
                    return true;
                }
            }
        }
        false
    }

    pub fn collapse_selected(&mut self) -> bool {
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.is_dir && self.expanded_paths.contains(&entry.path) {
                let path = entry.path.clone();
                self.expanded_paths.remove(&path);
                self.refresh();
                return true;
            } else if entry.depth > 0 {
                let current_depth = entry.depth;
                for i in (0..self.selected).rev() {
                    if self.entries[i].depth < current_depth && self.entries[i].is_dir {
                        self.selected = i;
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn reveal_path(&mut self, path: &Path) {
        let mut current = path.parent();
        let mut changed = false;
        while let Some(parent) = current {
            if parent == self.root {
                break;
            }
            if !self.expanded_paths.contains(parent) {
                self.expanded_paths.insert(parent.to_path_buf());
                changed = true;
            }
            current = parent.parent();
        }
        if changed {
            self.refresh();
        }
        if let Some(pos) = self.entries.iter().position(|e| e.path == path) {
            self.selected = pos;
        }
    }

    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.selected)
    }
}
