//! Multi-tab inline code editor buffer management, selection, clipboard,
//! undo/redo history, auto-indentation, and bracket pair matching.

use std::path::{Path, PathBuf};

use crate::app::App;

#[derive(Clone, Debug)]
pub struct EditorTab {
    pub file_path: PathBuf,
    pub file_name: String,
    pub lines: Vec<String>,
    pub scroll: usize,
    pub cursor: (usize, usize),
    pub dirty: bool,
    pub is_pinned: bool,
    pub undo_stack: Vec<(Vec<String>, (usize, usize))>,
    pub redo_stack: Vec<(Vec<String>, (usize, usize))>,
    pub selection_anchor: Option<(usize, usize)>,
}

impl App {
    pub fn open_file(&mut self, path: &Path, pinned: bool) {
        // If already open in an existing tab, simply switch to it!
        if let Some(pos) = self.editor_tabs.iter().position(|t| t.file_path == path) {
            if pinned {
                self.editor_tabs[pos].is_pinned = true;
            }
            self.switch_editor_tab(pos);
            return;
        }

        // Save current tab state before pushing a new one
        self.sync_active_tab_from_state();

        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let lines = match std::fs::read_to_string(path) {
            Ok(content) => content.lines().map(|s| s.to_string()).collect(),
            Err(_) => {
                let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                vec![
                    format!("Binary or non-UTF8 file: {}", file_name),
                    format!("Size: {} bytes", size),
                    String::new(),
                    "Press Ctrl+E to open in external application (VS Code).".to_string(),
                ]
            }
        };

        let new_tab = EditorTab {
            file_path: path.to_path_buf(),
            file_name,
            lines,
            scroll: 0,
            cursor: (0, 0),
            dirty: false,
            is_pinned: pinned,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            selection_anchor: None,
        };

        // If previewing (not pinned) and current active tab is clean and unpinned, reuse it (VS Code preview tab behavior)
        if !pinned && !self.editor_tabs.is_empty() {
            let active = self.active_editor_tab_idx;
            if active < self.editor_tabs.len()
                && !self.editor_tabs[active].dirty
                && !self.editor_tabs[active].is_pinned
            {
                self.editor_tabs[active] = new_tab;
                self.sync_state_from_active_tab();
                self.ensure_editor_tab_visible(active);
                return;
            }
        }

        self.editor_tabs.push(new_tab);
        let new_idx = self.editor_tabs.len() - 1;
        self.active_editor_tab_idx = new_idx;
        self.sync_state_from_active_tab();
        self.ensure_editor_tab_visible(new_idx);
    }

    pub fn mark_editor_dirty(&mut self) {
        self.editor_dirty = true;
        if self.active_editor_tab_idx < self.editor_tabs.len() {
            self.editor_tabs[self.active_editor_tab_idx].dirty = true;
            self.editor_tabs[self.active_editor_tab_idx].is_pinned = true;
        }
    }

    pub fn sync_active_tab_from_state(&mut self) {
        if self.active_editor_tab_idx < self.editor_tabs.len() {
            if let Some((_, lines)) = &self.preview_content {
                self.editor_tabs[self.active_editor_tab_idx].lines = lines.clone();
            }
            self.editor_tabs[self.active_editor_tab_idx].scroll = self.preview_scroll;
            self.editor_tabs[self.active_editor_tab_idx].cursor = self.editor_cursor;
            self.editor_tabs[self.active_editor_tab_idx].dirty = self.editor_dirty;
            self.editor_tabs[self.active_editor_tab_idx].selection_anchor = self.editor_selection_anchor;
        }
    }

    pub fn sync_state_from_active_tab(&mut self) {
        if self.active_editor_tab_idx < self.editor_tabs.len() {
            let tab = &self.editor_tabs[self.active_editor_tab_idx];
            self.preview_content = Some((tab.file_name.clone(), tab.lines.clone()));
            self.editor_file_path = Some(tab.file_path.clone());
            self.preview_scroll = tab.scroll;
            self.editor_cursor = tab.cursor;
            self.editor_dirty = tab.dirty;
            self.editor_selection_anchor = tab.selection_anchor;
        } else {
            self.preview_content = None;
            self.editor_file_path = None;
            self.preview_scroll = 0;
            self.editor_cursor = (0, 0);
            self.editor_dirty = false;
            self.editor_selection_anchor = None;
        }
    }

    pub fn switch_editor_tab(&mut self, idx: usize) {
        if idx < self.editor_tabs.len() {
            self.sync_active_tab_from_state();
            self.active_editor_tab_idx = idx;
            self.sync_state_from_active_tab();
            self.ensure_editor_tab_visible(idx);
        }
    }

    pub fn close_editor_tab(&mut self, idx: usize) {
        if idx < self.editor_tabs.len() {
            let closed_name = self.editor_tabs[idx].file_name.clone();
            self.editor_tabs.remove(idx);
            if self.editor_tabs.is_empty() {
                self.active_editor_tab_idx = 0;
                self.preview_content = None;
                self.editor_file_path = None;
                self.preview_scroll = 0;
                self.editor_cursor = (0, 0);
                self.editor_dirty = false;
            } else {
                if self.active_editor_tab_idx >= self.editor_tabs.len() {
                    self.active_editor_tab_idx = self.editor_tabs.len() - 1;
                }
                self.sync_state_from_active_tab();
                let current_idx = self.active_editor_tab_idx;
                self.ensure_editor_tab_visible(current_idx);
            }
            self.status_message = Some((
                format!("Closed tab: {}", closed_name),
                std::time::Instant::now(),
            ));
        }
    }

    pub fn next_editor_tab(&mut self) {
        if !self.editor_tabs.is_empty() {
            let next = (self.active_editor_tab_idx + 1) % self.editor_tabs.len();
            self.switch_editor_tab(next);
        }
    }

    pub fn prev_editor_tab(&mut self) {
        if !self.editor_tabs.is_empty() {
            let prev = (self.active_editor_tab_idx + self.editor_tabs.len() - 1) % self.editor_tabs.len();
            self.switch_editor_tab(prev);
        }
    }

    pub fn ensure_editor_tab_visible(&mut self, idx: usize) {
        if idx < self.editor_tab_scroll {
            self.editor_tab_scroll = idx;
        } else if idx >= self.editor_tab_scroll + 4 {
            self.editor_tab_scroll = idx.saturating_sub(3);
        }
    }

    pub fn save_editor_content(&mut self) {
        self.sync_active_tab_from_state();
        if let (Some(path), Some((_, lines))) = (&self.editor_file_path, &self.preview_content) {
            let content = lines.join("\r\n");
            match std::fs::write(path, content) {
                Ok(_) => {
                    self.editor_dirty = false;
                    if self.active_editor_tab_idx < self.editor_tabs.len() {
                        self.editor_tabs[self.active_editor_tab_idx].dirty = false;
                    }
                    self.status_message = Some((
                        format!("Saved {}", path.file_name().unwrap_or_default().to_string_lossy()),
                        std::time::Instant::now(),
                    ));
                }
                Err(e) => {
                    self.status_message = Some((
                        format!("Failed to save: {}", e),
                        std::time::Instant::now(),
                    ));
                }
            }
        }
    }

    pub fn push_editor_undo_snapshot(&mut self) {
        if self.active_editor_tab_idx < self.editor_tabs.len() {
            if let Some((_, lines)) = &self.preview_content {
                let tab = &mut self.editor_tabs[self.active_editor_tab_idx];
                if let Some((last_lines, last_cursor)) = tab.undo_stack.last() {
                    if last_lines == lines && *last_cursor == self.editor_cursor {
                        return;
                    }
                }
                tab.undo_stack.push((lines.clone(), self.editor_cursor));
                if tab.undo_stack.len() > 100 {
                    tab.undo_stack.remove(0);
                }
                tab.redo_stack.clear();
            }
        }
    }

    pub fn editor_undo(&mut self) {
        if self.active_editor_tab_idx >= self.editor_tabs.len() {
            return;
        }
        let tab = &mut self.editor_tabs[self.active_editor_tab_idx];
        if let Some((prev_lines, prev_cursor)) = tab.undo_stack.pop() {
            if let Some((_, curr_lines)) = &self.preview_content {
                tab.redo_stack.push((curr_lines.clone(), self.editor_cursor));
                if tab.redo_stack.len() > 100 {
                    tab.redo_stack.remove(0);
                }
            }
            if let Some((_, lines)) = &mut self.preview_content {
                *lines = prev_lines.clone();
            }
            tab.lines = prev_lines;
            self.editor_cursor = prev_cursor;
            tab.cursor = prev_cursor;
            self.mark_editor_dirty();
            self.status_message = Some(("⤺ Undo".to_string(), std::time::Instant::now()));
        } else {
            self.status_message = Some(("Already at oldest change".to_string(), std::time::Instant::now()));
        }
    }

    pub fn editor_redo(&mut self) {
        if self.active_editor_tab_idx >= self.editor_tabs.len() {
            return;
        }
        let tab = &mut self.editor_tabs[self.active_editor_tab_idx];
        if let Some((next_lines, next_cursor)) = tab.redo_stack.pop() {
            if let Some((_, curr_lines)) = &self.preview_content {
                tab.undo_stack.push((curr_lines.clone(), self.editor_cursor));
                if tab.undo_stack.len() > 100 {
                    tab.undo_stack.remove(0);
                }
            }
            if let Some((_, lines)) = &mut self.preview_content {
                *lines = next_lines.clone();
            }
            tab.lines = next_lines;
            self.editor_cursor = next_cursor;
            tab.cursor = next_cursor;
            self.mark_editor_dirty();
            self.status_message = Some(("⤻ Redo".to_string(), std::time::Instant::now()));
        } else {
            self.status_message = Some(("Already at newest change".to_string(), std::time::Instant::now()));
        }
    }

    /// Inserts a newline at the cursor while preserving leading indentation and auto-indenting blocks.
    pub fn editor_newline_with_auto_indent(&mut self) {
        if self.editor_has_selection() {
            self.editor_delete_selection();
        } else {
            self.push_editor_undo_snapshot();
        }

        if let Some((_, lines)) = &mut self.preview_content {
            if lines.is_empty() {
                lines.push(String::new());
            }
            let (row, col) = self.editor_cursor;
            let row = row.min(lines.len().saturating_sub(1));
            if let Some(line) = lines.get_mut(row) {
                let base_indent: String = line.chars().take_while(|c| *c == ' ' || *c == '\t').collect();
                let byte_idx = line.char_indices().nth(col).map(|(i, _)| i).unwrap_or(line.len());
                let rest = line.split_off(byte_idx);
                let rest_trimmed = rest.trim_start();

                let before_trimmed = line.trim_end();
                let opens_block = before_trimmed.ends_with('{')
                    || before_trimmed.ends_with('(')
                    || before_trimmed.ends_with('[')
                    || before_trimmed.ends_with(':');

                let closes_immediately = (before_trimmed.ends_with('{') && rest_trimmed.starts_with('}'))
                    || (before_trimmed.ends_with('(') && rest_trimmed.starts_with(')'))
                    || (before_trimmed.ends_with('[') && rest_trimmed.starts_with(']'));

                if closes_immediately {
                    let inner_indent = format!("{}    ", base_indent);
                    let new_col = inner_indent.chars().count();
                    let closing_line = format!("{}{}", base_indent, rest_trimmed);
                    lines.insert(row + 1, inner_indent);
                    lines.insert(row + 2, closing_line);
                    self.editor_cursor = (row + 1, new_col);
                } else {
                    let next_indent = if opens_block {
                        format!("{}    ", base_indent)
                    } else {
                        base_indent
                    };
                    let new_col = next_indent.chars().count();
                    lines.insert(row + 1, format!("{}{}", next_indent, rest));
                    self.editor_cursor = (row + 1, new_col);
                }

                self.editor_dirty = true;
                let view_height = self.preview_area.height.saturating_sub(3) as usize;
                if self.editor_cursor.0 >= self.preview_scroll + view_height.max(1) {
                    self.preview_scroll += 1;
                }
            }
        }
        if self.editor_dirty {
            self.mark_editor_dirty();
        }
    }

    /// Outdents (dedents) the current line or active selection by up to 4 leading spaces.
    pub fn editor_outdent(&mut self) {
        if self.preview_content.is_none() {
            return;
        }
        self.push_editor_undo_snapshot();
        let bounds = self.editor_selection_bounds();
        if let Some((_, lines)) = &mut self.preview_content {
            if lines.is_empty() {
                return;
            }
            let (start_row, end_row) = if let Some(((sr, _), (er, _))) = bounds {
                (sr.min(lines.len() - 1), er.min(lines.len() - 1))
            } else {
                let r = self.editor_cursor.0.min(lines.len() - 1);
                (r, r)
            };

            let mut changed = false;
            for r in start_row..=end_row {
                let line = &mut lines[r];
                let mut remove_bytes = 0;
                let mut remove_chars = 0;
                for ch in line.chars().take(4) {
                    if ch == ' ' {
                        remove_bytes += 1;
                        remove_chars += 1;
                    } else if ch == '\t' && remove_chars == 0 {
                        remove_bytes += 1;
                        remove_chars += 1;
                        break;
                    } else {
                        break;
                    }
                }
                if remove_bytes > 0 {
                    line.drain(..remove_bytes);
                    changed = true;
                    if r == self.editor_cursor.0 {
                        self.editor_cursor.1 = self.editor_cursor.1.saturating_sub(remove_chars);
                    }
                }
            }
            if changed {
                self.mark_editor_dirty();
            }
        }
    }

    /// Finds the matching bracket position `((open_or_cursor_row, col), (match_row, col))` for the bracket at or before the cursor.
    pub fn editor_matching_bracket_pos(&self) -> Option<((usize, usize), (usize, usize))> {
        let (_, lines) = self.preview_content.as_ref()?;
        let (row, col) = self.editor_cursor;
        let line = lines.get(row)?;
        let chars: Vec<char> = line.chars().collect();

        let (target_col, ch) = if col < chars.len() && is_bracket(chars[col]) {
            (col, chars[col])
        } else if col > 0 && col <= chars.len() && is_bracket(chars[col - 1]) {
            (col - 1, chars[col - 1])
        } else {
            return None;
        };

        let (pair_ch, forward) = match ch {
            '(' => (')', true),
            '[' => (']', true),
            '{' => ('}', true),
            ')' => ('(', false),
            ']' => ('[', false),
            '}' => ('{', false),
            _ => return None,
        };

        let mut depth = 0i32;
        if forward {
            for (r, l) in lines.iter().enumerate().skip(row) {
                let start_c = if r == row { target_col } else { 0 };
                for (c, curr) in l.chars().enumerate().skip(start_c) {
                    if curr == ch {
                        depth += 1;
                    } else if curr == pair_ch {
                        depth -= 1;
                        if depth == 0 {
                            return Some(((row, target_col), (r, c)));
                        }
                    }
                }
            }
        } else {
            for r in (0..=row).rev() {
                let l_chars: Vec<char> = lines[r].chars().collect();
                let end_c = if r == row { target_col } else { l_chars.len().saturating_sub(1) };
                if l_chars.is_empty() {
                    continue;
                }
                for c in (0..=end_c.min(l_chars.len() - 1)).rev() {
                    let curr = l_chars[c];
                    if curr == ch {
                        depth += 1;
                    } else if curr == pair_ch {
                        depth -= 1;
                        if depth == 0 {
                            return Some(((row, target_col), (r, c)));
                        }
                    }
                }
            }
        }

        None
    }

    pub fn duplicate_current_line(&mut self) {
        if self.preview_content.is_none() {
            return;
        }
        self.push_editor_undo_snapshot();
        if let Some((_, lines)) = &mut self.preview_content {
            if lines.is_empty() {
                lines.push(String::new());
            }
            let row = self.editor_cursor.0.min(lines.len().saturating_sub(1));
            let line_to_dup = lines[row].clone();
            lines.insert(row + 1, line_to_dup);
            self.editor_cursor.0 += 1;
            self.mark_editor_dirty();
            self.status_message = Some(("Duplicated line (Ctrl+D)".to_string(), std::time::Instant::now()));
        }
    }

    pub fn move_current_line_up(&mut self) {
        let row = self.editor_cursor.0;
        if row == 0 {
            return;
        }
        let can_move = self.preview_content.as_ref().map(|(_, l)| row < l.len()).unwrap_or(false);
        if !can_move {
            return;
        }
        self.push_editor_undo_snapshot();
        if let Some((_, lines)) = &mut self.preview_content {
            lines.swap(row, row - 1);
            self.editor_cursor.0 -= 1;
            if self.editor_cursor.0 < self.preview_scroll {
                self.preview_scroll = self.editor_cursor.0;
            }
            self.mark_editor_dirty();
        }
    }

    pub fn move_current_line_down(&mut self) {
        let row = self.editor_cursor.0;
        let can_move = self.preview_content.as_ref().map(|(_, l)| row + 1 < l.len()).unwrap_or(false);
        if !can_move {
            return;
        }
        self.push_editor_undo_snapshot();
        if let Some((_, lines)) = &mut self.preview_content {
            lines.swap(row, row + 1);
            self.editor_cursor.0 += 1;
            let view_height = self.preview_area.height.saturating_sub(3) as usize;
            if self.editor_cursor.0 >= self.preview_scroll + view_height.max(1) {
                self.preview_scroll += 1;
            }
            self.mark_editor_dirty();
        }
    }

    pub fn delete_current_line(&mut self) {
        if self.preview_content.as_ref().map(|(_, l)| l.is_empty()).unwrap_or(true) {
            return;
        }
        self.push_editor_undo_snapshot();
        if let Some((_, lines)) = &mut self.preview_content {
            let row = self.editor_cursor.0.min(lines.len().saturating_sub(1));
            if lines.len() > 1 {
                lines.remove(row);
                if self.editor_cursor.0 >= lines.len() {
                    self.editor_cursor.0 = lines.len() - 1;
                }
            } else {
                lines[0].clear();
                self.editor_cursor.0 = 0;
            }
            let max_col = lines.get(self.editor_cursor.0).map(|l| l.chars().count()).unwrap_or(0);
            self.editor_cursor.1 = self.editor_cursor.1.min(max_col);
            self.mark_editor_dirty();
            self.status_message = Some(("Deleted line (Ctrl+K)".to_string(), std::time::Instant::now()));
        }
    }

    pub fn toggle_comment_on_current_line(&mut self) {
        if self.preview_content.as_ref().map(|(_, l)| l.is_empty()).unwrap_or(true) {
            return;
        }
        let ext = self
            .editor_file_path
            .as_ref()
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        let comment_prefix = match ext.as_str() {
            "py" | "sh" | "bash" | "zsh" | "ps1" | "toml" | "yaml" | "yml" | "dockerfile" => "# ",
            "sql" => "-- ",
            _ => "// ",
        };

        self.push_editor_undo_snapshot();
        let row = self.editor_cursor.0;
        if let Some((_, lines)) = &mut self.preview_content {
            let row = row.min(lines.len().saturating_sub(1));
            if let Some(line) = lines.get_mut(row) {
                let trimmed = line.trim_start();
                let indent_len = line.len() - trimmed.len();
                let indent = line[..indent_len].to_string();

                if trimmed.starts_with(comment_prefix) {
                    *line = format!("{}{}", indent, &trimmed[comment_prefix.len()..]);
                    self.editor_cursor.1 = self.editor_cursor.1.saturating_sub(comment_prefix.len());
                } else if trimmed.starts_with(comment_prefix.trim_end()) {
                    let p = comment_prefix.trim_end();
                    *line = format!("{}{}", indent, &trimmed[p.len()..]);
                    self.editor_cursor.1 = self.editor_cursor.1.saturating_sub(p.len());
                } else {
                    *line = format!("{}{}{}", indent, comment_prefix, trimmed);
                    self.editor_cursor.1 += comment_prefix.len();
                }
                self.mark_editor_dirty();
                self.status_message = Some(("Toggled comment (Ctrl+/)".to_string(), std::time::Instant::now()));
            }
        }
    }

    pub fn get_clipboard_text(&self) -> Option<String> {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if let Ok(text) = clipboard.get_text() {
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
        if !self.internal_clipboard.is_empty() {
            Some(self.internal_clipboard.clone())
        } else {
            None
        }
    }

    pub fn set_clipboard_text(&mut self, text: String) {
        self.internal_clipboard = text.clone();
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(text);
        }
    }

    pub fn editor_selection_bounds(&self) -> Option<((usize, usize), (usize, usize))> {
        let anchor = self.editor_selection_anchor?;
        let cursor = self.editor_cursor;
        if anchor == cursor {
            return None;
        }
        if anchor.0 < cursor.0 || (anchor.0 == cursor.0 && anchor.1 <= cursor.1) {
            Some((anchor, cursor))
        } else {
            Some((cursor, anchor))
        }
    }

    pub fn editor_has_selection(&self) -> bool {
        self.editor_selection_bounds().is_some()
    }

    pub fn editor_clear_selection(&mut self) {
        self.editor_selection_anchor = None;
    }

    pub fn editor_selected_text(&self) -> Option<String> {
        let ((start_row, start_col), (end_row, end_col)) = self.editor_selection_bounds()?;
        let (_, lines) = self.preview_content.as_ref()?;
        if start_row >= lines.len() {
            return None;
        }

        if start_row == end_row {
            let line = &lines[start_row];
            let start_byte = line.char_indices().nth(start_col).map(|(i, _)| i).unwrap_or(line.len());
            let end_byte = line.char_indices().nth(end_col).map(|(i, _)| i).unwrap_or(line.len());
            if start_byte <= end_byte && start_byte <= line.len() && end_byte <= line.len() {
                return Some(line[start_byte..end_byte].to_string());
            }
            return None;
        }

        let mut result = Vec::new();
        let first_line = &lines[start_row];
        let start_byte = first_line.char_indices().nth(start_col).map(|(i, _)| i).unwrap_or(first_line.len());
        result.push(first_line[start_byte..].to_string());

        for r in (start_row + 1)..end_row.min(lines.len()) {
            result.push(lines[r].clone());
        }

        if end_row < lines.len() {
            let last_line = &lines[end_row];
            let end_byte = last_line.char_indices().nth(end_col).map(|(i, _)| i).unwrap_or(last_line.len());
            result.push(last_line[..end_byte].to_string());
        }

        Some(result.join("\r\n"))
    }

    pub fn editor_delete_selection(&mut self) -> bool {
        let bounds = self.editor_selection_bounds();
        let ((start_row, start_col), (end_row, end_col)) = match bounds {
            Some(b) => b,
            None => return false,
        };

        if self.preview_content.is_none() {
            return false;
        }

        self.push_editor_undo_snapshot();
        let (_, lines) = match &mut self.preview_content {
            Some(c) => c,
            None => return false,
        };

        if start_row >= lines.len() {
            self.editor_selection_anchor = None;
            return false;
        }

        if start_row == end_row {
            let line = &mut lines[start_row];
            let start_byte = line.char_indices().nth(start_col).map(|(i, _)| i).unwrap_or(line.len());
            let end_byte = line.char_indices().nth(end_col).map(|(i, _)| i).unwrap_or(line.len());
            if start_byte <= end_byte && start_byte <= line.len() && end_byte <= line.len() {
                line.replace_range(start_byte..end_byte, "");
            }
        } else {
            let first_prefix = {
                let first_line = &lines[start_row];
                let byte = first_line.char_indices().nth(start_col).map(|(i, _)| i).unwrap_or(first_line.len());
                first_line[..byte].to_string()
            };
            let last_suffix = if end_row < lines.len() {
                let last_line = &lines[end_row];
                let byte = last_line.char_indices().nth(end_col).map(|(i, _)| i).unwrap_or(last_line.len());
                last_line[byte..].to_string()
            } else {
                String::new()
            };

            let end_drain = (end_row + 1).min(lines.len());
            if start_row + 1 < end_drain {
                lines.drain((start_row + 1)..end_drain);
            }

            lines[start_row] = format!("{}{}", first_prefix, last_suffix);
        }

        self.editor_cursor = (start_row, start_col);
        self.editor_selection_anchor = None;
        self.mark_editor_dirty();
        true
    }

    pub fn editor_copy(&mut self) {
        if let Some(selected) = self.editor_selected_text() {
            let len = selected.chars().count();
            self.set_clipboard_text(selected);
            self.status_message = Some((format!("Copied {} chars (Ctrl+C)", len), std::time::Instant::now()));
        } else if let Some((_, lines)) = &self.preview_content {
            if !lines.is_empty() {
                let row = self.editor_cursor.0.min(lines.len().saturating_sub(1));
                let line_with_nl = format!("{}\r\n", lines[row]);
                self.set_clipboard_text(line_with_nl);
                self.status_message = Some(("Copied current line (Ctrl+C)".to_string(), std::time::Instant::now()));
            }
        }
    }

    pub fn editor_cut(&mut self) {
        if self.editor_has_selection() {
            if let Some(selected) = self.editor_selected_text() {
                let len = selected.chars().count();
                self.set_clipboard_text(selected);
                self.editor_delete_selection();
                self.status_message = Some((format!("Cut {} chars (Ctrl+X)", len), std::time::Instant::now()));
            }
        } else if let Some((_, lines)) = &self.preview_content {
            if !lines.is_empty() {
                let row = self.editor_cursor.0.min(lines.len().saturating_sub(1));
                let line_with_nl = format!("{}\r\n", lines[row]);
                self.set_clipboard_text(line_with_nl);
                self.delete_current_line();
                self.status_message = Some(("Cut current line (Ctrl+X)".to_string(), std::time::Instant::now()));
            }
        }
    }

    pub fn editor_paste_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        if self.editor_has_selection() {
            self.editor_delete_selection();
        } else {
            self.push_editor_undo_snapshot();
        }

        if let Some((_, lines)) = &mut self.preview_content {
            if lines.is_empty() {
                lines.push(String::new());
            }

            let (row, col) = self.editor_cursor;
            let row = row.min(lines.len().saturating_sub(1));

            let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
            let paste_lines: Vec<&str> = normalized.split('\n').collect();

            if paste_lines.len() == 1 {
                let p = paste_lines[0];
                let line = &mut lines[row];
                let byte_idx = line.char_indices().nth(col).map(|(i, _)| i).unwrap_or(line.len());
                line.insert_str(byte_idx, p);
                self.editor_cursor.1 = col + p.chars().count();
            } else {
                let line = &mut lines[row];
                let byte_idx = line.char_indices().nth(col).map(|(i, _)| i).unwrap_or(line.len());
                let suffix = line.split_off(byte_idx);
                line.push_str(paste_lines[0]);

                let mut current_row = row;
                for p in &paste_lines[1..paste_lines.len() - 1] {
                    current_row += 1;
                    lines.insert(current_row, p.to_string());
                }

                current_row += 1;
                let last_prefix = paste_lines.last().copied().unwrap_or("");
                let last_chars = last_prefix.chars().count();
                lines.insert(current_row, format!("{}{}", last_prefix, suffix));

                self.editor_cursor = (current_row, last_chars);
            }

            self.editor_selection_anchor = None;
            self.mark_editor_dirty();

            let view_height = self.preview_area.height.saturating_sub(3) as usize;
            if self.editor_cursor.0 >= self.preview_scroll + view_height.max(1) {
                self.preview_scroll = self.editor_cursor.0.saturating_sub(view_height.saturating_sub(1));
            } else if self.editor_cursor.0 < self.preview_scroll {
                self.preview_scroll = self.editor_cursor.0;
            }
        }
    }

    pub fn editor_paste(&mut self) {
        if let Some(text) = self.get_clipboard_text() {
            let len = text.chars().count();
            self.editor_paste_text(&text);
            self.status_message = Some((format!("Pasted {} chars (Ctrl+V)", len), std::time::Instant::now()));
        } else {
            self.status_message = Some(("Clipboard is empty".to_string(), std::time::Instant::now()));
        }
    }

    pub fn editor_select_all(&mut self) {
        if let Some((_, lines)) = &self.preview_content {
            if lines.is_empty() {
                return;
            }
            let last_row = lines.len().saturating_sub(1);
            let last_col = lines[last_row].chars().count();
            self.editor_selection_anchor = Some((0, 0));
            self.editor_cursor = (last_row, last_col);
            self.status_message = Some(("Selected all (Ctrl+A)".to_string(), std::time::Instant::now()));
        }
    }
}

fn is_bracket(c: char) -> bool {
    matches!(c, '(' | ')' | '[' | ']' | '{' | '}')
}

