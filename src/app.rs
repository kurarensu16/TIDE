//! Application state, keyboard dispatch, and event coordination.

use std::path::PathBuf;
use crossbeam_channel::{unbounded, Receiver, Sender};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::agents::AgentSession;
use crate::explorer::FileExplorer;
use crate::git::GitRepoInfo;
use crate::terminal::TerminalSession;
use crate::worktree::WorktreeInfo;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ActivePane {
    Explorer,
    FilePreview,
    Terminal,
    Git,
    Agents,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExplorerOpModal {
    None,
    NewItem {
        target_dir: PathBuf,
        input: String,
        is_folder: bool,
    },
    Rename {
        target_path: PathBuf,
        input: String,
        is_dir: bool,
    },
    DeleteConfirm {
        target_path: PathBuf,
        is_dir: bool,
        name: String,
    },
}

#[derive(Clone, Debug)]
pub enum PaletteAction {
    OpenFile(PathBuf),
    SpawnTerminal(String, String, Vec<String>),
    LaunchAgentModal,
    StopAgent,
    RefreshGit,
    RefreshExplorer,
    ToggleExplorer,
    TogglePreview,
    ToggleGit,
    ToggleAgents,
    MaximizeToggle,
    OpenInVSCode,
    SaveFile,
    UndoEdit,
    RedoEdit,
    DuplicateLine,
    DeleteLine,
    ToggleComment,
    CycleIconTheme,
    ShowFindInFile,
    ShowGlobalSearch,
    PromptNewItem,
    PromptRenameItem,
    PromptDeleteItem,
    GitStageAll,
    GitUnstageAll,
    GitCommitModal,
    GitBranchModal,
    GitStashModal,
    GitPush,
    GitPull,
    OpenThemeModal,
    SetColorTheme(crate::theme::ColorTheme),
    TerminalZoomIn,
    TerminalZoomOut,
    TerminalZoomReset,
    TerminalScrollUp,
    TerminalScrollDown,
    TerminalScrollReset,
    ToggleWelcomeScreen,
    Quit,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WelcomePromptModal {
    None,
    OpenFolder { input: String },
    CloneRepo { url_input: String, dest_input: String, active_field: usize },
    NewProject { name_input: String },
}

#[derive(Clone, Debug)]
pub struct WorkspaceEntry {
    pub name: String,
    pub path: PathBuf,
    pub kind: String,
    pub last_modified: Option<std::time::SystemTime>,
    pub git_branch: Option<String>,
}

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
}

#[derive(Clone, Debug)]
pub struct GrepResult {
    pub file_path: PathBuf,
    pub rel_path: String,
    pub line_num: usize,
    pub line_content: String,
    pub match_start: usize,
    #[allow(dead_code)]
    pub match_len: usize,
}

#[derive(Clone, Debug)]
pub struct PaletteItem {
    pub icon: String,
    pub title: String,
    pub subtitle: String,
    pub action: PaletteAction,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SplitterKind {
    LeftColumn,
    RightColumn,
    CenterRow,
    RightRow,
}

pub struct App {
    pub workspace_dir: PathBuf,
    pub workspace_name: String,
    pub active_pane: ActivePane,
    pub explorer: FileExplorer,
    pub git_info: GitRepoInfo,
    pub terminals: Vec<TerminalSession>,
    pub active_terminal_idx: usize,
    pub agents: Vec<AgentSession>,
    pub active_agent_idx: usize,
    pub editor_tabs: Vec<EditorTab>,
    pub active_editor_tab_idx: usize,
    pub editor_tab_scroll: usize,
    pub preview_tabs_area: ratatui::layout::Rect,
    pub preview_content: Option<(String, Vec<String>)>,
    pub preview_scroll: usize,
    pub editor_cursor: (usize, usize),
    pub editor_file_path: Option<PathBuf>,
    pub editor_dirty: bool,
    pub show_explorer: bool,
    pub show_preview: bool,
    pub show_git: bool,
    pub show_agents: bool,
    pub is_maximized: bool,
    pub left_col_pct: u16,
    pub right_col_pct: u16,
    pub center_split_pct: u16,
    pub right_split_pct: u16,
    pub active_splitter_drag: Option<SplitterKind>,
    pub body_area: ratatui::layout::Rect,
    pub explorer_area: ratatui::layout::Rect,
    pub explorer_footer_area: ratatui::layout::Rect,
    pub preview_area: ratatui::layout::Rect,
    pub terminal_area: ratatui::layout::Rect,
    pub terminal_tabs_area: ratatui::layout::Rect,
    pub git_area: ratatui::layout::Rect,
    pub agents_area: ratatui::layout::Rect,
    pub header_area: ratatui::layout::Rect,
    pub header_left_area: ratatui::layout::Rect,
    pub header_mid_area: ratatui::layout::Rect,
    pub header_right_area: ratatui::layout::Rect,
    pub dirty_tx: Sender<()>,
    pub dirty_rx: Receiver<()>,
    pub show_help: bool,
    pub show_command_palette: bool,
    pub palette_query: String,
    pub palette_selected_idx: usize,
    pub palette_modal_area: ratatui::layout::Rect,
    pub show_terminal_menu: bool,
    pub terminal_menu_selected_idx: usize,
    pub terminal_menu_modal_area: ratatui::layout::Rect,
    pub show_agent_launch_modal: bool,
    pub agent_launch_selected_idx: usize,
    pub agent_launch_modal_area: ratatui::layout::Rect,
    pub show_diff_modal: bool,
    pub diff_modal_content: Option<(String, String)>,
    pub status_message: Option<(String, std::time::Instant)>,
    pub should_quit: bool,
    pub show_find_in_file: bool,
    pub find_query: String,
    pub find_matches: Vec<(usize, usize, usize)>,
    pub active_find_match_idx: usize,
    pub find_bar_area: ratatui::layout::Rect,
    pub show_global_search: bool,
    pub global_search_query: String,
    pub global_search_results: Vec<GrepResult>,
    pub global_search_selected_idx: usize,
    pub global_search_modal_area: ratatui::layout::Rect,
    pub explorer_op_modal: ExplorerOpModal,
    pub explorer_op_modal_area: ratatui::layout::Rect,
    pub git_selected_idx: usize,
    pub git_scroll: usize,
    pub show_commit_modal: bool,
    pub commit_message_input: String,
    pub commit_modal_area: ratatui::layout::Rect,
    pub show_branch_modal: bool,
    pub branch_items: Vec<crate::git::GitBranchItem>,
    pub branch_selected_idx: usize,
    pub branch_filter_query: String,
    pub branch_is_creating: bool,
    pub branch_new_name: String,
    pub branch_modal_area: ratatui::layout::Rect,
    pub show_stash_modal: bool,
    pub stash_items: Vec<crate::git::GitStashItem>,
    pub stash_selected_idx: usize,
    pub stash_is_creating: bool,
    pub stash_new_message: String,
    pub stash_include_untracked: bool,
    pub stash_modal_area: ratatui::layout::Rect,
    pub diff_modal_scroll: usize,
    pub diff_modal_area: ratatui::layout::Rect,
    pub agents_scroll: usize,
    pub terminal_tab_scroll: usize,
    pub icon_theme: crate::icons::IconTheme,
    pub color_theme: crate::theme::ColorTheme,
    pub show_theme_modal: bool,
    pub theme_modal_selected_idx: usize,
    pub theme_modal_area: ratatui::layout::Rect,
    pub previous_color_theme: crate::theme::ColorTheme,
    pub terminal_zoom_level: i8,
    pub bg_notify_tx: Sender<(String, bool)>,
    pub bg_notify_rx: Receiver<(String, bool)>,
    pub show_welcome_screen: bool,
    pub detected_workspaces: Vec<WorkspaceEntry>,
    pub recent_workspaces: Vec<PathBuf>,
    pub welcome_selected_idx: usize,
    pub welcome_filter_query: String,
    pub welcome_prompt_modal: WelcomePromptModal,
    pub welcome_area: ratatui::layout::Rect,
}

impl App {
    pub fn new(workspace_dir: PathBuf) -> anyhow::Result<Self> {
        let (dirty_tx, dirty_rx) = unbounded();
        let (bg_notify_tx, bg_notify_rx) = unbounded();

        let is_home = Self::is_home_or_root_dir(&workspace_dir);
        let recent_workspaces = Self::load_recent_workspaces();
        let detected_workspaces = if is_home {
            Self::scan_workspaces(&workspace_dir)
        } else if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
            Self::scan_workspaces(&PathBuf::from(home))
        } else {
            Vec::new()
        };

        let workspace_name = workspace_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "workspace".to_string());

        let explorer = FileExplorer::new(&workspace_dir);
        let git_info = GitRepoInfo::fetch(&workspace_dir.to_string_lossy());

        let mut app = Self {
            workspace_dir: workspace_dir.clone(),
            workspace_name,
            active_pane: ActivePane::Explorer,
            explorer,
            git_info,
            terminals: Vec::new(),
            active_terminal_idx: 0,
            agents: Vec::new(),
            active_agent_idx: 0,
            preview_content: None,
            preview_scroll: 0,
            editor_cursor: (0, 0),
            editor_file_path: None,
            editor_dirty: false,
            show_explorer: true,
            show_preview: true,
            show_git: true,
            show_agents: true,
            is_maximized: false,
            left_col_pct: 20,
            right_col_pct: 25,
            center_split_pct: 55,
            right_split_pct: 45,
            active_splitter_drag: None,
            body_area: ratatui::layout::Rect::default(),
            explorer_area: ratatui::layout::Rect::default(),
            explorer_footer_area: ratatui::layout::Rect::default(),
            preview_area: ratatui::layout::Rect::default(),
            terminal_area: ratatui::layout::Rect::default(),
            terminal_tabs_area: ratatui::layout::Rect::default(),
            git_area: ratatui::layout::Rect::default(),
            agents_area: ratatui::layout::Rect::default(),
            header_area: ratatui::layout::Rect::default(),
            header_left_area: ratatui::layout::Rect::default(),
            header_mid_area: ratatui::layout::Rect::default(),
            header_right_area: ratatui::layout::Rect::default(),
            dirty_tx: dirty_tx.clone(),
            dirty_rx,
            bg_notify_tx,
            bg_notify_rx,
            show_help: false,
            show_command_palette: false,
            palette_query: String::new(),
            palette_selected_idx: 0,
            palette_modal_area: ratatui::layout::Rect::default(),
            show_terminal_menu: false,
            terminal_menu_selected_idx: 0,
            terminal_menu_modal_area: ratatui::layout::Rect::default(),
            show_agent_launch_modal: false,
            agent_launch_selected_idx: 0,
            agent_launch_modal_area: ratatui::layout::Rect::default(),
            show_diff_modal: false,
            diff_modal_content: None,
            status_message: None,
            should_quit: false,
            show_find_in_file: false,
            find_query: String::new(),
            find_matches: Vec::new(),
            active_find_match_idx: 0,
            find_bar_area: ratatui::layout::Rect::default(),
            show_global_search: false,
            global_search_query: String::new(),
            global_search_results: Vec::new(),
            global_search_selected_idx: 0,
            global_search_modal_area: ratatui::layout::Rect::default(),
            explorer_op_modal: ExplorerOpModal::None,
            explorer_op_modal_area: ratatui::layout::Rect::default(),
            git_selected_idx: 0,
            git_scroll: 0,
            show_commit_modal: false,
            commit_message_input: String::new(),
            commit_modal_area: ratatui::layout::Rect::default(),
            show_branch_modal: false,
            branch_items: Vec::new(),
            branch_selected_idx: 0,
            branch_filter_query: String::new(),
            branch_is_creating: false,
            branch_new_name: String::new(),
            branch_modal_area: ratatui::layout::Rect::default(),
            show_stash_modal: false,
            stash_items: Vec::new(),
            stash_selected_idx: 0,
            stash_is_creating: false,
            stash_new_message: String::new(),
            stash_include_untracked: true,
            stash_modal_area: ratatui::layout::Rect::default(),
            diff_modal_scroll: 0,
            diff_modal_area: ratatui::layout::Rect::default(),
            editor_tabs: Vec::new(),
            active_editor_tab_idx: 0,
            editor_tab_scroll: 0,
            preview_tabs_area: ratatui::layout::Rect::default(),
            agents_scroll: 0,
            terminal_tab_scroll: 0,
            icon_theme: crate::icons::IconTheme::NerdFont,
            color_theme: crate::theme::ColorTheme::TideOcean,
            show_theme_modal: false,
            theme_modal_selected_idx: 0,
            theme_modal_area: ratatui::layout::Rect::default(),
            previous_color_theme: crate::theme::ColorTheme::TideOcean,
            terminal_zoom_level: 0,
            show_welcome_screen: is_home,
            detected_workspaces,
            recent_workspaces,
            welcome_selected_idx: 0,
            welcome_filter_query: String::new(),
            welcome_prompt_modal: WelcomePromptModal::None,
            welcome_area: ratatui::layout::Rect::default(),
        };

        if !is_home {
            app.add_recent_workspace(&app.workspace_dir.clone());
        }

        app.update_file_preview();

        // Spawn initial shell session (PowerShell on Windows, Bash on Linux/macOS)
        #[cfg(windows)]
        let (shell_title, shell_cmd) = ("PowerShell", "powershell.exe");
        #[cfg(not(windows))]
        let (shell_title, shell_cmd) = ("Bash", "bash");

        app.spawn_terminal(shell_title, shell_cmd, &[])?;

        Ok(app)
    }

    pub fn theme(&self) -> crate::theme::ThemePalette {
        self.color_theme.palette()
    }

    pub fn zoom_in_terminal(&mut self) {
        self.terminal_zoom_level = (self.terminal_zoom_level + 1).min(5);
        let pct = 100 + (self.terminal_zoom_level as i16) * 10;
        self.status_message = Some((format!("Terminal Scale / Zoom: {}%", pct), std::time::Instant::now()));
    }

    pub fn zoom_out_terminal(&mut self) {
        self.terminal_zoom_level = (self.terminal_zoom_level - 1).max(-4);
        let pct = 100 + (self.terminal_zoom_level as i16) * 10;
        self.status_message = Some((format!("Terminal Scale / Zoom: {}%", pct), std::time::Instant::now()));
    }

    pub fn zoom_reset_terminal(&mut self) {
        self.terminal_zoom_level = 0;
        self.status_message = Some(("Terminal Scale / Zoom: 100% (Reset)".to_string(), std::time::Instant::now()));
    }

    pub fn update_file_preview(&mut self) {
        if let Some(entry) = self.explorer.selected_entry() {
            if !entry.is_dir {
                let path = entry.path.clone();
                self.open_file(&path, false);
            }
        }
    }

    pub fn prompt_new_item(&mut self) {
        let target_dir = if let Some(entry) = self.explorer.selected_entry() {
            if entry.is_dir {
                entry.path.clone()
            } else {
                entry.path.parent().unwrap_or(&self.workspace_dir).to_path_buf()
            }
        } else {
            self.workspace_dir.clone()
        };
        self.explorer_op_modal = ExplorerOpModal::NewItem {
            target_dir,
            input: String::new(),
            is_folder: false,
        };
    }

    pub fn prompt_rename_item(&mut self) {
        if let Some(entry) = self.explorer.selected_entry() {
            if entry.path == self.workspace_dir {
                self.status_message = Some((
                    "Cannot rename workspace root directory".to_string(),
                    std::time::Instant::now(),
                ));
                return;
            }
            self.explorer_op_modal = ExplorerOpModal::Rename {
                target_path: entry.path.clone(),
                input: entry.name.clone(),
                is_dir: entry.is_dir,
            };
        } else {
            self.status_message = Some((
                "No file or folder selected to rename".to_string(),
                std::time::Instant::now(),
            ));
        }
    }

    pub fn prompt_delete_item(&mut self) {
        if let Some(entry) = self.explorer.selected_entry() {
            if entry.path == self.workspace_dir {
                self.status_message = Some((
                    "Cannot delete workspace root directory".to_string(),
                    std::time::Instant::now(),
                ));
                return;
            }
            self.explorer_op_modal = ExplorerOpModal::DeleteConfirm {
                target_path: entry.path.clone(),
                is_dir: entry.is_dir,
                name: entry.name.clone(),
            };
        } else {
            self.status_message = Some((
                "No file or folder selected to delete".to_string(),
                std::time::Instant::now(),
            ));
        }
    }

    pub fn execute_new_item(&mut self) {
        let (target_dir, input, is_folder) = match &self.explorer_op_modal {
            ExplorerOpModal::NewItem { target_dir, input, is_folder } => (target_dir.clone(), input.clone(), *is_folder),
            _ => return,
        };
        self.explorer_op_modal = ExplorerOpModal::None;

        let input_trimmed = input.trim();
        if input_trimmed.is_empty() {
            return;
        }

        let will_be_folder = is_folder || input_trimmed.ends_with('/') || input_trimmed.ends_with('\\');
        let clean_name = input_trimmed.trim_matches(|c| c == '/' || c == '\\');
        let full_path = target_dir.join(clean_name);

        if full_path.exists() {
            self.status_message = Some((
                format!("Error: '{}' already exists", clean_name),
                std::time::Instant::now(),
            ));
            return;
        }

        if will_be_folder {
            match std::fs::create_dir_all(&full_path) {
                Ok(_) => {
                    self.explorer.expanded_paths.insert(target_dir);
                    self.explorer.refresh();
                    self.explorer.reveal_path(&full_path);
                    self.status_message = Some((
                        format!("Created folder '{}'", clean_name),
                        std::time::Instant::now(),
                    ));
                }
                Err(e) => {
                    self.status_message = Some((
                        format!("Failed to create folder: {}", e),
                        std::time::Instant::now(),
                    ));
                }
            }
        } else {
            if let Some(parent) = full_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::write(&full_path, b"") {
                Ok(_) => {
                    self.explorer.expanded_paths.insert(target_dir);
                    self.explorer.refresh();
                    self.explorer.reveal_path(&full_path);
                    self.open_file(&full_path, true);
                    self.active_pane = ActivePane::FilePreview;
                    self.status_message = Some((
                        format!("Created and opened '{}'", clean_name),
                        std::time::Instant::now(),
                    ));
                }
                Err(e) => {
                    self.status_message = Some((
                        format!("Failed to create file: {}", e),
                        std::time::Instant::now(),
                    ));
                }
            }
        }
    }

    pub fn execute_rename_item(&mut self) {
        let (target_path, input, is_dir) = match &self.explorer_op_modal {
            ExplorerOpModal::Rename { target_path, input, is_dir } => (target_path.clone(), input.clone(), *is_dir),
            _ => return,
        };
        self.explorer_op_modal = ExplorerOpModal::None;

        let input_trimmed = input.trim();
        let old_name = target_path.file_name().unwrap_or_default().to_string_lossy();
        if input_trimmed.is_empty() || input_trimmed == old_name {
            return;
        }

        let parent = target_path.parent().unwrap_or(&self.workspace_dir);
        let new_path = parent.join(input_trimmed);

        if new_path.exists() && new_path != target_path {
            self.status_message = Some((
                format!("Error: '{}' already exists", input_trimmed),
                std::time::Instant::now(),
            ));
            return;
        }

        match std::fs::rename(&target_path, &new_path) {
            Ok(_) => {
                let is_directory = is_dir;
                for tab in &mut self.editor_tabs {
                    if tab.file_path == target_path {
                        tab.file_path = new_path.clone();
                        tab.file_name = input_trimmed.to_string();
                    } else if is_directory && tab.file_path.starts_with(&target_path) {
                        if let Ok(rel) = tab.file_path.strip_prefix(&target_path) {
                            tab.file_path = new_path.join(rel);
                        }
                    }
                }
                if let Some((name, _)) = &mut self.preview_content {
                    if *name == target_path.to_string_lossy() {
                        *name = new_path.to_string_lossy().to_string();
                    }
                }
                if let Some(path) = &mut self.editor_file_path {
                    if *path == target_path {
                        *path = new_path.clone();
                    }
                }
                self.explorer.refresh();
                self.explorer.reveal_path(&new_path);
                self.status_message = Some((
                    format!("Renamed to '{}'", input_trimmed),
                    std::time::Instant::now(),
                ));
            }
            Err(e) => {
                self.status_message = Some((
                    format!("Rename failed: {}", e),
                    std::time::Instant::now(),
                ));
            }
        }
    }

    pub fn execute_delete_item(&mut self) {
        let (target_path, is_dir, name) = match &self.explorer_op_modal {
            ExplorerOpModal::DeleteConfirm { target_path, is_dir, name } => (target_path.clone(), *is_dir, name.clone()),
            _ => return,
        };
        self.explorer_op_modal = ExplorerOpModal::None;

        let res = if is_dir {
            std::fs::remove_dir_all(&target_path)
        } else {
            std::fs::remove_file(&target_path)
        };

        match res {
            Ok(_) => {
                let is_directory = is_dir;
                self.editor_tabs.retain(|tab| {
                    if is_directory {
                        !tab.file_path.starts_with(&target_path)
                    } else {
                        tab.file_path != target_path
                    }
                });

                if self.active_editor_tab_idx >= self.editor_tabs.len() {
                    self.active_editor_tab_idx = self.editor_tabs.len().saturating_sub(1);
                }

                if let Some(tab) = self.editor_tabs.get(self.active_editor_tab_idx).cloned() {
                    self.preview_content = Some((tab.file_name.clone(), tab.lines.clone()));
                    self.editor_cursor = tab.cursor;
                    self.editor_dirty = tab.dirty;
                    self.editor_file_path = Some(tab.file_path.clone());
                } else {
                    self.preview_content = None;
                    self.editor_file_path = None;
                    self.editor_dirty = false;
                }

                self.explorer.refresh();
                self.status_message = Some((
                    format!("Deleted '{}'", name),
                    std::time::Instant::now(),
                ));
            }
            Err(e) => {
                self.status_message = Some((
                    format!("Delete failed: {}", e),
                    std::time::Instant::now(),
                ));
            }
        }
    }

    pub fn open_file(&mut self, path: &std::path::Path, pinned: bool) {
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
        };

        // If previewing (not pinned) and current active tab is clean and unpinned, reuse it (VS Code preview tab behavior)
        if !pinned && !self.editor_tabs.is_empty() {
            let active = self.active_editor_tab_idx;
            if active < self.editor_tabs.len() && !self.editor_tabs[active].dirty && !self.editor_tabs[active].is_pinned {
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
        } else {
            self.preview_content = None;
            self.editor_file_path = None;
            self.preview_scroll = 0;
            self.editor_cursor = (0, 0);
            self.editor_dirty = false;
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
        let ext = self.editor_file_path
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

    pub fn open_in_external_editor(&mut self, path: &std::path::Path) {
        #[cfg(windows)]
        {
            // Execute cmd.exe /c code <path> to launch official VS Code CLI
            let child = std::process::Command::new("cmd.exe")
                .args(["/c", "code", &path.to_string_lossy()])
                .spawn();

            match child {
                Ok(_) => {
                    self.status_message = Some((
                        format!("Opened {} in VS Code", path.file_name().unwrap_or_default().to_string_lossy()),
                        std::time::Instant::now(),
                    ));
                }
                Err(_) => {
                    let _ = std::process::Command::new("notepad.exe")
                        .arg(path.to_string_lossy().to_string())
                        .spawn();
                    self.status_message = Some((
                        format!("Opened {} in Notepad", path.file_name().unwrap_or_default().to_string_lossy()),
                        std::time::Instant::now(),
                    ));
                }
            }
        }
        #[cfg(not(windows))]
        {
            let _ = std::process::Command::new("code")
                .arg(path.to_string_lossy().to_string())
                .spawn();
        }
    }

    pub fn update_find_matches(&mut self) {
        self.find_matches.clear();
        if self.find_query.is_empty() {
            self.active_find_match_idx = 0;
            return;
        }

        let q_lower = self.find_query.to_lowercase();
        let q_char_count = self.find_query.chars().count();

        if let Some((_, lines)) = &self.preview_content {
            for (row_idx, line) in lines.iter().enumerate() {
                let line_lower = line.to_lowercase();
                let mut start_byte = 0;
                while let Some(byte_pos) = line_lower[start_byte..].find(&q_lower) {
                    let actual_byte = start_byte + byte_pos;
                    let char_col = line[..actual_byte].chars().count();
                    self.find_matches.push((row_idx, char_col, q_char_count));
                    start_byte = actual_byte + q_lower.len().max(1);
                }
            }
        }

        if !self.find_matches.is_empty() {
            let (curr_row, curr_col) = self.editor_cursor;
            let closest = self.find_matches.iter().position(|&(r, c, _)| {
                r > curr_row || (r == curr_row && c >= curr_col)
            }).unwrap_or(0);
            self.active_find_match_idx = closest;
            self.jump_to_current_find_match();
        } else {
            self.active_find_match_idx = 0;
        }
    }

    pub fn next_find_match(&mut self) {
        if !self.find_matches.is_empty() {
            self.active_find_match_idx = (self.active_find_match_idx + 1) % self.find_matches.len();
            self.jump_to_current_find_match();
        }
    }

    pub fn prev_find_match(&mut self) {
        if !self.find_matches.is_empty() {
            self.active_find_match_idx = (self.active_find_match_idx + self.find_matches.len() - 1) % self.find_matches.len();
            self.jump_to_current_find_match();
        }
    }

    pub fn jump_to_current_find_match(&mut self) {
        if let Some(&(row, col, _)) = self.find_matches.get(self.active_find_match_idx) {
            self.editor_cursor = (row, col);
            let view_height = self.preview_area.height.saturating_sub(4) as usize;
            if view_height > 0 {
                if row < self.preview_scroll || row >= self.preview_scroll + view_height {
                    self.preview_scroll = row.saturating_sub(view_height / 2);
                }
            }
        }
    }

    pub fn run_global_search(&mut self) {
        self.global_search_results.clear();
        if self.global_search_query.trim().is_empty() {
            self.global_search_selected_idx = 0;
            return;
        }

        let q_lower = self.global_search_query.to_lowercase();
        let q_len = self.global_search_query.chars().count();
        let mut results = Vec::new();
        let max_results = 150;

        fn scan_dir(
            dir: &std::path::Path,
            root: &std::path::Path,
            q_lower: &str,
            q_len: usize,
            results: &mut Vec<GrepResult>,
            max_results: usize,
        ) {
            if results.len() >= max_results {
                return;
            }

            let entries = match std::fs::read_dir(dir) {
                Ok(e) => e,
                Err(_) => return,
            };

            for entry in entries.flatten() {
                if results.len() >= max_results {
                    break;
                }
                let path = entry.path();
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();

                if file_name.starts_with('.')
                    || file_name == "target"
                    || file_name == "node_modules"
                    || file_name == "dist"
                    || file_name == "build"
                {
                    continue;
                }

                if path.is_dir() {
                    scan_dir(&path, root, q_lower, q_len, results, max_results);
                } else if path.is_file() {
                    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    if size > 2 * 1024 * 1024 {
                        continue;
                    }

                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let rel_path = path
                            .strip_prefix(root)
                            .unwrap_or(&path)
                            .to_string_lossy()
                            .to_string();

                        for (line_idx, line) in content.lines().enumerate() {
                            let line_lower = line.to_lowercase();
                            if let Some(pos_byte) = line_lower.find(q_lower) {
                                let match_start = line[..pos_byte].chars().count();
                                results.push(GrepResult {
                                    file_path: path.clone(),
                                    rel_path: rel_path.clone(),
                                    line_num: line_idx + 1,
                                    line_content: line.trim().to_string(),
                                    match_start,
                                    match_len: q_len,
                                });
                                if results.len() >= max_results {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        scan_dir(&self.workspace_dir, &self.workspace_dir, &q_lower, q_len, &mut results, max_results);
        self.global_search_results = results;
        self.global_search_selected_idx = 0;
    }

    pub fn get_palette_items(&self) -> Vec<PaletteItem> {
        let q = self.palette_query.trim().to_lowercase();
        let is_command_mode = q.starts_with('>');

        let p_icon = |action: &str| crate::icons::action_icon(action, self.icon_theme).to_string();
        let s_icon = |shell: &str| crate::icons::shell_icon(shell, self.icon_theme).to_string();
        let pane_ic = |p: &str| crate::icons::pane_icon(p, self.icon_theme).to_string();

        if is_command_mode {
            let cmd_query = q.trim_start_matches('>').trim();
            let all_commands = vec![
                PaletteItem {
                    icon: s_icon("cmd"),
                    title: "Terminal: New Command Prompt (cmd.exe)".to_string(),
                    subtitle: "Spawn a native Windows Command Prompt shell".to_string(),
                    action: PaletteAction::SpawnTerminal("cmd.exe".to_string(), "cmd.exe".to_string(), vec![]),
                },
                PaletteItem {
                    icon: s_icon("powershell"),
                    title: "Terminal: New PowerShell Tab".to_string(),
                    subtitle: "Spawn a Windows PowerShell session".to_string(),
                    action: PaletteAction::SpawnTerminal("PowerShell".to_string(), "powershell.exe".to_string(), vec![]),
                },
                PaletteItem {
                    icon: s_icon("bash"),
                    title: "Terminal: New Git Bash Tab".to_string(),
                    subtitle: "Spawn a Git for Windows Bash shell".to_string(),
                    action: PaletteAction::SpawnTerminal("Git Bash".to_string(), "bash.exe".to_string(), vec![]),
                },
                PaletteItem {
                    icon: s_icon("wsl"),
                    title: "Terminal: New WSL / Ubuntu Tab".to_string(),
                    subtitle: "Spawn a Windows Subsystem for Linux shell".to_string(),
                    action: PaletteAction::SpawnTerminal("WSL".to_string(), "wsl.exe".to_string(), vec![]),
                },
                PaletteItem {
                    icon: p_icon("agent"),
                    title: "Agents: Launch New AI Agent".to_string(),
                    subtitle: "Start Antigravity, Claude Code, or isolated shell (Ctrl+A)".to_string(),
                    action: PaletteAction::LaunchAgentModal,
                },
                PaletteItem {
                    icon: p_icon("stop"),
                    title: "Agents: Stop Current Agent Session".to_string(),
                    subtitle: "Halt running process tree, keep worktree changes (s)".to_string(),
                    action: PaletteAction::StopAgent,
                },
                PaletteItem {
                    icon: pane_ic("explorer"),
                    title: "View: Toggle Explorer Panel".to_string(),
                    subtitle: "Show or hide file tree explorer (F2)".to_string(),
                    action: PaletteAction::ToggleExplorer,
                },
                PaletteItem {
                    icon: pane_ic("preview"),
                    title: "View: Toggle Code Preview Panel".to_string(),
                    subtitle: "Show or hide file editor / code preview (F5)".to_string(),
                    action: PaletteAction::TogglePreview,
                },
                PaletteItem {
                    icon: pane_ic("git"),
                    title: "View: Toggle Git Status Panel".to_string(),
                    subtitle: "Show or hide git commits and status (F3)".to_string(),
                    action: PaletteAction::ToggleGit,
                },
                PaletteItem {
                    icon: pane_ic("agents"),
                    title: "View: Toggle Agents Panel".to_string(),
                    subtitle: "Show or hide active agent sessions (F4)".to_string(),
                    action: PaletteAction::ToggleAgents,
                },
                PaletteItem {
                    icon: p_icon("maximize"),
                    title: "View: Maximize / Restore Active Pane".to_string(),
                    subtitle: "Expand current pane to full screen (F11)".to_string(),
                    action: PaletteAction::MaximizeToggle,
                },
                PaletteItem {
                    icon: p_icon("refresh"),
                    title: "Git: Refresh Repository Status".to_string(),
                    subtitle: "Reload working tree changes and commit log".to_string(),
                    action: PaletteAction::RefreshGit,
                },
                PaletteItem {
                    icon: p_icon("refresh"),
                    title: "Explorer: Refresh Directory Files".to_string(),
                    subtitle: "Rescan workspace folder tree".to_string(),
                    action: PaletteAction::RefreshExplorer,
                },
                PaletteItem {
                    icon: p_icon("new_item"),
                    title: "Explorer: New File or Folder".to_string(),
                    subtitle: "Create a new file or directory (press 'a' in Explorer)".to_string(),
                    action: PaletteAction::PromptNewItem,
                },
                PaletteItem {
                    icon: p_icon("rename"),
                    title: "Explorer: Rename Selected Item".to_string(),
                    subtitle: "Rename current file or folder (press 'r' in Explorer)".to_string(),
                    action: PaletteAction::PromptRenameItem,
                },
                PaletteItem {
                    icon: p_icon("delete"),
                    title: "Explorer: Delete Selected Item".to_string(),
                    subtitle: "Permanently delete current file or folder (press 'd' in Explorer)".to_string(),
                    action: PaletteAction::PromptDeleteItem,
                },
                PaletteItem {
                    icon: p_icon("vscode"),
                    title: "File: Open in External VS Code".to_string(),
                    subtitle: "Launch current file in Microsoft VS Code (Ctrl+E)".to_string(),
                    action: PaletteAction::OpenInVSCode,
                },
                PaletteItem {
                    icon: p_icon("save"),
                    title: "File: Save Current File Buffer".to_string(),
                    subtitle: "Write buffer changes to disk (Ctrl+S)".to_string(),
                    action: PaletteAction::SaveFile,
                },
                PaletteItem {
                    icon: p_icon("undo"),
                    title: "Edit: Undo Change".to_string(),
                    subtitle: "Revert last edit in active editor buffer (Ctrl+Z)".to_string(),
                    action: PaletteAction::UndoEdit,
                },
                PaletteItem {
                    icon: p_icon("redo"),
                    title: "Edit: Redo Change".to_string(),
                    subtitle: "Re-apply reverted edit in active editor buffer (Ctrl+Y / Ctrl+Shift+Z)".to_string(),
                    action: PaletteAction::RedoEdit,
                },
                PaletteItem {
                    icon: p_icon("duplicate"),
                    title: "Edit: Duplicate Current Line".to_string(),
                    subtitle: "Copy current line directly beneath cursor (Ctrl+D)".to_string(),
                    action: PaletteAction::DuplicateLine,
                },
                PaletteItem {
                    icon: p_icon("delete"),
                    title: "Edit: Delete Current Line".to_string(),
                    subtitle: "Remove entire current line (Ctrl+K)".to_string(),
                    action: PaletteAction::DeleteLine,
                },
                PaletteItem {
                    icon: p_icon("comment"),
                    title: "Edit: Toggle Line Comment".to_string(),
                    subtitle: "Toggle comment prefix for current line (Ctrl+/)".to_string(),
                    action: PaletteAction::ToggleComment,
                },
                PaletteItem {
                    icon: p_icon("search"),
                    title: "Find: Search in Current File".to_string(),
                    subtitle: "Find text within active editor tab (Ctrl+F)".to_string(),
                    action: PaletteAction::ShowFindInFile,
                },
                PaletteItem {
                    icon: p_icon("search"),
                    title: "Find: Search Across Workspace".to_string(),
                    subtitle: "Grep text across all project files (Ctrl+Shift+F)".to_string(),
                    action: PaletteAction::ShowGlobalSearch,
                },
                PaletteItem {
                    icon: pane_ic("wave"),
                    title: "Appearance: Toggle Icon Theme".to_string(),
                    subtitle: format!("Switch between Nerd Fonts, Emojis, and Unicode (Current: {})", self.icon_theme.name()),
                    action: PaletteAction::CycleIconTheme,
                },
                PaletteItem {
                    icon: p_icon("theme"),
                    title: "Theme: Switch Color Theme (Tokyo Night, Catppuccin, Nord, Gruvbox, Cyberpunk, Dracula, Rosé Pine, Monokai Pro, One Dark)".to_string(),
                    subtitle: format!("Live preview and customize workspace color palette (Current: {}, Key: F12)", self.color_theme.name()),
                    action: PaletteAction::OpenThemeModal,
                },
                PaletteItem {
                    icon: p_icon("theme"),
                    title: "Theme: Tokyo Night".to_string(),
                    subtitle: "Clean midnight blue storm aesthetic with soft neon lavender & sky blue".to_string(),
                    action: PaletteAction::SetColorTheme(crate::theme::ColorTheme::TokyoNight),
                },
                PaletteItem {
                    icon: p_icon("theme"),
                    title: "Theme: Catppuccin Mocha".to_string(),
                    subtitle: "Soothing, luxurious dark pastel palette with mauve, lavender, and peach".to_string(),
                    action: PaletteAction::SetColorTheme(crate::theme::ColorTheme::CatppuccinMocha),
                },
                PaletteItem {
                    icon: p_icon("theme"),
                    title: "Theme: Nord".to_string(),
                    subtitle: "Arctic, north-bluish cold aesthetic with frost blues and polar snow tones".to_string(),
                    action: PaletteAction::SetColorTheme(crate::theme::ColorTheme::Nord),
                },
                PaletteItem {
                    icon: p_icon("theme"),
                    title: "Theme: Gruvbox Dark".to_string(),
                    subtitle: "Warm retro groove palette with golden amber, forest aqua, and burnt orange".to_string(),
                    action: PaletteAction::SetColorTheme(crate::theme::ColorTheme::GruvboxDark),
                },
                PaletteItem {
                    icon: p_icon("theme"),
                    title: "Theme: Cyberpunk".to_string(),
                    subtitle: "High-voltage synthwave with neon fuchsia pink, electric cyan, and laser yellow".to_string(),
                    action: PaletteAction::SetColorTheme(crate::theme::ColorTheme::Cyberpunk),
                },
                PaletteItem {
                    icon: p_icon("theme"),
                    title: "Theme: TIDE Ocean".to_string(),
                    subtitle: "Default deep sapphire oceanic dark mode with cyan and electric wave accents".to_string(),
                    action: PaletteAction::SetColorTheme(crate::theme::ColorTheme::TideOcean),
                },
                PaletteItem {
                    icon: p_icon("theme"),
                    title: "Theme: Dracula".to_string(),
                    subtitle: "Gothic dark purple-charcoal with neon orchid pink, cyan, and lime accents".to_string(),
                    action: PaletteAction::SetColorTheme(crate::theme::ColorTheme::Dracula),
                },
                PaletteItem {
                    icon: p_icon("theme"),
                    title: "Theme: Rosé Pine".to_string(),
                    subtitle: "Alluring dark pine dusk palette with blush rose, lilac iris, and foam gold".to_string(),
                    action: PaletteAction::SetColorTheme(crate::theme::ColorTheme::RosePine),
                },
                PaletteItem {
                    icon: p_icon("theme"),
                    title: "Theme: Monokai Pro".to_string(),
                    subtitle: "Volcanic matte charcoal with sublime yellow, coral magenta, and mint green".to_string(),
                    action: PaletteAction::SetColorTheme(crate::theme::ColorTheme::MonokaiPro),
                },
                PaletteItem {
                    icon: p_icon("theme"),
                    title: "Theme: One Dark Pro".to_string(),
                    subtitle: "Clean neutral graphite slate with iconic chalk blue, purple, and sage green".to_string(),
                    action: PaletteAction::SetColorTheme(crate::theme::ColorTheme::OneDarkPro),
                },
                PaletteItem {
                    icon: p_icon("branch"),
                    title: "Git: Stage All Changes".to_string(),
                    subtitle: "Stage all modified and untracked files into index (git add -A)".to_string(),
                    action: PaletteAction::GitStageAll,
                },
                PaletteItem {
                    icon: p_icon("branch"),
                    title: "Git: Unstage All Changes".to_string(),
                    subtitle: "Reset staged files back to working tree (git restore --staged .)".to_string(),
                    action: PaletteAction::GitUnstageAll,
                },
                PaletteItem {
                    icon: p_icon("commit"),
                    title: "Git: Commit Staged Changes".to_string(),
                    subtitle: "Open commit prompt to record staged changes (git commit -m)".to_string(),
                    action: PaletteAction::GitCommitModal,
                },
                PaletteItem {
                    icon: p_icon("branch"),
                    title: "Git: Switch or Create Branch (b)".to_string(),
                    subtitle: "Checkout, create, search, and manage Git branches".to_string(),
                    action: PaletteAction::GitBranchModal,
                },
                PaletteItem {
                    icon: p_icon("stash"),
                    title: "Git: Stash Manager (z)".to_string(),
                    subtitle: "List, save, pop, apply, drop, and diff Git stashes".to_string(),
                    action: PaletteAction::GitStashModal,
                },
                PaletteItem {
                    icon: p_icon("push"),
                    title: "Git: Push to Remote".to_string(),
                    subtitle: "Push local commits to upstream tracking branch (git push)".to_string(),
                    action: PaletteAction::GitPush,
                },
                PaletteItem {
                    icon: p_icon("pull"),
                    title: "Git: Pull from Remote".to_string(),
                    subtitle: "Fetch and integrate remote changes (git pull)".to_string(),
                    action: PaletteAction::GitPull,
                },
                PaletteItem {
                    icon: p_icon("zoom"),
                    title: "Terminal: Zoom In / Enlarge (Ctrl+=)".to_string(),
                    subtitle: "Increase terminal display scale factor and viewport focus".to_string(),
                    action: PaletteAction::TerminalZoomIn,
                },
                PaletteItem {
                    icon: p_icon("zoom"),
                    title: "Terminal: Zoom Out / Shrink (Ctrl+-)".to_string(),
                    subtitle: "Decrease terminal display scale factor".to_string(),
                    action: PaletteAction::TerminalZoomOut,
                },
                PaletteItem {
                    icon: p_icon("zoom"),
                    title: "Terminal: Reset Zoom (Ctrl+0)".to_string(),
                    subtitle: "Reset terminal display scale to 100% default".to_string(),
                    action: PaletteAction::TerminalZoomReset,
                },
                PaletteItem {
                    icon: p_icon("scroll"),
                    title: "Terminal: Scroll Up Output History (Shift+PageUp)".to_string(),
                    subtitle: "Scroll back into previous command output buffer".to_string(),
                    action: PaletteAction::TerminalScrollUp,
                },
                PaletteItem {
                    icon: p_icon("scroll"),
                    title: "Terminal: Scroll Down Output History (Shift+PageDown)".to_string(),
                    subtitle: "Scroll toward live output buffer".to_string(),
                    action: PaletteAction::TerminalScrollDown,
                },
                PaletteItem {
                    icon: p_icon("scroll"),
                    title: "Terminal: Jump to Live View (Shift+End)".to_string(),
                    subtitle: "Reset scrollback offset to active interactive prompt".to_string(),
                    action: PaletteAction::TerminalScrollReset,
                },
                PaletteItem {
                    icon: pane_ic("explorer"),
                    title: "Workspace: Open Welcome & Workspace Launcher (<Ctrl-o>)".to_string(),
                    subtitle: "Browse detected projects, recent workspaces, open folder, or clone repo".to_string(),
                    action: PaletteAction::ToggleWelcomeScreen,
                },
                PaletteItem {
                    icon: p_icon("quit"),
                    title: "Quit TIDE".to_string(),
                    subtitle: "Exit workspace and terminate all Job Objects (Ctrl+Q)".to_string(),
                    action: PaletteAction::Quit,
                },
            ];

            if cmd_query.is_empty() {
                all_commands
            } else {
                all_commands
                    .into_iter()
                    .filter(|item| {
                        item.title.to_lowercase().contains(cmd_query)
                            || item.subtitle.to_lowercase().contains(cmd_query)
                    })
                    .collect()
            }
        } else {
            // File Search mode
            let mut file_items = Vec::new();
            for entry in &self.explorer.entries {
                if entry.is_dir {
                    continue;
                }
                let rel_path = entry
                    .path
                    .strip_prefix(&self.workspace_dir)
                    .unwrap_or(&entry.path)
                    .to_string_lossy()
                    .to_string();

                if q.is_empty()
                    || entry.name.to_lowercase().contains(&q)
                    || rel_path.to_lowercase().contains(&q)
                {
                    file_items.push(PaletteItem {
                        icon: entry.icon_styled(self.icon_theme).0.to_string(),
                        title: entry.name.clone(),
                        subtitle: rel_path,
                        action: PaletteAction::OpenFile(entry.path.clone()),
                    });
                }
            }

            // Also append matching commands if user typed something that matches commands
            if !q.is_empty() {
                if "cmd".contains(&q) || "command".contains(&q) {
                    file_items.push(PaletteItem {
                        icon: s_icon("cmd"),
                        title: "New Terminal: Command Prompt (cmd.exe)".to_string(),
                        subtitle: "Command: Spawn a Windows CMD shell".to_string(),
                        action: PaletteAction::SpawnTerminal("cmd.exe".to_string(), "cmd.exe".to_string(), vec![]),
                    });
                }
                if "terminal".contains(&q) || "shell".contains(&q) || "powershell".contains(&q) {
                    file_items.push(PaletteItem {
                        icon: s_icon("powershell"),
                        title: "New Terminal: PowerShell".to_string(),
                        subtitle: "Command: Spawn a PowerShell session".to_string(),
                        action: PaletteAction::SpawnTerminal("PowerShell".to_string(), "powershell.exe".to_string(), vec![]),
                    });
                }
                if "bash".contains(&q) || "git bash".contains(&q) {
                    file_items.push(PaletteItem {
                        icon: s_icon("bash"),
                        title: "New Terminal: Git Bash".to_string(),
                        subtitle: "Command: Spawn a Git Bash session".to_string(),
                        action: PaletteAction::SpawnTerminal("Git Bash".to_string(), "bash.exe".to_string(), vec![]),
                    });
                }
                if "agent".contains(&q) || "claude".contains(&q) || "agy".contains(&q) {
                    file_items.push(PaletteItem {
                        icon: p_icon("agent"),
                        title: "Launch Coding Agent Session".to_string(),
                        subtitle: "Command: Start Antigravity, Claude Code, or shell".to_string(),
                        action: PaletteAction::LaunchAgentModal,
                    });
                }
                if "git".contains(&q) {
                    file_items.push(PaletteItem {
                        icon: p_icon("branch"),
                        title: "Git: Refresh Status & Commits".to_string(),
                        subtitle: "Command: Fetch branch and commit log".to_string(),
                        action: PaletteAction::RefreshGit,
                    });
                }
            }

            file_items
        }
    }

    pub fn execute_palette_action(&mut self, action: PaletteAction) {
        match action {
            PaletteAction::OpenFile(path) => {
                self.open_file(&path, true);
                self.active_pane = ActivePane::FilePreview;
                self.explorer.reveal_path(&path);
                self.status_message = Some((
                    format!("Opened {}", path.file_name().unwrap_or_default().to_string_lossy()),
                    std::time::Instant::now(),
                ));
            }
            PaletteAction::SpawnTerminal(title, cmd, args) => {
                let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                let _ = self.spawn_terminal(&title, &cmd, &arg_refs);
                self.active_pane = ActivePane::Terminal;
            }
            PaletteAction::LaunchAgentModal => {
                self.show_agent_launch_modal = true;
            }
            PaletteAction::StopAgent => {
                let _ = self.stop_active_agent();
            }
            PaletteAction::RefreshGit => {
                self.git_info = GitRepoInfo::fetch(&self.workspace_dir.to_string_lossy());
                self.status_message = Some(("Refreshed Git status".to_string(), std::time::Instant::now()));
            }
            PaletteAction::RefreshExplorer => {
                self.explorer.refresh();
                self.update_file_preview();
                self.status_message = Some(("Refreshed Explorer".to_string(), std::time::Instant::now()));
            }
            PaletteAction::ToggleExplorer => {
                self.show_explorer = !self.show_explorer;
            }
            PaletteAction::TogglePreview => {
                self.show_preview = !self.show_preview;
            }
            PaletteAction::ToggleGit => {
                self.show_git = !self.show_git;
            }
            PaletteAction::ToggleAgents => {
                self.show_agents = !self.show_agents;
            }
            PaletteAction::MaximizeToggle => {
                self.is_maximized = !self.is_maximized;
            }
            PaletteAction::OpenInVSCode => {
                if let Some(path) = self.editor_file_path.clone() {
                    self.open_in_external_editor(&path);
                }
            }
            PaletteAction::SaveFile => {
                self.save_editor_content();
            }
            PaletteAction::UndoEdit => {
                self.editor_undo();
                self.active_pane = ActivePane::FilePreview;
            }
            PaletteAction::RedoEdit => {
                self.editor_redo();
                self.active_pane = ActivePane::FilePreview;
            }
            PaletteAction::DuplicateLine => {
                self.duplicate_current_line();
                self.active_pane = ActivePane::FilePreview;
            }
            PaletteAction::DeleteLine => {
                self.delete_current_line();
                self.active_pane = ActivePane::FilePreview;
            }
            PaletteAction::ToggleComment => {
                self.toggle_comment_on_current_line();
                self.active_pane = ActivePane::FilePreview;
            }
            PaletteAction::CycleIconTheme => {
                self.icon_theme = self.icon_theme.next();
                self.status_message = Some((
                    format!("Icon Theme switched to: {}", self.icon_theme.name()),
                    std::time::Instant::now(),
                ));
            }
            PaletteAction::ShowFindInFile => {
                self.show_find_in_file = true;
                self.active_pane = ActivePane::FilePreview;
                self.update_find_matches();
            }
            PaletteAction::ShowGlobalSearch => {
                self.show_global_search = true;
                self.global_search_query.clear();
                self.global_search_results.clear();
                self.global_search_selected_idx = 0;
            }
            PaletteAction::PromptNewItem => {
                self.prompt_new_item();
            }
            PaletteAction::PromptRenameItem => {
                self.prompt_rename_item();
            }
            PaletteAction::PromptDeleteItem => {
                self.prompt_delete_item();
            }
            PaletteAction::GitStageAll => {
                self.stage_all_git_items();
            }
            PaletteAction::GitUnstageAll => {
                self.unstage_all_git_items();
            }
            PaletteAction::GitCommitModal => {
                self.show_commit_modal = true;
                self.commit_message_input.clear();
            }
            PaletteAction::GitBranchModal => {
                self.open_branch_modal();
            }
            PaletteAction::GitStashModal => {
                self.open_stash_modal();
            }
            PaletteAction::GitPush => {
                self.push_git();
            }
            PaletteAction::GitPull => {
                self.pull_git();
            }
            PaletteAction::OpenThemeModal => {
                self.show_theme_modal = true;
                self.previous_color_theme = self.color_theme;
                self.theme_modal_selected_idx = crate::theme::ColorTheme::ALL
                    .iter()
                    .position(|t| *t == self.color_theme)
                    .unwrap_or(0);
            }
            PaletteAction::SetColorTheme(t) => {
                self.color_theme = t;
                self.status_message = Some((
                    format!("Theme switched to: {}", t.name()),
                    std::time::Instant::now(),
                ));
            }
            PaletteAction::TerminalZoomIn => {
                self.zoom_in_terminal();
            }
            PaletteAction::TerminalZoomOut => {
                self.zoom_out_terminal();
            }
            PaletteAction::TerminalZoomReset => {
                self.zoom_reset_terminal();
            }
            PaletteAction::TerminalScrollUp => {
                if let Some(session) = self.terminals.get(self.active_terminal_idx) {
                    session.scroll_up(15);
                }
            }
            PaletteAction::TerminalScrollDown => {
                if let Some(session) = self.terminals.get(self.active_terminal_idx) {
                    session.scroll_down(15);
                }
            }
            PaletteAction::TerminalScrollReset => {
                if let Some(session) = self.terminals.get(self.active_terminal_idx) {
                    session.scroll_reset();
                }
            }
            PaletteAction::ToggleWelcomeScreen => {
                self.toggle_welcome_screen();
            }
            PaletteAction::Quit => {
                self.should_quit = true;
            }
        }
    }

    pub fn spawn_selected_terminal(&mut self, idx: usize) -> anyhow::Result<()> {
        #[cfg(windows)]
        let res = match idx {
            0 => self.spawn_terminal("cmd.exe", "cmd.exe", &[]),
            1 => self.spawn_terminal("PowerShell", "powershell.exe", &[]),
            2 => self.spawn_terminal("Git Bash", "bash.exe", &[]),
            3 => self.spawn_terminal("WSL", "wsl.exe", &[]),
            _ => self.spawn_terminal("PowerShell", "powershell.exe", &[]),
        };
        #[cfg(not(windows))]
        let res = match idx {
            0 => self.spawn_terminal("Bash", "bash", &[]),
            1 => self.spawn_terminal("Sh", "sh", &[]),
            2 => self.spawn_terminal("Zsh", "zsh", &[]),
            _ => self.spawn_terminal("Bash", "bash", &[]),
        };
        self.active_pane = ActivePane::Terminal;
        res
    }

    pub fn spawn_terminal(&mut self, title: &str, cmd: &str, args: &[&str]) -> anyhow::Result<()> {
        let dir = self.workspace_dir.to_string_lossy().to_string();
        self.spawn_terminal_in_dir(title, cmd, args, Some(&dir))
    }

    pub fn spawn_terminal_in_dir(&mut self, title: &str, cmd: &str, args: &[&str], dir: Option<&str>) -> anyhow::Result<()> {
        let id = format!("term-{}", self.terminals.len() + 1);
        let session = TerminalSession::spawn(
            &id,
            title,
            cmd,
            args,
            dir,
            24,
            80,
            self.dirty_tx.clone(),
        )?;
        self.terminals.push(session);
        self.active_terminal_idx = self.terminals.len() - 1;
        Ok(())
    }

    pub fn launch_agent(&mut self, agent_name: &str, cmd: &str, args: &[&str], task: &str) -> anyhow::Result<()> {
        let session_id = format!("{}-{}", agent_name.to_lowercase().replace(' ', "-"), self.agents.len() + 1);
        let base_branch = if self.git_info.branch.is_empty() { "main" } else { &self.git_info.branch };

        // 1. Create Git Worktree
        let wt = match WorktreeInfo::create(&self.workspace_dir, &session_id, base_branch) {
            Ok(w) => w,
            Err(e) => {
                let err_str = format!("Failed to create worktree: {}", e);
                self.status_message = Some((err_str.clone(), std::time::Instant::now()));
                return Err(anyhow::anyhow!(err_str));
            }
        };

        // 2. Spawn Terminal in the isolated worktree directory
        let title = format!("{} ({})", agent_name, session_id);
        let wt_path = wt.path.to_string_lossy().to_string();
        if let Err(e) = self.spawn_terminal_in_dir(&title, cmd, args, Some(&wt_path)) {
            let err_str = format!("Failed to spawn command '{}': {}", cmd, e);
            self.status_message = Some((err_str.clone(), std::time::Instant::now()));
            let _ = wt.cleanup(&self.workspace_dir);
            return Err(anyhow::anyhow!(err_str));
        }
        let term_idx = self.terminals.len() - 1;

        // 3. Register AgentSession
        let branch_name = wt.branch.clone();
        let mut session = AgentSession::new(&session_id, agent_name, task, &branch_name);
        session.worktree = Some(wt);
        session.terminal_idx = Some(term_idx);
        self.agents.push(session);
        self.active_agent_idx = self.agents.len() - 1;

        self.status_message = Some((
            format!("Agent '{}' running on branch {}", agent_name, branch_name),
            std::time::Instant::now(),
        ));
        self.active_pane = ActivePane::Terminal;
        Ok(())
    }

    /// Automatically updates agent status and confidence based on PTY process lifecycle and idle heuristics.
    pub fn update_agent_heuristics(&mut self) {
        for agent in &mut self.agents {
            if let Some(t_idx) = agent.terminal_idx {
                if let Some(term) = self.terminals.get(t_idx) {
                    let is_alive = term.is_alive.lock().map(|a| *a).unwrap_or(true);
                    if !is_alive {
                        agent.state = crate::agents::AgentState::Completed;
                        agent.confidence = crate::agents::Confidence::Confirmed;
                    } else {
                        let elapsed_activity = term.last_activity.lock()
                            .map(|t| t.elapsed().as_secs())
                            .unwrap_or(0);
                        if elapsed_activity > 4 {
                            agent.state = crate::agents::AgentState::WaitingForInput;
                            agent.confidence = crate::agents::Confidence::Inferred;
                        } else {
                            agent.state = crate::agents::AgentState::Running;
                            agent.confidence = crate::agents::Confidence::Confirmed;
                        }
                    }
                }
            }
        }
    }

    pub fn inspect_agent_diff(&mut self) {
        if let Some(agent) = self.agents.get(self.active_agent_idx) {
            if let Some(wt) = &agent.worktree {
                if let Ok((_changed, diff_text)) = wt.get_diff(&self.workspace_dir) {
                    self.diff_modal_content = Some((format!("Diff: {} ({})", agent.name, wt.branch), diff_text));
                    self.show_diff_modal = true;
                    return;
                }
            }
        }
        self.diff_modal_content = Some(("Agent Diff".to_string(), "No isolated worktree associated with this agent session or no changes made yet.".to_string()));
        self.show_diff_modal = true;
    }

    pub fn merge_active_agent(&mut self) -> anyhow::Result<()> {
        if let Some(agent) = self.agents.get(self.active_agent_idx) {
            if let Some(wt) = &agent.worktree {
                wt.merge_into_base(&self.workspace_dir)?;
                self.git_info = GitRepoInfo::fetch(&self.workspace_dir.to_string_lossy());
            }
        }
        self.show_diff_modal = false;
        Ok(())
    }

    pub fn discard_active_agent(&mut self) -> anyhow::Result<()> {
        if self.active_agent_idx < self.agents.len() {
            let agent = self.agents.remove(self.active_agent_idx);
            if let Some(wt) = &agent.worktree {
                let _ = wt.cleanup(&self.workspace_dir);
            }
            if let Some(t_idx) = agent.terminal_idx {
                if t_idx < self.terminals.len() {
                    self.terminals.remove(t_idx);
                    // Update remaining agent terminal indexes
                    for other_agent in &mut self.agents {
                        if let Some(ref mut idx) = other_agent.terminal_idx {
                            if *idx > t_idx {
                                *idx -= 1;
                            }
                        }
                    }
                    if self.active_terminal_idx >= self.terminals.len() && !self.terminals.is_empty() {
                        self.active_terminal_idx = self.terminals.len() - 1;
                    }
                }
            }
            if self.active_agent_idx >= self.agents.len() && !self.agents.is_empty() {
                self.active_agent_idx = self.agents.len() - 1;
            }
            self.status_message = Some((
                format!("Agent session '{}' discarded and cleaned up.", agent.name),
                std::time::Instant::now(),
            ));
        }
        self.show_diff_modal = false;
        Ok(())
    }

    /// Stops the currently selected agent process gracefully without deleting worktree files.
    pub fn stop_active_agent(&mut self) -> anyhow::Result<()> {
        if let Some(agent) = self.agents.get_mut(self.active_agent_idx) {
            if let Some(t_idx) = agent.terminal_idx {
                if let Some(term) = self.terminals.get(t_idx) {
                    term.stop();
                }
            }
            agent.state = crate::agents::AgentState::Completed;
            agent.confidence = crate::agents::Confidence::Confirmed;
            let name = agent.name.clone();
            self.status_message = Some((
                format!("Agent '{}' session stopped. (Changes kept in worktree: [d] Diff, [m] Merge)", name),
                std::time::Instant::now(),
            ));
        }
        Ok(())
    }

    pub fn cycle_active_pane(&mut self) {
        let candidates = [
            (ActivePane::Explorer, self.show_explorer),
            (ActivePane::FilePreview, self.show_preview),
            (ActivePane::Terminal, true),
            (ActivePane::Git, self.show_git),
            (ActivePane::Agents, self.show_agents),
        ];

        let next_pane = self.active_pane;
        let mut found_current = false;

        for _ in 0..2 {
            for (pane, is_visible) in candidates {
                if found_current && is_visible {
                    self.active_pane = pane;
                    return;
                }
                if pane == next_pane {
                    found_current = true;
                }
            }
        }
    }

    pub fn poll_bg_notifications(&mut self) {
        while let Ok((msg, refresh_git)) = self.bg_notify_rx.try_recv() {
            self.status_message = Some((msg, std::time::Instant::now()));
            if refresh_git {
                self.git_info = GitRepoInfo::fetch(&self.workspace_dir.to_string_lossy());
            }
        }
    }

    pub fn total_git_items(&self) -> usize {
        self.git_info.staged_files.len() + self.git_info.unstaged_files.len()
    }

    pub fn stage_selected_git_item(&mut self) {
        let staged_len = self.git_info.staged_files.len();
        let total = self.total_git_items();
        if total == 0 {
            return;
        }
        let repo_dir = self.workspace_dir.to_string_lossy().to_string();

        if self.git_selected_idx < staged_len {
            // File is currently staged -> unstage it
            let file = &self.git_info.staged_files[self.git_selected_idx];
            let path = file.path.clone();
            match GitRepoInfo::unstage_file(&repo_dir, &path) {
                Ok(_) => {
                    self.git_info = GitRepoInfo::fetch(&repo_dir);
                    self.status_message = Some((format!("Unstaged {}", path), std::time::Instant::now()));
                }
                Err(e) => {
                    self.status_message = Some((format!("Unstage error: {}", e), std::time::Instant::now()));
                }
            }
        } else {
            // File is currently unstaged -> stage it
            let unstaged_idx = self.git_selected_idx - staged_len;
            if unstaged_idx < self.git_info.unstaged_files.len() {
                let file = &self.git_info.unstaged_files[unstaged_idx];
                let path = file.path.clone();
                match GitRepoInfo::stage_file(&repo_dir, &path) {
                    Ok(_) => {
                        self.git_info = GitRepoInfo::fetch(&repo_dir);
                        self.status_message = Some((format!("Staged {}", path), std::time::Instant::now()));
                    }
                    Err(e) => {
                        self.status_message = Some((format!("Stage error: {}", e), std::time::Instant::now()));
                    }
                }
            }
        }
    }

    pub fn stage_all_git_items(&mut self) {
        let repo_dir = self.workspace_dir.to_string_lossy().to_string();
        match GitRepoInfo::stage_all(&repo_dir) {
            Ok(_) => {
                self.git_info = GitRepoInfo::fetch(&repo_dir);
                self.status_message = Some(("Staged all modified and untracked files".to_string(), std::time::Instant::now()));
            }
            Err(e) => {
                self.status_message = Some((format!("Stage all error: {}", e), std::time::Instant::now()));
            }
        }
    }

    pub fn unstage_all_git_items(&mut self) {
        let repo_dir = self.workspace_dir.to_string_lossy().to_string();
        match GitRepoInfo::unstage_all(&repo_dir) {
            Ok(_) => {
                self.git_info = GitRepoInfo::fetch(&repo_dir);
                self.status_message = Some(("Unstaged all files from index".to_string(), std::time::Instant::now()));
            }
            Err(e) => {
                self.status_message = Some((format!("Unstage all error: {}", e), std::time::Instant::now()));
            }
        }
    }

    pub fn diff_selected_git_item(&mut self) {
        let staged_len = self.git_info.staged_files.len();
        let total = self.total_git_items();
        if total == 0 {
            self.status_message = Some(("No changed files to diff".to_string(), std::time::Instant::now()));
            return;
        }
        let repo_dir = self.workspace_dir.to_string_lossy().to_string();

        let (path, is_staged) = if self.git_selected_idx < staged_len {
            let item = &self.git_info.staged_files[self.git_selected_idx];
            (item.path.clone(), true)
        } else {
            let unstaged_idx = self.git_selected_idx - staged_len;
            if unstaged_idx < self.git_info.unstaged_files.len() {
                let item = &self.git_info.unstaged_files[unstaged_idx];
                (item.path.clone(), false)
            } else {
                return;
            }
        };

        match GitRepoInfo::diff_file(&repo_dir, &path, is_staged) {
            Ok(diff) => {
                let staged_tag = if is_staged { "staged" } else { "unstaged" };
                self.diff_modal_content = Some((format!("Git Diff: {} ({})", path, staged_tag), diff));
                self.diff_modal_scroll = 0;
                self.show_diff_modal = true;
            }
            Err(e) => {
                self.status_message = Some((format!("Diff failed: {}", e), std::time::Instant::now()));
            }
        }
    }

    pub fn execute_commit(&mut self) {
        let msg = self.commit_message_input.trim().to_string();
        if msg.is_empty() {
            self.status_message = Some(("Commit aborted: message cannot be empty".to_string(), std::time::Instant::now()));
            return;
        }
        if self.git_info.staged_files.is_empty() {
            self.status_message = Some(("Commit aborted: no staged changes (press [s] or [a] to stage first)".to_string(), std::time::Instant::now()));
            return;
        }
        let repo_dir = self.workspace_dir.to_string_lossy().to_string();
        self.show_commit_modal = false;
        match GitRepoInfo::commit(&repo_dir, &msg) {
            Ok(out) => {
                self.git_info = GitRepoInfo::fetch(&repo_dir);
                self.commit_message_input.clear();
                let summary = if out.is_empty() { "Changes committed successfully".to_string() } else { out };
                self.status_message = Some((format!("Commit: {}", summary), std::time::Instant::now()));
            }
            Err(e) => {
                self.status_message = Some((format!("Commit failed: {}", e), std::time::Instant::now()));
            }
        }
    }

    pub fn push_git(&mut self) {
        let repo_dir = self.workspace_dir.to_string_lossy().to_string();
        let tx = self.bg_notify_tx.clone();
        let dirty_tx = self.dirty_tx.clone();
        self.status_message = Some(("Git: Pushing commits to remote...".to_string(), std::time::Instant::now()));
        std::thread::spawn(move || {
            match GitRepoInfo::push(&repo_dir) {
                Ok(msg) => {
                    let clean_msg = if msg.is_empty() { "Git: Pushed commits successfully".to_string() } else { format!("Git push: {}", msg) };
                    let _ = tx.send((clean_msg, true));
                }
                Err(e) => {
                    let _ = tx.send((format!("Git push error: {}", e), false));
                }
            }
            let _ = dirty_tx.send(());
        });
    }

    pub fn pull_git(&mut self) {
        let repo_dir = self.workspace_dir.to_string_lossy().to_string();
        let tx = self.bg_notify_tx.clone();
        let dirty_tx = self.dirty_tx.clone();
        self.status_message = Some(("Git: Pulling latest changes from remote...".to_string(), std::time::Instant::now()));
        std::thread::spawn(move || {
            match GitRepoInfo::pull(&repo_dir) {
                Ok(msg) => {
                    let clean_msg = if msg.is_empty() { "Git: Pulled changes successfully".to_string() } else { format!("Git pull: {}", msg) };
                    let _ = tx.send((clean_msg, true));
                }
                Err(e) => {
                    let _ = tx.send((format!("Git pull error: {}", e), false));
                }
            }
            let _ = dirty_tx.send(());
        });
    }

    pub fn get_filtered_branches(&self) -> Vec<&crate::git::GitBranchItem> {
        let q = self.branch_filter_query.trim().to_lowercase();
        if q.is_empty() {
            self.branch_items.iter().collect()
        } else {
            self.branch_items
                .iter()
                .filter(|b| {
                    b.name.to_lowercase().contains(&q)
                        || b.commit_msg.to_lowercase().contains(&q)
                        || b.commit_hash.to_lowercase().contains(&q)
                })
                .collect()
        }
    }

    pub fn open_branch_modal(&mut self) {
        let repo_dir = self.workspace_dir.to_string_lossy().to_string();
        match GitRepoInfo::list_branches(&repo_dir) {
            Ok(branches) => {
                let current_idx = branches.iter().position(|b| b.is_current).unwrap_or(0);
                self.branch_items = branches;
                self.branch_selected_idx = current_idx;
                self.branch_filter_query.clear();
                self.branch_is_creating = false;
                self.branch_new_name.clear();
                self.show_branch_modal = true;
            }
            Err(e) => {
                self.status_message = Some((format!("Failed to list branches: {}", e), std::time::Instant::now()));
            }
        }
    }

    pub fn checkout_selected_branch(&mut self) {
        let filtered = self.get_filtered_branches();
        if filtered.is_empty() {
            return;
        }
        let sel_idx = self.branch_selected_idx.min(filtered.len().saturating_sub(1));
        let item = filtered[sel_idx];
        if item.is_current {
            self.status_message = Some((format!("Already on branch '{}'", item.display_name()), std::time::Instant::now()));
            self.show_branch_modal = false;
            return;
        }
        let repo_dir = self.workspace_dir.to_string_lossy().to_string();
        let target_name = item.name.clone();
        let disp_name = item.display_name().to_string();
        match GitRepoInfo::checkout_branch(&repo_dir, &target_name) {
            Ok(_) => {
                self.git_info = GitRepoInfo::fetch(&repo_dir);
                self.show_branch_modal = false;
                self.status_message = Some((format!("Switched to branch '{}'", disp_name), std::time::Instant::now()));
            }
            Err(e) => {
                self.status_message = Some((format!("Checkout error: {}", e), std::time::Instant::now()));
            }
        }
    }

    pub fn create_and_switch_branch(&mut self) {
        let name = self.branch_new_name.trim().to_string();
        if name.is_empty() {
            self.status_message = Some(("Branch name cannot be empty".to_string(), std::time::Instant::now()));
            return;
        }
        let repo_dir = self.workspace_dir.to_string_lossy().to_string();
        match GitRepoInfo::create_and_checkout_branch(&repo_dir, &name) {
            Ok(_) => {
                self.git_info = GitRepoInfo::fetch(&repo_dir);
                self.show_branch_modal = false;
                self.branch_is_creating = false;
                self.branch_new_name.clear();
                self.status_message = Some((format!("Created & switched to branch '{}'", name), std::time::Instant::now()));
            }
            Err(e) => {
                self.status_message = Some((format!("Create branch error: {}", e), std::time::Instant::now()));
            }
        }
    }

    pub fn delete_selected_branch(&mut self, force: bool) {
        let filtered = self.get_filtered_branches();
        if filtered.is_empty() {
            return;
        }
        let sel_idx = self.branch_selected_idx.min(filtered.len().saturating_sub(1));
        let item = filtered[sel_idx];
        if item.is_current {
            self.status_message = Some(("Cannot delete currently checked out branch!".to_string(), std::time::Instant::now()));
            return;
        }
        if item.is_worktree {
            self.status_message = Some(("Cannot delete branch checked out in active worktree!".to_string(), std::time::Instant::now()));
            return;
        }
        let target_name = item.name.clone();
        let repo_dir = self.workspace_dir.to_string_lossy().to_string();
        match GitRepoInfo::delete_branch(&repo_dir, &target_name, force) {
            Ok(_) => {
                if let Ok(branches) = GitRepoInfo::list_branches(&repo_dir) {
                    self.branch_items = branches;
                    self.branch_selected_idx = self.branch_selected_idx.saturating_sub(1);
                }
                self.status_message = Some((format!("Deleted branch '{}'", target_name), std::time::Instant::now()));
            }
            Err(e) => {
                let hint = if !force { " (try [Shift+d] to force delete)" } else { "" };
                self.status_message = Some((format!("Delete error: {}{}", e, hint), std::time::Instant::now()));
            }
        }
    }

    pub fn open_stash_modal(&mut self) {
        let repo_dir = self.workspace_dir.to_string_lossy().to_string();
        match GitRepoInfo::list_stashes(&repo_dir) {
            Ok(stashes) => {
                self.stash_items = stashes;
                self.stash_selected_idx = 0;
                self.stash_is_creating = false;
                self.stash_new_message.clear();
                self.stash_include_untracked = true;
                self.show_stash_modal = true;
            }
            Err(e) => {
                self.status_message = Some((format!("Failed to list stashes: {}", e), std::time::Instant::now()));
            }
        }
    }

    pub fn pop_selected_stash(&mut self) {
        if self.stash_items.is_empty() {
            self.status_message = Some(("No stashes to pop".to_string(), std::time::Instant::now()));
            return;
        }
        let sel_idx = self.stash_selected_idx.min(self.stash_items.len().saturating_sub(1));
        let target_idx = self.stash_items[sel_idx].index;
        let repo_dir = self.workspace_dir.to_string_lossy().to_string();
        match GitRepoInfo::stash_pop(&repo_dir, target_idx) {
            Ok(_) => {
                self.git_info = GitRepoInfo::fetch(&repo_dir);
                let _ = GitRepoInfo::list_stashes(&repo_dir).map(|s| self.stash_items = s);
                self.stash_selected_idx = self.stash_selected_idx.saturating_sub(1);
                self.show_stash_modal = false;
                self.status_message = Some((format!("Popped stash@{{{}}} successfully", target_idx), std::time::Instant::now()));
            }
            Err(e) => {
                self.status_message = Some((format!("Stash pop error: {}", e), std::time::Instant::now()));
            }
        }
    }

    pub fn apply_selected_stash(&mut self) {
        if self.stash_items.is_empty() {
            self.status_message = Some(("No stashes to apply".to_string(), std::time::Instant::now()));
            return;
        }
        let sel_idx = self.stash_selected_idx.min(self.stash_items.len().saturating_sub(1));
        let target_idx = self.stash_items[sel_idx].index;
        let repo_dir = self.workspace_dir.to_string_lossy().to_string();
        match GitRepoInfo::stash_apply(&repo_dir, target_idx) {
            Ok(_) => {
                self.git_info = GitRepoInfo::fetch(&repo_dir);
                self.show_stash_modal = false;
                self.status_message = Some((format!("Applied stash@{{{}}} successfully", target_idx), std::time::Instant::now()));
            }
            Err(e) => {
                self.status_message = Some((format!("Stash apply error: {}", e), std::time::Instant::now()));
            }
        }
    }

    pub fn drop_selected_stash(&mut self) {
        if self.stash_items.is_empty() {
            return;
        }
        let sel_idx = self.stash_selected_idx.min(self.stash_items.len().saturating_sub(1));
        let target_idx = self.stash_items[sel_idx].index;
        let repo_dir = self.workspace_dir.to_string_lossy().to_string();
        match GitRepoInfo::stash_drop(&repo_dir, target_idx) {
            Ok(_) => {
                if let Ok(stashes) = GitRepoInfo::list_stashes(&repo_dir) {
                    self.stash_items = stashes;
                    self.stash_selected_idx = self.stash_selected_idx.saturating_sub(1);
                }
                self.status_message = Some((format!("Dropped stash@{{{}}}", target_idx), std::time::Instant::now()));
            }
            Err(e) => {
                self.status_message = Some((format!("Stash drop error: {}", e), std::time::Instant::now()));
            }
        }
    }

    pub fn view_selected_stash_diff(&mut self) {
        if self.stash_items.is_empty() {
            return;
        }
        let sel_idx = self.stash_selected_idx.min(self.stash_items.len().saturating_sub(1));
        let item = &self.stash_items[sel_idx];
        let repo_dir = self.workspace_dir.to_string_lossy().to_string();
        match GitRepoInfo::stash_diff(&repo_dir, item.index) {
            Ok(diff) => {
                self.diff_modal_content = Some((format!("Stash Diff: {} ({})", item.selector, item.message), diff));
                self.diff_modal_scroll = 0;
                self.show_diff_modal = true;
            }
            Err(e) => {
                self.status_message = Some((format!("Stash diff error: {}", e), std::time::Instant::now()));
            }
        }
    }

    pub fn save_new_stash(&mut self) {
        let repo_dir = self.workspace_dir.to_string_lossy().to_string();
        let msg = self.stash_new_message.trim();
        let opt_msg = if msg.is_empty() { None } else { Some(msg) };
        match GitRepoInfo::stash_save(&repo_dir, opt_msg, self.stash_include_untracked) {
            Ok(_) => {
                self.git_info = GitRepoInfo::fetch(&repo_dir);
                if let Ok(stashes) = GitRepoInfo::list_stashes(&repo_dir) {
                    self.stash_items = stashes;
                }
                self.stash_is_creating = false;
                self.stash_new_message.clear();
                self.status_message = Some(("Changes stashed successfully".to_string(), std::time::Instant::now()));
            }
            Err(e) => {
                self.status_message = Some((format!("Stash error: {}", e), std::time::Instant::now()));
            }
        }
    }

    pub fn handle_mouse_event(&mut self, mouse: crossterm::event::MouseEvent) {
        let (x, y) = (mouse.column, mouse.row);

        if self.show_welcome_screen {
            self.handle_welcome_mouse_event(mouse);
            return;
        }

        // Handle clicks on Explorer Operations Modal (New / Rename / Delete)
        if self.explorer_op_modal != ExplorerOpModal::None {
            if let crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) = mouse.kind {
                if !self.explorer_op_modal_area.contains(ratatui::layout::Position { x, y }) {
                    self.explorer_op_modal = ExplorerOpModal::None;
                    return;
                }
                // Check [×] button in top right
                if y == self.explorer_op_modal_area.y && x >= self.explorer_op_modal_area.right().saturating_sub(4) {
                    self.explorer_op_modal = ExplorerOpModal::None;
                    return;
                }
            }
            return;
        }

        // Handle clicks on Command Palette / Fuzzy Finder Modal
        if self.show_command_palette {
            if let crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) = mouse.kind {
                if !self.palette_modal_area.contains(ratatui::layout::Position { x, y }) {
                    self.show_command_palette = false;
                    return;
                }

                // Check [×] button
                if y == self.palette_modal_area.y && x >= self.palette_modal_area.right().saturating_sub(4) {
                    self.show_command_palette = false;
                    return;
                }

                // List items start 4 lines below modal top (border + input box)
                let list_start_y = self.palette_modal_area.y + 4;
                if y >= list_start_y {
                    let clicked_idx = (y - list_start_y) as usize;
                    let items = self.get_palette_items();
                    if clicked_idx < items.len() {
                        let action = items[clicked_idx].action.clone();
                        self.show_command_palette = false;
                        self.execute_palette_action(action);
                        return;
                    }
                }
            }
            return;
        }

        // Handle clicks on Terminal Menu Modal
        if self.show_terminal_menu {
            if let crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) = mouse.kind {
                if !self.terminal_menu_modal_area.contains(ratatui::layout::Position { x, y }) {
                    self.show_terminal_menu = false;
                    return;
                }

                // Check [×] button
                if y == self.terminal_menu_modal_area.y && x >= self.terminal_menu_modal_area.right().saturating_sub(4) {
                    self.show_terminal_menu = false;
                    return;
                }

                let inner_y = self.terminal_menu_modal_area.y + 1;
                // Option 1: cmd.exe
                if y >= inner_y + 2 && y <= inner_y + 4 {
                    self.terminal_menu_selected_idx = 0;
                    return;
                }
                // Option 2: PowerShell
                if y >= inner_y + 5 && y <= inner_y + 7 {
                    self.terminal_menu_selected_idx = 1;
                    return;
                }
                // Option 3: Git Bash
                if y >= inner_y + 8 && y <= inner_y + 10 {
                    self.terminal_menu_selected_idx = 2;
                    return;
                }
                // Option 4: WSL
                if y >= inner_y + 11 && y <= inner_y + 13 {
                    self.terminal_menu_selected_idx = 3;
                    return;
                }
                // Confirm & Cancel buttons row
                if y >= self.terminal_menu_modal_area.bottom().saturating_sub(3) {
                    let mid_x = self.terminal_menu_modal_area.x + self.terminal_menu_modal_area.width / 2;
                    if x < mid_x {
                        let _ = self.spawn_selected_terminal(self.terminal_menu_selected_idx);
                    }
                    self.show_terminal_menu = false;
                    return;
                }
            }
            return;
        }

        // Handle clicks on Agent Launch Modal
        if self.show_agent_launch_modal {
            if let crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) = mouse.kind {
                if !self.agent_launch_modal_area.contains(ratatui::layout::Position { x, y }) {
                    self.show_agent_launch_modal = false;
                    return;
                }

                // Check [×] button
                if y == self.agent_launch_modal_area.y && x >= self.agent_launch_modal_area.right().saturating_sub(4) {
                    self.show_agent_launch_modal = false;
                    return;
                }

                let inner_y = self.agent_launch_modal_area.y + 1;
                // Option 1 (Antigravity)
                if y >= inner_y + 2 && y <= inner_y + 4 {
                    self.agent_launch_selected_idx = 0;
                    return;
                }
                // Option 2 (Claude Code)
                if y >= inner_y + 5 && y <= inner_y + 7 {
                    self.agent_launch_selected_idx = 1;
                    return;
                }
                // Option 3 (Worktree Shell)
                if y >= inner_y + 8 && y <= inner_y + 10 {
                    self.agent_launch_selected_idx = 2;
                    return;
                }
                // Confirm & Cancel buttons row
                if y >= self.agent_launch_modal_area.bottom().saturating_sub(3) {
                    let mid_x = self.agent_launch_modal_area.x + self.agent_launch_modal_area.width / 2;
                    if x < mid_x {
                        // Confirm clicked!
                        match self.agent_launch_selected_idx {
                            0 => { let _ = self.launch_agent("Antigravity", "agy", &[], "Interactive Coding Agent"); }
                            1 => { let _ = self.launch_agent("Claude Code", "claude", &[], "Autonomous Agent Session"); }
                            2 => {
                                let shell = if cfg!(windows) { "powershell.exe" } else { "bash" };
                                let _ = self.launch_agent("Worktree Shell", shell, &[], "Isolated Worktree Shell");
                            }
                            _ => {}
                        }
                    }
                    self.show_agent_launch_modal = false;
                    return;
                }
            }
            return;
        }

        // Handle clicks on Commit Modal
        if self.show_commit_modal {
            if let crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) = mouse.kind {
                if !self.commit_modal_area.contains(ratatui::layout::Position { x, y }) {
                    self.show_commit_modal = false;
                    return;
                }

                // Check [×] button
                if y == self.commit_modal_area.y && x >= self.commit_modal_area.right().saturating_sub(4) {
                    self.show_commit_modal = false;
                    return;
                }

                // Confirm (Commit) & Cancel buttons row
                if y >= self.commit_modal_area.bottom().saturating_sub(3) {
                    let mid_x = self.commit_modal_area.x + self.commit_modal_area.width / 2;
                    if x < mid_x {
                        self.execute_commit();
                    } else {
                        self.show_commit_modal = false;
                    }
                    return;
                }
            }
            return;
        }

        // Handle clicks and wheel on Diff Modal
        if self.show_diff_modal {
            match mouse.kind {
                crossterm::event::MouseEventKind::ScrollDown => {
                    self.diff_modal_scroll += 3;
                    return;
                }
                crossterm::event::MouseEventKind::ScrollUp => {
                    self.diff_modal_scroll = self.diff_modal_scroll.saturating_sub(3);
                    return;
                }
                crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                    if !self.diff_modal_area.contains(ratatui::layout::Position { x, y }) {
                        self.show_diff_modal = false;
                        return;
                    }
                    if y == self.diff_modal_area.y && x >= self.diff_modal_area.right().saturating_sub(4) {
                        self.show_diff_modal = false;
                        return;
                    }
                    // Bottom actions row:
                    if y >= self.diff_modal_area.bottom().saturating_sub(2) {
                        let rel_x = x.saturating_sub(self.diff_modal_area.x);
                        let w = self.diff_modal_area.width;
                        let is_agent_diff = self.diff_modal_content.as_ref().map(|(t, _)| t.starts_with("Diff:")).unwrap_or(false);
                        if is_agent_diff {
                            if rel_x < w / 3 {
                                let _ = self.merge_active_agent();
                            } else if rel_x < (2 * w) / 3 {
                                let _ = self.discard_active_agent();
                            } else {
                                self.show_diff_modal = false;
                            }
                        } else {
                            if rel_x < w / 2 {
                                self.stage_selected_git_item();
                            } else {
                                self.show_diff_modal = false;
                            }
                        }
                        return;
                    }
                }
                _ => {}
            }
            return;
        }

        // Handle clicks on Theme Modal
        if self.show_theme_modal {
            if let crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) = mouse.kind {
                if !self.theme_modal_area.contains(ratatui::layout::Position { x, y }) {
                    self.color_theme = self.previous_color_theme;
                    self.show_theme_modal = false;
                    return;
                }

                // Check [×] button
                if y == self.theme_modal_area.y && x >= self.theme_modal_area.right().saturating_sub(4) {
                    self.color_theme = self.previous_color_theme;
                    self.show_theme_modal = false;
                    return;
                }

                let inner_y = self.theme_modal_area.y + 1;
                let list_start = inner_y + 2;
                let themes = crate::theme::ColorTheme::ALL;
                for (idx, theme) in themes.iter().enumerate() {
                    let item_top = list_start + (idx as u16 * 2);
                    if y >= item_top && y <= item_top + 1 {
                        self.theme_modal_selected_idx = idx;
                        self.color_theme = *theme;
                        return;
                    }
                }

                // Check bottom action buttons row
                if y >= self.theme_modal_area.bottom().saturating_sub(3) {
                    let mid_x = self.theme_modal_area.x + self.theme_modal_area.width / 2;
                    if x < mid_x {
                        self.status_message = Some((
                            format!("Theme switched to: {}", self.color_theme.name()),
                            std::time::Instant::now(),
                        ));
                    } else {
                        self.color_theme = self.previous_color_theme;
                    }
                    self.show_theme_modal = false;
                    return;
                }
            }
            return;
        }

        // Handle clicks on Branch Modal
        if self.show_branch_modal {
            if let crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) = mouse.kind {
                if !self.branch_modal_area.contains(ratatui::layout::Position { x, y }) {
                    self.show_branch_modal = false;
                    return;
                }
                // Check [×] button in top right
                if y == self.branch_modal_area.y && x >= self.branch_modal_area.right().saturating_sub(4) {
                    self.show_branch_modal = false;
                    return;
                }
                if !self.branch_is_creating {
                    // List items click (starts row 4 inside modal)
                    let list_start_y = self.branch_modal_area.y + 4;
                    let filtered = self.get_filtered_branches();
                    if y >= list_start_y && (y as usize) < list_start_y as usize + filtered.len() {
                        let clicked_idx = (y - list_start_y) as usize;
                        if clicked_idx < filtered.len() {
                            if self.branch_selected_idx == clicked_idx {
                                self.checkout_selected_branch();
                            } else {
                                self.branch_selected_idx = clicked_idx;
                            }
                        }
                        return;
                    }
                    // Bottom buttons: [Enter] Checkout, [n] New, [d] Delete, [Esc] Close
                    if y >= self.branch_modal_area.bottom().saturating_sub(2) {
                        let rel_x = x.saturating_sub(self.branch_modal_area.x);
                        let w = self.branch_modal_area.width;
                        let btn_w = (w / 4).max(1);
                        match rel_x / btn_w {
                            0 => self.checkout_selected_branch(),
                            1 => {
                                self.branch_is_creating = true;
                                self.branch_new_name.clear();
                            }
                            2 => self.delete_selected_branch(false),
                            _ => self.show_branch_modal = false,
                        }
                        return;
                    }
                } else {
                    // Creating mode bottom buttons: Confirm / Cancel
                    if y >= self.branch_modal_area.bottom().saturating_sub(2) {
                        let mid_x = self.branch_modal_area.x + self.branch_modal_area.width / 2;
                        if x < mid_x {
                            self.create_and_switch_branch();
                        } else {
                            self.branch_is_creating = false;
                        }
                        return;
                    }
                }
            }
            return;
        }

        // Handle clicks on Stash Modal
        if self.show_stash_modal {
            if let crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) = mouse.kind {
                if !self.stash_modal_area.contains(ratatui::layout::Position { x, y }) {
                    self.show_stash_modal = false;
                    return;
                }
                // Check [×] button
                if y == self.stash_modal_area.y && x >= self.stash_modal_area.right().saturating_sub(4) {
                    self.show_stash_modal = false;
                    return;
                }
                if !self.stash_is_creating {
                    // List items click
                    let list_start_y = self.stash_modal_area.y + 3;
                    if y >= list_start_y && (y as usize) < list_start_y as usize + self.stash_items.len() {
                        let clicked_idx = (y - list_start_y) as usize;
                        if clicked_idx < self.stash_items.len() {
                            if self.stash_selected_idx == clicked_idx {
                                self.pop_selected_stash();
                            } else {
                                self.stash_selected_idx = clicked_idx;
                            }
                        }
                        return;
                    }
                    // Bottom buttons: Pop, Apply, New Stash, Diff, Drop
                    if y >= self.stash_modal_area.bottom().saturating_sub(2) {
                        let rel_x = x.saturating_sub(self.stash_modal_area.x);
                        let w = self.stash_modal_area.width;
                        let btn_w = (w / 5).max(1);
                        match rel_x / btn_w {
                            0 => self.pop_selected_stash(),
                            1 => self.apply_selected_stash(),
                            2 => {
                                self.stash_is_creating = true;
                                self.stash_new_message.clear();
                            }
                            3 => self.view_selected_stash_diff(),
                            _ => self.drop_selected_stash(),
                        }
                        return;
                    }
                } else {
                    // Creating mode: toggle untracked or confirm/cancel
                    if y == self.stash_modal_area.y + 5 {
                        self.stash_include_untracked = !self.stash_include_untracked;
                        return;
                    }
                    if y >= self.stash_modal_area.bottom().saturating_sub(2) {
                        let mid_x = self.stash_modal_area.x + self.stash_modal_area.width / 2;
                        if x < mid_x {
                            self.save_new_stash();
                        } else {
                            self.stash_is_creating = false;
                        }
                        return;
                    }
                }
            }
            return;
        }

        match mouse.kind {
            crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                // Check splitters for card resizing FIRST before pane content
                // 1. Center row splitter (between File Preview and Terminal)
                if self.show_preview && self.preview_area.height > 0 {
                    let split_y = self.preview_area.bottom();
                    if (y as i32 - split_y as i32).abs() <= 1 && x >= self.preview_area.x && x < self.preview_area.right() {
                        self.active_splitter_drag = Some(SplitterKind::CenterRow);
                        return;
                    }
                }

                // 2. Right row splitter (between Git and Agents)
                if self.show_git && self.show_agents && self.git_area.height > 0 {
                    let split_y = self.git_area.bottom();
                    if (y as i32 - split_y as i32).abs() <= 1 && x >= self.git_area.x && x < self.git_area.right() {
                        self.active_splitter_drag = Some(SplitterKind::RightRow);
                        return;
                    }
                }

                // 3. Left column splitter (between Explorer and Center)
                if self.show_explorer && self.explorer_area.width > 0 {
                    let split_x = self.explorer_area.right();
                    if (x as i32 - split_x as i32).abs() <= 1 && y >= self.body_area.y && y < self.body_area.bottom() {
                        self.active_splitter_drag = Some(SplitterKind::LeftColumn);
                        return;
                    }
                }

                // 4. Right column splitter (between Center and Right column)
                let has_right = self.show_git || self.show_agents;
                if has_right {
                    let right_x = if self.show_git { self.git_area.x } else { self.agents_area.x };
                    if right_x > 0 && (x as i32 - right_x as i32).abs() <= 1 && y >= self.body_area.y && y < self.body_area.bottom() {
                        self.active_splitter_drag = Some(SplitterKind::RightColumn);
                        return;
                    }
                }

                // 4b. Header left branding click -> Workspace Launcher
                if self.header_left_area.contains(ratatui::layout::Position { x, y }) {
                    self.toggle_welcome_screen();
                    return;
                }

                // 5. Header quick-toggle buttons click
                if self.header_mid_area.contains(ratatui::layout::Position { x, y }) {
                    let rel_x = x.saturating_sub(self.header_mid_area.x);
                    let w = self.header_area.width as usize;
                    if w >= 100 {
                        // [F2 Exp] (0..8) [F5 Prev] (9..18) [F3 Git] (19..27) [F4 Agt] (28..36) [F11 Max] (37..46)
                        if rel_x <= 8 {
                            self.show_explorer = !self.show_explorer;
                        } else if rel_x <= 18 {
                            self.show_preview = !self.show_preview;
                        } else if rel_x <= 27 {
                            self.show_git = !self.show_git;
                        } else if rel_x <= 36 {
                            self.show_agents = !self.show_agents;
                        } else if rel_x <= 46 {
                            self.is_maximized = !self.is_maximized;
                        }
                    } else if w >= 80 {
                        // [F2] (0..4) [F5] (5..9) [F3] (10..14) [F4] (15..19) [F11] (20..25)
                        if rel_x <= 4 {
                            self.show_explorer = !self.show_explorer;
                        } else if rel_x <= 9 {
                            self.show_preview = !self.show_preview;
                        } else if rel_x <= 14 {
                            self.show_git = !self.show_git;
                        } else if rel_x <= 19 {
                            self.show_agents = !self.show_agents;
                        } else if rel_x <= 25 {
                            self.is_maximized = !self.is_maximized;
                        }
                    } else if w >= 60 {
                        // [F2] (0..4) [F3] (5..9) [F4] (10..14) [F11] (15..20)
                        if rel_x <= 4 {
                            self.show_explorer = !self.show_explorer;
                        } else if rel_x <= 9 {
                            self.show_git = !self.show_git;
                        } else if rel_x <= 14 {
                            self.show_agents = !self.show_agents;
                        } else if rel_x <= 20 {
                            self.is_maximized = !self.is_maximized;
                        }
                    }
                    return;
                }

                // 6. Header right shortcuts click (Theme, Help, Quit, Pane)
                if self.header_right_area.contains(ratatui::layout::Position { x, y }) {
                    let from_right = self.header_right_area.right().saturating_sub(x);
                    if from_right <= 10 {
                        // Quit
                        self.should_quit = true;
                    } else if from_right <= 20 {
                        // Help
                        self.show_help = !self.show_help;
                    } else if from_right <= 32 {
                        // F12 Theme
                        self.show_theme_modal = true;
                        self.previous_color_theme = self.color_theme;
                        self.theme_modal_selected_idx = crate::theme::ColorTheme::ALL
                            .iter()
                            .position(|&t| t == self.color_theme)
                            .unwrap_or(0);
                    } else if from_right <= 42 {
                        // F6 Pane
                        self.active_pane = match self.active_pane {
                            ActivePane::Explorer => {
                                if self.show_preview {
                                    ActivePane::FilePreview
                                } else {
                                    ActivePane::Terminal
                                }
                            }
                            ActivePane::FilePreview => ActivePane::Terminal,
                            ActivePane::Terminal => {
                                if self.show_git {
                                    ActivePane::Git
                                } else if self.show_agents {
                                    ActivePane::Agents
                                } else if self.show_explorer {
                                    ActivePane::Explorer
                                } else {
                                    ActivePane::Terminal
                                }
                            }
                            ActivePane::Git => {
                                if self.show_agents {
                                    ActivePane::Agents
                                } else if self.show_explorer {
                                    ActivePane::Explorer
                                } else {
                                    ActivePane::Terminal
                                }
                            }
                            ActivePane::Agents => {
                                if self.show_explorer {
                                    ActivePane::Explorer
                                } else {
                                    ActivePane::Terminal
                                }
                            }
                        };
                    }
                    return;
                }

                // Check Terminal tab bar clicks
                if self.terminal_tabs_area.contains(ratatui::layout::Position { x, y }) {
                    self.active_pane = ActivePane::Terminal;
                    let rel_x = x.saturating_sub(self.terminal_tabs_area.x);

                    // Check navigation buttons on far right: [F7 ◀] [▶ F8]
                    let right_boundary = self.terminal_tabs_area.width;
                    if rel_x >= right_boundary.saturating_sub(26) {
                        if rel_x < right_boundary.saturating_sub(13) {
                            if !self.terminals.is_empty() {
                                self.active_terminal_idx = (self.active_terminal_idx + self.terminals.len() - 1) % self.terminals.len();
                            }
                        } else {
                            if !self.terminals.is_empty() {
                                self.active_terminal_idx = (self.active_terminal_idx + 1) % self.terminals.len();
                            }
                        }
                        return;
                    }

                    let available_w = (self.terminal_tabs_area.width.saturating_sub(26)) as usize;

                    // Check left overflow arrow
                    let mut current_offset: u16 = 0;
                    if self.terminal_tab_scroll > 0 {
                        if rel_x < 3 {
                            if self.active_terminal_idx > 0 {
                                self.active_terminal_idx -= 1;
                            }
                            return;
                        }
                        current_offset += 3;
                    }

                    let mut last_visible_tab = self.terminal_tab_scroll;
                    for i in self.terminal_tab_scroll..self.terminals.len() {
                        let clean = format_terminal_tab_title(&self.terminals[i].title);
                        let tab_w = (4 + format!("{}: {}", i + 1, clean).chars().count() + 2) as u16;

                        if i > self.terminal_tab_scroll && (current_offset as usize + tab_w as usize + 8) > available_w {
                            break;
                        }

                        if rel_x >= current_offset && rel_x < current_offset + tab_w {
                            self.active_terminal_idx = i;
                            return;
                        }
                        current_offset += tab_w;
                        last_visible_tab = i;
                    }

                    // Check right overflow arrow
                    if last_visible_tab + 1 < self.terminals.len() {
                        if rel_x >= current_offset && rel_x < current_offset + 3 {
                            if self.active_terminal_idx + 1 < self.terminals.len() {
                                self.active_terminal_idx += 1;
                            }
                            return;
                        }
                        current_offset += 3;
                    }

                    // Check [+] button
                    if rel_x >= current_offset && rel_x <= current_offset + 6 {
                        self.show_terminal_menu = true;
                        self.terminal_menu_selected_idx = 0;
                        return;
                    }

                    return;
                }

                // Check Explorer Footer button clicks (New, Rename, Delete, Refresh)
                if self.show_explorer && self.explorer_footer_area.height > 0 && self.explorer_footer_area.contains(ratatui::layout::Position { x, y }) {
                    self.active_pane = ActivePane::Explorer;
                    let rel_x = x.saturating_sub(self.explorer_footer_area.x);
                    let rel_y = y.saturating_sub(self.explorer_footer_area.y);
                    let footer_w = self.explorer_footer_area.width;
                    let footer_h = self.explorer_footer_area.height;

                    if footer_h >= 2 {
                        if rel_y == 0 {
                            if rel_x < footer_w / 2 {
                                self.prompt_new_item();
                            } else {
                                self.prompt_rename_item();
                            }
                        } else {
                            if rel_x < footer_w / 2 {
                                self.prompt_delete_item();
                            } else {
                                self.explorer.refresh();
                                self.status_message = Some(("Explorer refreshed".to_string(), std::time::Instant::now()));
                            }
                        }
                    } else {
                        let col_w = (footer_w / 4).max(1);
                        let quadrant = rel_x / col_w;
                        match quadrant {
                            0 => self.prompt_new_item(),
                            1 => self.prompt_rename_item(),
                            2 => self.prompt_delete_item(),
                            _ => {
                                self.explorer.refresh();
                                self.status_message = Some(("Explorer refreshed".to_string(), std::time::Instant::now()));
                            }
                        }
                    }
                    return;
                }

                // Check Explorer clicks
                if self.explorer_area.contains(ratatui::layout::Position { x, y }) {
                    if y == self.explorer_area.y && x >= self.explorer_area.right().saturating_sub(4) {
                        self.show_explorer = false;
                        return;
                    }
                    self.active_pane = ActivePane::Explorer;
                    let inner_y = self.explorer_area.y + 1;
                    let footer_h = self.explorer_footer_area.height;
                    let view_height = self.explorer_area.height.saturating_sub(2 + footer_h) as usize;
                    let scroll = self.explorer.effective_scroll(view_height);
                    if y >= inner_y && y < self.explorer_area.bottom().saturating_sub(1 + footer_h) {
                        let clicked_row = (y - inner_y) as usize;
                        let clicked_idx = clicked_row + scroll;
                        if clicked_idx < self.explorer.entries.len() {
                            self.explorer.selected = clicked_idx;
                            let is_dir = self.explorer.entries[clicked_idx].is_dir;
                            if is_dir {
                                self.explorer.toggle_selected();
                            }
                            self.update_file_preview();
                        }
                    }
                    return;
                }

                // Check Global Search modal clicks
                if self.show_global_search {
                    if !self.global_search_modal_area.contains(ratatui::layout::Position { x, y }) {
                        self.show_global_search = false;
                        return;
                    }
                    if y == self.global_search_modal_area.y && x >= self.global_search_modal_area.right().saturating_sub(4) {
                        self.show_global_search = false;
                        return;
                    }
                    let list_y = self.global_search_modal_area.y + 4;
                    if y >= list_y && y < self.global_search_modal_area.bottom().saturating_sub(2) {
                        let clicked_row = (y - list_y) as usize;
                        if clicked_row < self.global_search_results.len() {
                            self.global_search_selected_idx = clicked_row;
                            if let Some(result) = self.global_search_results.get(clicked_row).cloned() {
                                self.show_global_search = false;
                                self.open_file(&result.file_path, true);
                                self.active_pane = ActivePane::FilePreview;
                                let line_idx = result.line_num.saturating_sub(1);
                                self.editor_cursor = (line_idx, result.match_start);
                                let view_height = self.preview_area.height.saturating_sub(4) as usize;
                                if view_height > 0 {
                                    self.preview_scroll = line_idx.saturating_sub(view_height / 2);
                                }
                                self.status_message = Some((
                                    format!("Jumped to {}:{}", result.rel_path, result.line_num),
                                    std::time::Instant::now(),
                                ));
                            }
                        }
                    }
                    return;
                }

                // Check Find Bar clicks
                if self.show_find_in_file && self.find_bar_area.contains(ratatui::layout::Position { x, y }) {
                    self.active_pane = ActivePane::FilePreview;
                    let rel_x = x.saturating_sub(self.find_bar_area.x);
                    let right_boundary = self.find_bar_area.width;
                    if rel_x >= right_boundary.saturating_sub(10) {
                        self.show_find_in_file = false;
                    } else if rel_x >= right_boundary.saturating_sub(24) {
                        self.prev_find_match();
                    } else if rel_x >= right_boundary.saturating_sub(35) {
                        self.next_find_match();
                    }
                    return;
                }

                // Check Editor Tab Bar clicks
                if self.preview_tabs_area.contains(ratatui::layout::Position { x, y }) {
                    self.active_pane = ActivePane::FilePreview;
                    let rel_x = x.saturating_sub(self.preview_tabs_area.x);
                    let right_boundary = self.preview_tabs_area.width;

                    // Check navigation buttons on far right: [F9 ◀] [▶ F10]
                    if rel_x >= right_boundary.saturating_sub(26) {
                        if rel_x < right_boundary.saturating_sub(13) {
                            self.prev_editor_tab();
                        } else {
                            self.next_editor_tab();
                        }
                        return;
                    }

                    let available_w = (self.preview_tabs_area.width.saturating_sub(26)) as usize;

                    // Check left overflow arrow
                    let mut current_offset: u16 = 0;
                    if self.editor_tab_scroll > 0 {
                        if rel_x < 3 {
                            self.prev_editor_tab();
                            return;
                        }
                        current_offset += 3;
                    }

                    let mut last_visible_tab = self.editor_tab_scroll;
                    for i in self.editor_tab_scroll..self.editor_tabs.len() {
                        let tab = &self.editor_tabs[i];
                        let (f_icon, _) = crate::icons::file_icon(&tab.file_name, self.icon_theme);
                        let dirty_len = if tab.dirty { 2 } else { 0 };
                        let tab_w = (1 + f_icon.chars().count() + 1 + tab.file_name.chars().count() + dirty_len + 5 + 1) as u16;

                        if i > self.editor_tab_scroll && (current_offset as usize + tab_w as usize + 6) > available_w {
                            break;
                        }

                        if rel_x >= current_offset && rel_x < current_offset + tab_w {
                            // Check if clicked close button [×] (last 5 chars before separator)
                            let close_start = current_offset + tab_w.saturating_sub(6);
                            let close_end = current_offset + tab_w.saturating_sub(1);
                            if rel_x >= close_start && rel_x < close_end {
                                self.close_editor_tab(i);
                            } else {
                                self.switch_editor_tab(i);
                                self.editor_tabs[i].is_pinned = true;
                            }
                            return;
                        }

                        current_offset += tab_w;
                        last_visible_tab = i;
                    }

                    // Check right overflow arrow
                    if last_visible_tab + 1 < self.editor_tabs.len() {
                        if rel_x >= current_offset && rel_x < current_offset + 3 {
                            self.next_editor_tab();
                            return;
                        }
                    }

                    return;
                }

                // Check Preview clicks
                if self.preview_area.contains(ratatui::layout::Position { x, y }) {
                    if y == self.preview_area.y && x >= self.preview_area.right().saturating_sub(4) {
                        self.show_preview = false;
                        return;
                    }
                    self.active_pane = ActivePane::FilePreview;
                    let inner_y = self.preview_area.y + if self.show_find_in_file { 3 } else { 2 };
                    let inner_x = self.preview_area.x + 7;
                    if y >= inner_y {
                        let clicked_line = self.preview_scroll + (y - inner_y) as usize;
                        let clicked_col = (x as usize).saturating_sub(inner_x as usize);
                        if let Some((_, lines)) = &self.preview_content {
                            if clicked_line < lines.len() {
                                self.editor_cursor.0 = clicked_line;
                                self.editor_cursor.1 = clicked_col.min(lines[clicked_line].chars().count());
                            }
                        }
                    }
                    return;
                }

                // Check Terminal screen clicks
                if self.terminal_area.contains(ratatui::layout::Position { x, y }) {
                    self.active_pane = ActivePane::Terminal;
                    return;
                }

                // Check Git pane clicks
                if self.git_area.contains(ratatui::layout::Position { x, y }) {
                    if y == self.git_area.y && x >= self.git_area.right().saturating_sub(4) {
                        self.show_git = false;
                        return;
                    }
                    self.active_pane = ActivePane::Git;

                    // Check Branch header click (row 1 inside git pane)
                    if y == self.git_area.y + 1 {
                        self.open_branch_modal();
                        return;
                    }

                    // Check footer bar clicks
                    if y >= self.git_area.bottom().saturating_sub(2) {
                        let rel_x = x.saturating_sub(self.git_area.x);
                        let w = self.git_area.width;
                        let btn_w = (w / 8).max(1);
                        match rel_x / btn_w {
                            0 => self.stage_selected_git_item(),
                            1 => {
                                self.show_commit_modal = true;
                                self.commit_message_input.clear();
                            }
                            2 => self.push_git(),
                            3 => self.pull_git(),
                            4 => self.open_branch_modal(),
                            5 => self.open_stash_modal(),
                            6 => self.diff_selected_git_item(),
                            _ => {
                                self.git_info = GitRepoInfo::fetch(&self.workspace_dir.to_string_lossy());
                                self.status_message = Some(("Refreshed Git status".to_string(), std::time::Instant::now()));
                            }
                        }
                        return;
                    }

                    // Check item clicks in list
                    let staged_len = self.git_info.staged_files.len();
                    let unstaged_len = self.git_info.unstaged_files.len();
                    let total = staged_len + unstaged_len;

                    let list_start_y = self.git_area.y + 2;
                    if y >= list_start_y && y < self.git_area.bottom().saturating_sub(2) {
                        let row = (y - list_start_y) as usize + self.git_scroll;
                        if row < total {
                            if self.git_selected_idx == row {
                                self.stage_selected_git_item();
                            } else {
                                self.git_selected_idx = row;
                            }
                        }
                    }
                    return;
                }

                // Check Agents pane clicks
                if self.agents_area.contains(ratatui::layout::Position { x, y }) {
                    if y == self.agents_area.y && x >= self.agents_area.right().saturating_sub(4) {
                        self.show_agents = false;
                        return;
                    }
                    self.active_pane = ActivePane::Agents;
                    let inner_y = self.agents_area.y + 1;
                    if y >= inner_y {
                        let clicked_agent = (y - inner_y) as usize / 4 + self.agents_scroll;
                        if clicked_agent < self.agents.len() {
                            self.active_agent_idx = clicked_agent;
                        }
                    }
                }
            }
            crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
                if let Some(drag) = self.active_splitter_drag {
                    match drag {
                        SplitterKind::LeftColumn => {
                            if self.body_area.width > 0 {
                                let offset = x.saturating_sub(self.body_area.x);
                                let pct = ((offset as u32 * 100) / self.body_area.width as u32) as u16;
                                self.left_col_pct = pct.clamp(10, 45);
                            }
                        }
                        SplitterKind::RightColumn => {
                            if self.body_area.width > 0 {
                                let offset = self.body_area.right().saturating_sub(x);
                                let pct = ((offset as u32 * 100) / self.body_area.width as u32) as u16;
                                self.right_col_pct = pct.clamp(12, 50);
                            }
                        }
                        SplitterKind::CenterRow => {
                            let total_h = self.preview_area.height + self.terminal_area.height;
                            if total_h > 0 {
                                let offset = y.saturating_sub(self.preview_area.y);
                                let pct = ((offset as u32 * 100) / total_h as u32) as u16;
                                self.center_split_pct = pct.clamp(15, 85);
                            }
                        }
                        SplitterKind::RightRow => {
                            let total_h = self.git_area.height + self.agents_area.height;
                            if total_h > 0 {
                                let offset = y.saturating_sub(self.git_area.y);
                                let pct = ((offset as u32 * 100) / total_h as u32) as u16;
                                self.right_split_pct = pct.clamp(15, 85);
                            }
                        }
                    }
                }
            }
            crossterm::event::MouseEventKind::Up(_) => {
                self.active_splitter_drag = None;
            }
            crossterm::event::MouseEventKind::ScrollDown => {
                if self.preview_tabs_area.contains(ratatui::layout::Position { x, y }) {
                    self.next_editor_tab();
                } else if self.preview_area.contains(ratatui::layout::Position { x, y }) {
                    if let Some((_, lines)) = &self.preview_content {
                        if self.preview_scroll + 2 < lines.len() {
                            self.preview_scroll += 2;
                        }
                    }
                } else if self.explorer_area.contains(ratatui::layout::Position { x, y }) {
                    self.explorer.next();
                    self.update_file_preview();
                } else if self.git_area.contains(ratatui::layout::Position { x, y }) {
                    let total = self.total_git_items();
                    if self.git_scroll + 1 < total {
                        self.git_scroll += 1;
                    }
                } else if self.agents_area.contains(ratatui::layout::Position { x, y }) {
                    if !self.agents.is_empty() {
                        self.active_agent_idx = (self.active_agent_idx + 1).min(self.agents.len().saturating_sub(1));
                    }
                } else if self.terminal_tabs_area.contains(ratatui::layout::Position { x, y }) {
                    if !self.terminals.is_empty() {
                        self.active_terminal_idx = (self.active_terminal_idx + 1) % self.terminals.len();
                    }
                } else if self.terminal_area.contains(ratatui::layout::Position { x, y }) {
                    if let Some(session) = self.terminals.get(self.active_terminal_idx) {
                        let rel_x = x.saturating_sub(self.terminal_area.x);
                        let rel_y = y.saturating_sub(self.terminal_area.y);
                        session.handle_mouse_scroll(false, rel_x, rel_y);
                    }
                }
            }
            crossterm::event::MouseEventKind::ScrollUp => {
                if self.preview_tabs_area.contains(ratatui::layout::Position { x, y }) {
                    self.prev_editor_tab();
                } else if self.preview_area.contains(ratatui::layout::Position { x, y }) {
                    self.preview_scroll = self.preview_scroll.saturating_sub(2);
                } else if self.explorer_area.contains(ratatui::layout::Position { x, y }) {
                    self.explorer.prev();
                    self.update_file_preview();
                } else if self.git_area.contains(ratatui::layout::Position { x, y }) {
                    self.git_scroll = self.git_scroll.saturating_sub(1);
                } else if self.agents_area.contains(ratatui::layout::Position { x, y }) {
                    if self.active_agent_idx > 0 {
                        self.active_agent_idx -= 1;
                    }
                } else if self.terminal_tabs_area.contains(ratatui::layout::Position { x, y }) {
                    if !self.terminals.is_empty() {
                        self.active_terminal_idx = (self.active_terminal_idx + self.terminals.len() - 1) % self.terminals.len();
                    }
                } else if self.terminal_area.contains(ratatui::layout::Position { x, y }) {
                    if let Some(session) = self.terminals.get(self.active_terminal_idx) {
                        let rel_x = x.saturating_sub(self.terminal_area.x);
                        let rel_y = y.saturating_sub(self.terminal_area.y);
                        session.handle_mouse_scroll(true, rel_x, rel_y);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        // Modal overlays consume keys first
        if self.show_help {
            if key.code == KeyCode::Esc || key.code == KeyCode::F(1) {
                self.show_help = false;
            }
            return;
        }

        if self.explorer_op_modal != ExplorerOpModal::None {
            match &mut self.explorer_op_modal {
                ExplorerOpModal::None => unreachable!(),
                ExplorerOpModal::NewItem { input, is_folder, .. } => {
                    match key.code {
                        KeyCode::Esc => {
                            self.explorer_op_modal = ExplorerOpModal::None;
                        }
                        KeyCode::Tab => {
                            *is_folder = !*is_folder;
                        }
                        KeyCode::Enter => {
                            self.execute_new_item();
                        }
                        KeyCode::Backspace => {
                            input.pop();
                        }
                        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) => {
                            input.push(c);
                        }
                        _ => {}
                    }
                }
                ExplorerOpModal::Rename { input, .. } => {
                    match key.code {
                        KeyCode::Esc => {
                            self.explorer_op_modal = ExplorerOpModal::None;
                        }
                        KeyCode::Enter => {
                            self.execute_rename_item();
                        }
                        KeyCode::Backspace => {
                            input.pop();
                        }
                        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) => {
                            input.push(c);
                        }
                        _ => {}
                    }
                }
                ExplorerOpModal::DeleteConfirm { .. } => {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                            self.execute_delete_item();
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            self.explorer_op_modal = ExplorerOpModal::None;
                        }
                        _ => {}
                    }
                }
            }
            return;
        }

        if self.show_terminal_menu {
            match key.code {
                KeyCode::Esc => self.show_terminal_menu = false,
                KeyCode::Up | KeyCode::Char('k') => {
                    self.terminal_menu_selected_idx = self.terminal_menu_selected_idx.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.terminal_menu_selected_idx < 3 {
                        self.terminal_menu_selected_idx += 1;
                    }
                }
                KeyCode::Char('1') => {
                    let _ = self.spawn_selected_terminal(0);
                    self.show_terminal_menu = false;
                }
                KeyCode::Char('2') => {
                    let _ = self.spawn_selected_terminal(1);
                    self.show_terminal_menu = false;
                }
                KeyCode::Char('3') => {
                    let _ = self.spawn_selected_terminal(2);
                    self.show_terminal_menu = false;
                }
                KeyCode::Char('4') => {
                    let _ = self.spawn_selected_terminal(3);
                    self.show_terminal_menu = false;
                }
                KeyCode::Enter => {
                    let _ = self.spawn_selected_terminal(self.terminal_menu_selected_idx);
                    self.show_terminal_menu = false;
                }
                _ => {}
            }
            return;
        }

        if self.show_agent_launch_modal {
            match key.code {
                KeyCode::Esc => self.show_agent_launch_modal = false,
                KeyCode::Up | KeyCode::Char('k') => {
                    self.agent_launch_selected_idx = self.agent_launch_selected_idx.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.agent_launch_selected_idx < 2 {
                        self.agent_launch_selected_idx += 1;
                    }
                }
                KeyCode::Char('1') => {
                    self.agent_launch_selected_idx = 0;
                }
                KeyCode::Char('2') => {
                    self.agent_launch_selected_idx = 1;
                }
                KeyCode::Char('3') => {
                    self.agent_launch_selected_idx = 2;
                }
                KeyCode::Enter => {
                    match self.agent_launch_selected_idx {
                        0 => {
                            let _ = self.launch_agent("Antigravity", "agy", &[], "Interactive Coding Agent");
                        }
                        1 => {
                            let _ = self.launch_agent("Claude Code", "claude", &[], "Autonomous Agent Session");
                        }
                        2 => {
                            let shell = if cfg!(windows) { "powershell.exe" } else { "bash" };
                            let _ = self.launch_agent("Worktree Shell", shell, &[], "Isolated Worktree Shell");
                        }
                        _ => {}
                    }
                    self.show_agent_launch_modal = false;
                }
                _ => {}
            }
            return;
        }

        if self.show_commit_modal {
            match key.code {
                KeyCode::Esc => {
                    self.show_commit_modal = false;
                }
                KeyCode::Enter => {
                    self.execute_commit();
                }
                KeyCode::Backspace => {
                    self.commit_message_input.pop();
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) => {
                    self.commit_message_input.push(c);
                }
                _ => {}
            }
            return;
        }

        if self.show_diff_modal {
            match key.code {
                KeyCode::Esc => self.show_diff_modal = false,
                KeyCode::Up | KeyCode::Char('k') => {
                    self.diff_modal_scroll = self.diff_modal_scroll.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.diff_modal_scroll += 1;
                }
                KeyCode::PageUp => {
                    self.diff_modal_scroll = self.diff_modal_scroll.saturating_sub(15);
                }
                KeyCode::PageDown => {
                    self.diff_modal_scroll += 15;
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    let _ = self.merge_active_agent();
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    let _ = self.discard_active_agent();
                }
                KeyCode::Char('s') => {
                    self.stage_selected_git_item();
                }
                _ => {}
            }
            return;
        }

        if self.show_branch_modal {
            if self.branch_is_creating {
                match key.code {
                    KeyCode::Esc => {
                        self.branch_is_creating = false;
                        self.branch_new_name.clear();
                    }
                    KeyCode::Enter => {
                        self.create_and_switch_branch();
                    }
                    KeyCode::Backspace => {
                        self.branch_new_name.pop();
                    }
                    KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) => {
                        self.branch_new_name.push(c);
                    }
                    _ => {}
                }
                return;
            }

            let filtered_len = self.get_filtered_branches().len();
            match key.code {
                KeyCode::Esc => {
                    self.show_branch_modal = false;
                }
                KeyCode::Down | KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) || self.branch_filter_query.is_empty() => {
                    if filtered_len > 0 && self.branch_selected_idx + 1 < filtered_len {
                        self.branch_selected_idx += 1;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) || self.branch_filter_query.is_empty() => {
                    self.branch_selected_idx = self.branch_selected_idx.saturating_sub(1);
                }
                KeyCode::Enter => {
                    self.checkout_selected_branch();
                }
                KeyCode::Char('n') if self.branch_filter_query.is_empty() => {
                    self.branch_is_creating = true;
                    self.branch_new_name.clear();
                }
                KeyCode::Char('d') if self.branch_filter_query.is_empty() => {
                    self.delete_selected_branch(false);
                }
                KeyCode::Char('D') if self.branch_filter_query.is_empty() => {
                    self.delete_selected_branch(true);
                }
                KeyCode::Char('r') if self.branch_filter_query.is_empty() => {
                    if let Ok(branches) = GitRepoInfo::list_branches(&self.workspace_dir.to_string_lossy()) {
                        self.branch_items = branches;
                        self.status_message = Some(("Refreshed branch list".to_string(), std::time::Instant::now()));
                    }
                }
                KeyCode::Backspace => {
                    self.branch_filter_query.pop();
                    self.branch_selected_idx = 0;
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) => {
                    self.branch_filter_query.push(c);
                    self.branch_selected_idx = 0;
                }
                _ => {}
            }
            return;
        }

        if self.show_stash_modal {
            if self.stash_is_creating {
                match key.code {
                    KeyCode::Esc => {
                        self.stash_is_creating = false;
                        self.stash_new_message.clear();
                    }
                    KeyCode::Enter => {
                        self.save_new_stash();
                    }
                    KeyCode::Backspace => {
                        self.stash_new_message.pop();
                    }
                    KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.stash_include_untracked = !self.stash_include_untracked;
                    }
                    KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) => {
                        self.stash_new_message.push(c);
                    }
                    _ => {}
                }
                return;
            }

            let stashes_len = self.stash_items.len();
            match key.code {
                KeyCode::Esc => {
                    self.show_stash_modal = false;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if stashes_len > 0 && self.stash_selected_idx + 1 < stashes_len {
                        self.stash_selected_idx += 1;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.stash_selected_idx = self.stash_selected_idx.saturating_sub(1);
                }
                KeyCode::Enter | KeyCode::Char('p') => {
                    self.pop_selected_stash();
                }
                KeyCode::Char('a') => {
                    self.apply_selected_stash();
                }
                KeyCode::Char('s') => {
                    self.stash_is_creating = true;
                    self.stash_new_message.clear();
                }
                KeyCode::Char('d') => {
                    self.drop_selected_stash();
                }
                KeyCode::Char('v') => {
                    self.view_selected_stash_diff();
                }
                KeyCode::Char('r') => {
                    if let Ok(stashes) = GitRepoInfo::list_stashes(&self.workspace_dir.to_string_lossy()) {
                        self.stash_items = stashes;
                        self.status_message = Some(("Refreshed stash list".to_string(), std::time::Instant::now()));
                    }
                }
                _ => {}
            }
            return;
        }

        if self.show_theme_modal {
            let themes = crate::theme::ColorTheme::ALL;
            match key.code {
                KeyCode::Esc => {
                    self.color_theme = self.previous_color_theme;
                    self.show_theme_modal = false;
                }
                KeyCode::Enter => {
                    self.show_theme_modal = false;
                    self.status_message = Some((
                        format!("Theme switched to: {}", self.color_theme.name()),
                        std::time::Instant::now(),
                    ));
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.theme_modal_selected_idx = self.theme_modal_selected_idx.saturating_sub(1);
                    self.color_theme = themes[self.theme_modal_selected_idx];
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.theme_modal_selected_idx + 1 < themes.len() {
                        self.theme_modal_selected_idx += 1;
                        self.color_theme = themes[self.theme_modal_selected_idx];
                    }
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    if let Some(digit) = c.to_digit(10) {
                        let idx = if digit == 0 { 9 } else { (digit as usize).saturating_sub(1) };
                        if idx < themes.len() {
                            self.theme_modal_selected_idx = idx;
                            self.color_theme = themes[idx];
                        }
                    }
                }
                _ => {}
            }
            return;
        }

        if self.show_command_palette {
            let items = self.get_palette_items();
            match key.code {
                KeyCode::Esc => {
                    self.show_command_palette = false;
                }
                KeyCode::Up => {
                    self.palette_selected_idx = self.palette_selected_idx.saturating_sub(1);
                }
                KeyCode::Down => {
                    if !items.is_empty() && self.palette_selected_idx + 1 < items.len() {
                        self.palette_selected_idx += 1;
                    }
                }
                KeyCode::Enter => {
                    if let Some(item) = items.get(self.palette_selected_idx) {
                        let action = item.action.clone();
                        self.show_command_palette = false;
                        self.execute_palette_action(action);
                    } else {
                        self.show_command_palette = false;
                    }
                }
                KeyCode::Backspace => {
                    self.palette_query.pop();
                    self.palette_selected_idx = 0;
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) => {
                    self.palette_query.push(c);
                    self.palette_selected_idx = 0;
                }
                _ => {}
            }
            return;
        }

        if self.show_global_search {
            match key.code {
                KeyCode::Esc => {
                    self.show_global_search = false;
                }
                KeyCode::Up => {
                    self.global_search_selected_idx = self.global_search_selected_idx.saturating_sub(1);
                }
                KeyCode::Down => {
                    if !self.global_search_results.is_empty() && self.global_search_selected_idx + 1 < self.global_search_results.len() {
                        self.global_search_selected_idx += 1;
                    }
                }
                KeyCode::Enter => {
                    if let Some(result) = self.global_search_results.get(self.global_search_selected_idx).cloned() {
                        self.show_global_search = false;
                        self.open_file(&result.file_path, true);
                        self.active_pane = ActivePane::FilePreview;
                        let line_idx = result.line_num.saturating_sub(1);
                        self.editor_cursor = (line_idx, result.match_start);
                        let view_height = self.preview_area.height.saturating_sub(4) as usize;
                        if view_height > 0 {
                            self.preview_scroll = line_idx.saturating_sub(view_height / 2);
                        }
                        self.status_message = Some((
                            format!("Jumped to {}:{}", result.rel_path, result.line_num),
                            std::time::Instant::now(),
                        ));
                    }
                }
                KeyCode::Backspace => {
                    self.global_search_query.pop();
                    self.run_global_search();
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) => {
                    self.global_search_query.push(c);
                    self.run_global_search();
                }
                _ => {}
            }
            return;
        }

        // Global hotkeys (always active)
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
            self.should_quit = true;
            return;
        }

        // Open Workspace Launcher / Welcome Screen: Ctrl+O or F10
        if (key.modifiers.contains(KeyModifiers::CONTROL) && (key.code == KeyCode::Char('o') || key.code == KeyCode::Char('O')))
            || key.code == KeyCode::F(10)
        {
            self.toggle_welcome_screen();
            return;
        }

        // If Welcome Screen is open, it captures all navigation & workspace selection
        if self.show_welcome_screen {
            self.handle_welcome_key_event(key);
            return;
        }

        // Terminal / Workspace Zoom: Ctrl+= (or Ctrl++), Ctrl+-, Ctrl+0
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('=') | KeyCode::Char('+') => {
                    self.zoom_in_terminal();
                    return;
                }
                KeyCode::Char('-') => {
                    self.zoom_out_terminal();
                    return;
                }
                KeyCode::Char('0') => {
                    self.zoom_reset_terminal();
                    return;
                }
                _ => {}
            }
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('p') {
            self.palette_query.clear();
            self.palette_selected_idx = 0;
            self.show_command_palette = true;
            return;
        }

        // Workspace Search: Ctrl+Shift+F (or Ctrl+F when outside FilePreview)
        if key.modifiers.contains(KeyModifiers::CONTROL) && (key.code == KeyCode::Char('F') || (key.modifiers.contains(KeyModifiers::SHIFT) && key.code == KeyCode::Char('f')) || (self.active_pane != ActivePane::FilePreview && key.code == KeyCode::Char('f'))) {
            self.show_global_search = true;
            self.global_search_query.clear();
            self.global_search_results.clear();
            self.global_search_selected_idx = 0;
            return;
        }

        // Universal Function Keys: never conflict with CLI tools (Claude, Agy, etc.)
        match key.code {
            KeyCode::F(1) => {
                self.show_help = !self.show_help;
                return;
            }
            KeyCode::F(2) => {
                self.show_explorer = !self.show_explorer;
                return;
            }
            KeyCode::F(3) => {
                self.show_git = !self.show_git;
                return;
            }
            KeyCode::F(4) => {
                self.show_agents = !self.show_agents;
                return;
            }
            KeyCode::F(5) => {
                self.show_preview = !self.show_preview;
                return;
            }
            KeyCode::F(6) => {
                self.cycle_active_pane();
                return;
            }
            KeyCode::F(7) => {
                if !self.terminals.is_empty() {
                    self.active_terminal_idx = (self.active_terminal_idx + self.terminals.len() - 1) % self.terminals.len();
                    self.status_message = Some((format!("Terminal Tab: {}", self.terminals[self.active_terminal_idx].title), std::time::Instant::now()));
                }
                return;
            }
            KeyCode::F(8) => {
                if !self.terminals.is_empty() {
                    self.active_terminal_idx = (self.active_terminal_idx + 1) % self.terminals.len();
                    self.status_message = Some((format!("Terminal Tab: {}", self.terminals[self.active_terminal_idx].title), std::time::Instant::now()));
                }
                return;
            }
            KeyCode::F(11) => {
                self.is_maximized = !self.is_maximized;
                return;
            }
            KeyCode::F(12) => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.color_theme = self.color_theme.next();
                    self.status_message = Some((
                        format!("Theme switched to: {}", self.color_theme.name()),
                        std::time::Instant::now(),
                    ));
                    return;
                }
                self.show_theme_modal = !self.show_theme_modal;
                if self.show_theme_modal {
                    self.previous_color_theme = self.color_theme;
                    self.theme_modal_selected_idx = crate::theme::ColorTheme::ALL
                        .iter()
                        .position(|t| *t == self.color_theme)
                        .unwrap_or(0);
                } else {
                    self.color_theme = self.previous_color_theme;
                }
                return;
            }
            _ => {}
        }

        // Universal Tab Switching: Ctrl+Tab, Ctrl+Shift+Tab, Alt+1..9
        if key.modifiers.contains(KeyModifiers::CONTROL) && (key.code == KeyCode::Tab || key.code == KeyCode::BackTab) {
            if self.active_pane == ActivePane::FilePreview {
                if key.modifiers.contains(KeyModifiers::SHIFT) || key.code == KeyCode::BackTab {
                    self.prev_editor_tab();
                } else {
                    self.next_editor_tab();
                }
                return;
            }
            if !self.terminals.is_empty() {
                if key.modifiers.contains(KeyModifiers::SHIFT) || key.code == KeyCode::BackTab {
                    self.active_terminal_idx = (self.active_terminal_idx + self.terminals.len() - 1) % self.terminals.len();
                } else {
                    self.active_terminal_idx = (self.active_terminal_idx + 1) % self.terminals.len();
                }
                self.status_message = Some((format!("Terminal Tab: {}", self.terminals[self.active_terminal_idx].title), std::time::Instant::now()));
            }
            return;
        }

        if key.modifiers.contains(KeyModifiers::ALT) {
            if let KeyCode::Char(c) = key.code {
                if let Some(d) = c.to_digit(10) {
                    let target = (d as usize).saturating_sub(1);
                    if target < self.terminals.len() {
                        self.active_terminal_idx = target;
                        self.status_message = Some((format!("Terminal Tab: {}", self.terminals[target].title), std::time::Instant::now()));
                        return;
                    }
                }
            }
        }

        // When NOT inside Terminal, hotkeys like Ctrl+p, Ctrl+a, and Tab are active
        if self.active_pane != ActivePane::Terminal {
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('p') {
                self.show_command_palette = true;
                return;
            }

            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('a') {
                self.show_agent_launch_modal = true;
                return;
            }

            if key.code == KeyCode::Tab {
                self.cycle_active_pane();
                return;
            }
        }

        // Card Resizing Hotkeys (Alt keys - accessible globally or outside terminal)
        if key.modifiers.contains(KeyModifiers::ALT) {
            match key.code {
                KeyCode::Char('[') => {
                    self.left_col_pct = self.left_col_pct.saturating_sub(2).clamp(10, 45);
                    self.status_message = Some((format!("Explorer width: {}%", self.left_col_pct), std::time::Instant::now()));
                    return;
                }
                KeyCode::Char(']') => {
                    self.left_col_pct = (self.left_col_pct + 2).clamp(10, 45);
                    self.status_message = Some((format!("Explorer width: {}%", self.left_col_pct), std::time::Instant::now()));
                    return;
                }
                KeyCode::Char('{') => {
                    self.right_col_pct = self.right_col_pct.saturating_sub(2).clamp(12, 50);
                    self.status_message = Some((format!("Right panel width: {}%", self.right_col_pct), std::time::Instant::now()));
                    return;
                }
                KeyCode::Char('}') => {
                    self.right_col_pct = (self.right_col_pct + 2).clamp(12, 50);
                    self.status_message = Some((format!("Right panel width: {}%", self.right_col_pct), std::time::Instant::now()));
                    return;
                }
                KeyCode::Char('-') => {
                    self.center_split_pct = self.center_split_pct.saturating_sub(3).clamp(15, 85);
                    self.status_message = Some((format!("Preview: {}% │ Terminal: {}%", self.center_split_pct, 100 - self.center_split_pct), std::time::Instant::now()));
                    return;
                }
                KeyCode::Char('=') | KeyCode::Char('+') => {
                    self.center_split_pct = (self.center_split_pct + 3).clamp(15, 85);
                    self.status_message = Some((format!("Preview: {}% │ Terminal: {}%", self.center_split_pct, 100 - self.center_split_pct), std::time::Instant::now()));
                    return;
                }
                _ => {}
            }
        }

        // Pane-specific routing
        match self.active_pane {
            ActivePane::Explorer => match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.explorer.next();
                    self.update_file_preview();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.explorer.prev();
                    self.update_file_preview();
                }
                KeyCode::Char(' ') => {
                    self.explorer.toggle_selected();
                    self.update_file_preview();
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    if self.explorer.expand_selected() {
                        self.update_file_preview();
                    }
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    if self.explorer.collapse_selected() {
                        self.update_file_preview();
                    }
                }
                // File Operations: New, Rename, Delete
                KeyCode::Char('a') | KeyCode::Char('n') => {
                    self.prompt_new_item();
                }
                KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.explorer.refresh();
                    self.update_file_preview();
                    self.status_message = Some(("Refreshed Explorer".to_string(), std::time::Instant::now()));
                }
                KeyCode::Char('R') => {
                    self.explorer.refresh();
                    self.update_file_preview();
                    self.status_message = Some(("Refreshed Explorer".to_string(), std::time::Instant::now()));
                }
                KeyCode::Char('r') | KeyCode::F(2) => {
                    self.prompt_rename_item();
                }
                KeyCode::Char('d') | KeyCode::Delete => {
                    self.prompt_delete_item();
                }
                KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(entry) = self.explorer.selected_entry() {
                        let path = entry.path.clone();
                        self.open_in_external_editor(&path);
                    }
                }
                KeyCode::Enter => {
                    if let Some(entry) = self.explorer.selected_entry() {
                        if entry.is_dir {
                            self.explorer.toggle_selected();
                            self.update_file_preview();
                        } else {
                            let path = entry.path.clone();
                            self.open_file(&path, true);
                            self.active_pane = ActivePane::FilePreview;
                        }
                    }
                }
                _ => {}
            },
            ActivePane::FilePreview => {
                // Find in File toggle: Ctrl+f
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('f') {
                    self.show_find_in_file = !self.show_find_in_file;
                    if self.show_find_in_file {
                        self.update_find_matches();
                    }
                    return;
                }

                // If Find bar is active, route search navigation & typing
                if self.show_find_in_file {
                    match key.code {
                        KeyCode::Esc => {
                            self.show_find_in_file = false;
                            return;
                        }
                        KeyCode::Enter | KeyCode::Down | KeyCode::F(3) => {
                            if key.modifiers.contains(KeyModifiers::SHIFT) {
                                self.prev_find_match();
                            } else {
                                self.next_find_match();
                            }
                            return;
                        }
                        KeyCode::Up => {
                            self.prev_find_match();
                            return;
                        }
                        KeyCode::Backspace => {
                            self.find_query.pop();
                            self.update_find_matches();
                            return;
                        }
                        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) => {
                            self.find_query.push(c);
                            self.update_find_matches();
                            return;
                        }
                        _ => {}
                    }
                }

                // Undo: Ctrl+z
                if key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::SHIFT) && key.code == KeyCode::Char('z') {
                    self.editor_undo();
                    return;
                }
                // Redo: Ctrl+y or Ctrl+Shift+Z
                if (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('y'))
                    || (key.modifiers.contains(KeyModifiers::CONTROL) && key.modifiers.contains(KeyModifiers::SHIFT) && (key.code == KeyCode::Char('Z') || key.code == KeyCode::Char('z'))) {
                    self.editor_redo();
                    return;
                }
                // Duplicate Line: Ctrl+d
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('d') {
                    self.duplicate_current_line();
                    return;
                }
                // Delete Line: Ctrl+k or Ctrl+Shift+K
                if key.modifiers.contains(KeyModifiers::CONTROL) && (key.code == KeyCode::Char('k') || key.code == KeyCode::Char('K')) {
                    self.delete_current_line();
                    return;
                }
                // Move Line Up: Alt+Up
                if key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Up {
                    self.move_current_line_up();
                    return;
                }
                // Move Line Down: Alt+Down
                if key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Down {
                    self.move_current_line_down();
                    return;
                }
                // Toggle Comment: Ctrl+/ (or Ctrl+_ or Ctrl+?)
                if key.modifiers.contains(KeyModifiers::CONTROL) && (key.code == KeyCode::Char('/') || key.code == KeyCode::Char('_') || key.code == KeyCode::Char('?')) {
                    self.toggle_comment_on_current_line();
                    return;
                }

                // Save with Ctrl+s
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
                    self.save_editor_content();
                    return;
                }
                // Close active editor tab with Ctrl+w
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('w') {
                    self.close_editor_tab(self.active_editor_tab_idx);
                    return;
                }
                // Editor tab cycling shortcuts: F9 / F10 or Alt+Left / Alt+Right
                if key.code == KeyCode::F(9) || (key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Left) {
                    self.prev_editor_tab();
                    return;
                }
                if key.code == KeyCode::F(10) || (key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Right) {
                    self.next_editor_tab();
                    return;
                }
                // Open in VS Code with Ctrl+e or Ctrl+o
                if key.modifiers.contains(KeyModifiers::CONTROL) && (key.code == KeyCode::Char('e') || key.code == KeyCode::Char('o')) {
                    if let Some(path) = &self.editor_file_path {
                        let path_clone = path.clone();
                        self.open_in_external_editor(&path_clone);
                    }
                    return;
                }
                // Esc returns to Explorer pane
                if key.code == KeyCode::Esc {
                    self.active_pane = ActivePane::Explorer;
                    return;
                }

                // Direct Editing & Navigation
                match key.code {
                    KeyCode::Up => {
                        if self.editor_cursor.0 > 0 {
                            self.editor_cursor.0 -= 1;
                            if let Some((_, lines)) = &self.preview_content {
                                if let Some(line) = lines.get(self.editor_cursor.0) {
                                    self.editor_cursor.1 = self.editor_cursor.1.min(line.chars().count());
                                }
                            }
                            if self.editor_cursor.0 < self.preview_scroll {
                                self.preview_scroll = self.editor_cursor.0;
                            }
                        }
                    }
                    KeyCode::Down => {
                        if let Some((_, lines)) = &self.preview_content {
                            if self.editor_cursor.0 + 1 < lines.len() {
                                self.editor_cursor.0 += 1;
                                if let Some(line) = lines.get(self.editor_cursor.0) {
                                    self.editor_cursor.1 = self.editor_cursor.1.min(line.chars().count());
                                }
                                let view_height = self.preview_area.height.saturating_sub(3) as usize;
                                if self.editor_cursor.0 >= self.preview_scroll + view_height.max(1) {
                                    self.preview_scroll = self.editor_cursor.0.saturating_sub(view_height.saturating_sub(1));
                                }
                            }
                        }
                    }
                    KeyCode::Left => {
                        if self.editor_cursor.1 > 0 {
                            self.editor_cursor.1 -= 1;
                        } else if self.editor_cursor.0 > 0 {
                            self.editor_cursor.0 -= 1;
                            if let Some((_, lines)) = &self.preview_content {
                                if let Some(line) = lines.get(self.editor_cursor.0) {
                                    self.editor_cursor.1 = line.chars().count();
                                }
                            }
                        }
                    }
                    KeyCode::Right => {
                        if let Some((_, lines)) = &self.preview_content {
                            if let Some(line) = lines.get(self.editor_cursor.0) {
                                if self.editor_cursor.1 < line.chars().count() {
                                    self.editor_cursor.1 += 1;
                                } else if self.editor_cursor.0 + 1 < lines.len() {
                                    self.editor_cursor.0 += 1;
                                    self.editor_cursor.1 = 0;
                                }
                            }
                        }
                    }
                    KeyCode::Home => {
                        if let Some((_, lines)) = &self.preview_content {
                            if let Some(line) = lines.get(self.editor_cursor.0) {
                                let first_non_ws = line.chars().take_while(|c| c.is_whitespace()).count();
                                if self.editor_cursor.1 == first_non_ws {
                                    self.editor_cursor.1 = 0;
                                } else {
                                    self.editor_cursor.1 = first_non_ws;
                                }
                            } else {
                                self.editor_cursor.1 = 0;
                            }
                        } else {
                            self.editor_cursor.1 = 0;
                        }
                    }
                    KeyCode::End => {
                        if let Some((_, lines)) = &self.preview_content {
                            if let Some(line) = lines.get(self.editor_cursor.0) {
                                self.editor_cursor.1 = line.chars().count();
                            }
                        }
                    }
                    KeyCode::PageUp => {
                        let step = 15;
                        self.editor_cursor.0 = self.editor_cursor.0.saturating_sub(step);
                        self.preview_scroll = self.preview_scroll.saturating_sub(step);
                    }
                    KeyCode::PageDown => {
                        let step = 15;
                        if let Some((_, lines)) = &self.preview_content {
                            let max_line = lines.len().saturating_sub(1);
                            self.editor_cursor.0 = (self.editor_cursor.0 + step).min(max_line);
                            self.preview_scroll = (self.preview_scroll + step).min(max_line);
                        }
                    }
                    KeyCode::Tab => {
                        self.push_editor_undo_snapshot();
                        if let Some((_, lines)) = &mut self.preview_content {
                            if lines.is_empty() {
                                lines.push(String::new());
                            }
                            let (row, col) = self.editor_cursor;
                            if let Some(line) = lines.get_mut(row) {
                                let byte_idx = line.char_indices().nth(col).map(|(i, _)| i).unwrap_or(line.len());
                                line.insert_str(byte_idx, "    ");
                                self.editor_cursor.1 += 4;
                                self.editor_dirty = true;
                            }
                        }
                    }
                    KeyCode::Enter => {
                        self.push_editor_undo_snapshot();
                        if let Some((_, lines)) = &mut self.preview_content {
                            if lines.is_empty() {
                                lines.push(String::new());
                            }
                            let (row, col) = self.editor_cursor;
                            if let Some(line) = lines.get_mut(row) {
                                let byte_idx = line.char_indices().nth(col).map(|(i, _)| i).unwrap_or(line.len());
                                let rest = line.split_off(byte_idx);
                                lines.insert(row + 1, rest);
                                self.editor_cursor.0 += 1;
                                self.editor_cursor.1 = 0;
                                self.editor_dirty = true;

                                let view_height = self.preview_area.height.saturating_sub(3) as usize;
                                if self.editor_cursor.0 >= self.preview_scroll + view_height.max(1) {
                                    self.preview_scroll += 1;
                                }
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        self.push_editor_undo_snapshot();
                        if let Some((_, lines)) = &mut self.preview_content {
                            let (row, col) = self.editor_cursor;
                            if col > 0 {
                                if let Some(line) = lines.get_mut(row) {
                                    let byte_idx = line.char_indices().nth(col - 1).map(|(i, _)| i).unwrap_or(0);
                                    line.remove(byte_idx);
                                    self.editor_cursor.1 -= 1;
                                    self.editor_dirty = true;
                                }
                            } else if row > 0 {
                                let curr_line = lines.remove(row);
                                let prev_line = &mut lines[row - 1];
                                self.editor_cursor.1 = prev_line.chars().count();
                                prev_line.push_str(&curr_line);
                                self.editor_cursor.0 -= 1;
                                self.editor_dirty = true;
                                if self.editor_cursor.0 < self.preview_scroll {
                                    self.preview_scroll = self.editor_cursor.0;
                                }
                            }
                        }
                    }
                    KeyCode::Delete => {
                        self.push_editor_undo_snapshot();
                        if let Some((_, lines)) = &mut self.preview_content {
                            let (row, col) = self.editor_cursor;
                            if let Some(line) = lines.get_mut(row) {
                                if col < line.chars().count() {
                                    let byte_idx = line.char_indices().nth(col).map(|(i, _)| i).unwrap_or(line.len());
                                    line.remove(byte_idx);
                                    self.editor_dirty = true;
                                } else if row + 1 < lines.len() {
                                    let next_line = lines.remove(row + 1);
                                    let curr = &mut lines[row];
                                    curr.push_str(&next_line);
                                    self.editor_dirty = true;
                                }
                            }
                        }
                    }
                    KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) => {
                        self.push_editor_undo_snapshot();
                        if let Some((_, lines)) = &mut self.preview_content {
                            if lines.is_empty() {
                                lines.push(String::new());
                            }
                            let (row, col) = self.editor_cursor;
                            if let Some(line) = lines.get_mut(row) {
                                let byte_idx = line.char_indices().nth(col).map(|(i, _)| i).unwrap_or(line.len());
                                line.insert(byte_idx, c);
                                self.editor_cursor.1 += 1;
                                self.editor_dirty = true;
                            }
                        }
                    }
                    _ => {}
                }
                if self.editor_dirty {
                    self.mark_editor_dirty();
                }
            },
            ActivePane::Terminal => {
                // Escape chord: Ctrl+b returns focus to TIDE Explorer pane
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('b') {
                    self.active_pane = ActivePane::Explorer;
                    return;
                }

                // New tab: Ctrl+t
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('t') {
                    self.show_terminal_menu = true;
                    self.terminal_menu_selected_idx = 0;
                    return;
                }

                // Close tab: Ctrl+w
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('w') {
                    if self.terminals.len() > 1 {
                        let closed_idx = self.active_terminal_idx;
                        self.terminals.remove(closed_idx);
                        for agent in &mut self.agents {
                            if let Some(ref mut idx) = agent.terminal_idx {
                                if *idx == closed_idx {
                                    agent.terminal_idx = None;
                                } else if *idx > closed_idx {
                                    *idx -= 1;
                                }
                            }
                        }
                        if self.active_terminal_idx >= self.terminals.len() {
                            self.active_terminal_idx = self.terminals.len().saturating_sub(1);
                        }
                    }
                    return;
                }

                // Switch tabs: Alt+1..9
                if key.modifiers.contains(KeyModifiers::ALT) {
                    if let KeyCode::Char(c) = key.code {
                        if let Some(d) = c.to_digit(10) {
                            let target = (d as usize).saturating_sub(1);
                            if target < self.terminals.len() {
                                self.active_terminal_idx = target;
                                return;
                            }
                        }
                    }
                }

                // Next/Previous tab: Alt+Right / Alt+Left or Ctrl+PageDown / Ctrl+PageUp
                if (key.modifiers.contains(KeyModifiers::ALT) && (key.code == KeyCode::Right || key.code == KeyCode::Left))
                    || (key.modifiers.contains(KeyModifiers::CONTROL) && (key.code == KeyCode::PageDown || key.code == KeyCode::PageUp))
                {
                    if !self.terminals.is_empty() {
                        if key.code == KeyCode::Right || key.code == KeyCode::PageDown {
                            self.active_terminal_idx = (self.active_terminal_idx + 1) % self.terminals.len();
                        } else {
                            self.active_terminal_idx = (self.active_terminal_idx + self.terminals.len() - 1) % self.terminals.len();
                        }
                    }
                    return;
                }

                // Scrollback history navigation: Shift+PageUp, Shift+PageDown, Shift+End
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    if let Some(session) = self.terminals.get(self.active_terminal_idx) {
                        match key.code {
                            KeyCode::PageUp => {
                                session.scroll_up(15);
                                return;
                            }
                            KeyCode::PageDown => {
                                session.scroll_down(15);
                                return;
                            }
                            KeyCode::End => {
                                session.scroll_reset();
                                return;
                            }
                            _ => {}
                        }
                    }
                }

                // Forward raw keystrokes to active PTY (including Tab, Ctrl+p, etc. for claude/agy)
                if let Some(session) = self.terminals.get(self.active_terminal_idx) {
                    // Typing input resets scrollback offset to live prompt
                    session.scroll_reset();
                    let input_bytes = key_event_to_bytes(key);
                    if !input_bytes.is_empty() {
                        let _ = session.write_input(&input_bytes);
                    }
                }
            }
            ActivePane::Git => {
                let total = self.total_git_items();
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => {
                        if total > 0 && self.git_selected_idx + 1 < total {
                            self.git_selected_idx += 1;
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        self.git_selected_idx = self.git_selected_idx.saturating_sub(1);
                    }
                    KeyCode::Char(' ') | KeyCode::Char('s') | KeyCode::Char('u') => {
                        self.stage_selected_git_item();
                    }
                    KeyCode::Char('a') | KeyCode::Char('S') => {
                        self.stage_all_git_items();
                    }
                    KeyCode::Char('U') => {
                        self.unstage_all_git_items();
                    }
                    KeyCode::Char('c') => {
                        self.show_commit_modal = true;
                        self.commit_message_input.clear();
                    }
                    KeyCode::Char('p') => {
                        self.push_git();
                    }
                    KeyCode::Char('P') => {
                        self.pull_git();
                    }
                    KeyCode::Char('d') | KeyCode::Enter => {
                        self.diff_selected_git_item();
                    }
                    KeyCode::Char('b') => {
                        self.open_branch_modal();
                    }
                    KeyCode::Char('z') => {
                        self.open_stash_modal();
                    }
                    KeyCode::Char('r') => {
                        self.git_info = GitRepoInfo::fetch(&self.workspace_dir.to_string_lossy());
                        self.status_message = Some(("Refreshed Git status".to_string(), std::time::Instant::now()));
                    }
                    _ => {}
                }
            }
            ActivePane::Agents => match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.active_agent_idx = (self.active_agent_idx + 1).min(self.agents.len().saturating_sub(1));
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if self.active_agent_idx > 0 {
                        self.active_agent_idx -= 1;
                    }
                }
                KeyCode::Char('a') => {
                    self.show_agent_launch_modal = true;
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    let _ = self.stop_active_agent();
                }
                KeyCode::Char('d') => {
                    self.inspect_agent_diff();
                }
                KeyCode::Char('m') => {
                    let _ = self.merge_active_agent();
                }
                KeyCode::Char('x') => {
                    let _ = self.discard_active_agent();
                }
                KeyCode::Enter => {
                    if let Some(agent) = self.agents.get(self.active_agent_idx) {
                        if let Some(t_idx) = agent.terminal_idx {
                            if t_idx < self.terminals.len() {
                                self.active_terminal_idx = t_idx;
                                self.active_pane = ActivePane::Terminal;
                            }
                        }
                    }
                }
                KeyCode::Char(' ') => {
                    if let Some(agent) = self.agents.get_mut(self.active_agent_idx) {
                        if agent.state == crate::agents::AgentState::WaitingForInput {
                            agent.state = crate::agents::AgentState::Running;
                        } else {
                            agent.state = crate::agents::AgentState::WaitingForInput;
                        }
                    }
                }
                _ => {}
            },
        }
    }

    pub fn is_home_or_root_dir(path: &std::path::Path) -> bool {
        let p_canon = match std::fs::canonicalize(path) {
            Ok(c) => c,
            Err(_) => path.to_path_buf(),
        };
        let p_str = p_canon.to_string_lossy();
        let p_clean = p_str.trim_start_matches(r"\\?\").trim_end_matches(['/', '\\']).to_lowercase();

        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            let u_clean = userprofile.trim_start_matches(r"\\?\").trim_end_matches(['/', '\\']).to_lowercase();
            if p_clean == u_clean {
                return true;
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let h_clean = home.trim_start_matches(r"\\?\").trim_end_matches(['/', '\\']).to_lowercase();
            if p_clean == h_clean {
                return true;
            }
        }
        if path.parent().is_none() || p_clean == "c:" || p_clean == "/" || p_clean.is_empty() {
            return true;
        }
        false
    }

    pub fn detect_project_kind(dir: &std::path::Path) -> Option<String> {
        if dir.join("Cargo.toml").is_file() {
            return Some("Rust".to_string());
        }
        if dir.join("package.json").is_file() {
            if dir.join("next.config.js").is_file() || dir.join("next.config.mjs").is_file() || dir.join("next.config.ts").is_file() {
                return Some("Next.js".to_string());
            }
            if dir.join("astro.config.mjs").is_file() || dir.join("astro.config.ts").is_file() {
                return Some("Astro".to_string());
            }
            if dir.join("vite.config.js").is_file() || dir.join("vite.config.ts").is_file() {
                return Some("Vite".to_string());
            }
            return Some("Node".to_string());
        }
        if dir.join("pyproject.toml").is_file() || dir.join("requirements.txt").is_file() || dir.join("manage.py").is_file() {
            return Some("Python".to_string());
        }
        if dir.join("go.mod").is_file() {
            return Some("Go".to_string());
        }
        if dir.join("pubspec.yaml").is_file() {
            return Some("Flutter".to_string());
        }
        if dir.join("composer.json").is_file() {
            return Some("PHP".to_string());
        }
        if dir.join(".git").exists() {
            return Some("Git".to_string());
        }
        None
    }

    pub fn quick_git_branch(dir: &std::path::Path) -> Option<String> {
        let head_file = dir.join(".git").join("HEAD");
        if let Ok(content) = std::fs::read_to_string(head_file) {
            let trimmed = content.trim();
            if let Some(branch) = trimmed.strip_prefix("ref: refs/heads/") {
                return Some(branch.to_string());
            }
            if trimmed.len() >= 7 {
                return Some(trimmed[..7].to_string());
            }
        }
        None
    }

    pub fn scan_workspaces(root: &std::path::Path) -> Vec<WorkspaceEntry> {
        let mut entries = Vec::new();
        let read_dir = match std::fs::read_dir(root) {
            Ok(rd) => rd,
            Err(_) => return entries,
        };

        let skip_dirs = [
            "appdata", "documents", "downloads", "music", "pictures", "videos",
            "favorites", "links", "saved games", "searches", "contacts", "onedrive",
            "virtualbox vms", "cisco packet tracer", "node_modules", "vimfiles",
            "target", "dist", "build", ".git", ".cargo", ".rustup", "temp", "cache"
        ];

        for entry in read_dir.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let file_name = match path.file_name() {
                Some(n) => n.to_string_lossy().to_string(),
                None => continue,
            };

            if file_name.starts_with('.') {
                continue;
            }

            let name_lower = file_name.to_lowercase();
            if skip_dirs.iter().any(|&s| name_lower == s || name_lower.starts_with(s)) {
                continue;
            }

            let kind = Self::detect_project_kind(&path).unwrap_or_else(|| "Folder".to_string());
            let last_modified = entry.metadata().ok().and_then(|m| m.modified().ok());
            let git_branch = if path.join(".git").exists() {
                Self::quick_git_branch(&path)
            } else {
                None
            };

            entries.push(WorkspaceEntry {
                name: file_name,
                path,
                kind,
                last_modified,
                git_branch,
            });
        }

        entries.sort_by(|a, b| {
            let a_is_dev = a.kind != "Folder";
            let b_is_dev = b.kind != "Folder";
            if a_is_dev != b_is_dev {
                return b_is_dev.cmp(&a_is_dev);
            }
            b.last_modified.cmp(&a.last_modified)
        });

        entries
    }

    pub fn get_tide_config_dir() -> PathBuf {
        if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
            let p = PathBuf::from(home).join(".tide");
            let _ = std::fs::create_dir_all(&p);
            p
        } else {
            PathBuf::from(".tide")
        }
    }

    pub fn load_recent_workspaces() -> Vec<PathBuf> {
        let cfg_dir = Self::get_tide_config_dir();
        let file = cfg_dir.join("recent_workspaces.json");
        if let Ok(content) = std::fs::read_to_string(file) {
            let mut list = Vec::new();
            for part in content.split(',') {
                let clean = part.trim()
                    .trim_matches(|c| c == '[' || c == ']' || c == '"' || c == '\n' || c == '\r')
                    .trim();
                if !clean.is_empty() {
                    let p = PathBuf::from(clean);
                    if p.is_dir() && !list.contains(&p) {
                        list.push(p);
                    }
                }
            }
            return list;
        }
        Vec::new()
    }

    pub fn save_recent_workspaces(&self) {
        let cfg_dir = Self::get_tide_config_dir();
        let file = cfg_dir.join("recent_workspaces.json");
        let mut json = String::from("[\n");
        let count = self.recent_workspaces.len().min(20);
        for (i, p) in self.recent_workspaces.iter().take(20).enumerate() {
            let escaped = p.to_string_lossy().replace('\\', "\\\\");
            json.push_str(&format!("  \"{}\"{}", escaped, if i + 1 < count { ",\n" } else { "\n" }));
        }
        json.push(']');
        let _ = std::fs::write(file, json);
    }

    pub fn add_recent_workspace(&mut self, path: &std::path::Path) {
        let pb = path.to_path_buf();
        self.recent_workspaces.retain(|p| p != &pb);
        self.recent_workspaces.insert(0, pb);
        if self.recent_workspaces.len() > 20 {
            self.recent_workspaces.truncate(20);
        }
        self.save_recent_workspaces();
    }

    pub fn switch_workspace(&mut self, path: PathBuf) -> anyhow::Result<()> {
        if !path.exists() || !path.is_dir() {
            self.status_message = Some((
                format!("Invalid directory: {}", path.display()),
                std::time::Instant::now(),
            ));
            return Ok(());
        }

        self.workspace_dir = path.clone();
        self.workspace_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "workspace".to_string());

        self.explorer = FileExplorer::new(&self.workspace_dir);
        self.git_info = GitRepoInfo::fetch(&self.workspace_dir.to_string_lossy());
        self.editor_tabs.clear();
        self.active_editor_tab_idx = 0;
        self.preview_content = None;
        self.update_file_preview();
        self.add_recent_workspace(&path);
        self.show_welcome_screen = false;

        #[cfg(windows)]
        let shell_cmd = "powershell.exe";
        #[cfg(not(windows))]
        let shell_cmd = "bash";
        let title = self.workspace_name.clone();
        let dir_str = self.workspace_dir.to_string_lossy().to_string();
        let _ = self.spawn_terminal_in_dir(
            &title,
            shell_cmd,
            &[],
            Some(&dir_str),
        );

        self.status_message = Some((
            format!("Switched workspace to: {}", self.workspace_name),
            std::time::Instant::now(),
        ));
        Ok(())
    }

    pub fn toggle_welcome_screen(&mut self) {
        self.show_welcome_screen = !self.show_welcome_screen;
        if self.show_welcome_screen {
            self.welcome_filter_query.clear();
            self.welcome_selected_idx = 0;
            self.welcome_prompt_modal = WelcomePromptModal::None;
            let root = if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
                PathBuf::from(home)
            } else {
                self.workspace_dir.clone()
            };
            self.detected_workspaces = Self::scan_workspaces(&root);
            self.recent_workspaces = Self::load_recent_workspaces();
        }
    }

    pub fn filtered_welcome_workspaces(&self) -> Vec<&WorkspaceEntry> {
        let q = self.welcome_filter_query.trim().to_lowercase();
        if q.is_empty() {
            self.detected_workspaces.iter().collect()
        } else {
            self.detected_workspaces
                .iter()
                .filter(|entry| {
                    entry.name.to_lowercase().contains(&q)
                        || entry.kind.to_lowercase().contains(&q)
                        || entry.git_branch.as_ref().map(|b| b.to_lowercase().contains(&q)).unwrap_or(false)
                })
                .collect()
        }
    }

    pub fn handle_welcome_key_event(&mut self, key: KeyEvent) {
        if self.welcome_prompt_modal != WelcomePromptModal::None {
            match &mut self.welcome_prompt_modal {
                WelcomePromptModal::None => unreachable!(),
                WelcomePromptModal::OpenFolder { input } => {
                    match key.code {
                        KeyCode::Esc => {
                            self.welcome_prompt_modal = WelcomePromptModal::None;
                        }
                        KeyCode::Enter => {
                            let raw_path = input.trim().to_string();
                            if !raw_path.is_empty() {
                                let expanded = if raw_path.starts_with('~') {
                                    if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
                                        let rest = raw_path.trim_start_matches('~').trim_start_matches(['/', '\\']);
                                        PathBuf::from(home).join(rest)
                                    } else {
                                        PathBuf::from(&raw_path)
                                    }
                                } else {
                                    PathBuf::from(&raw_path)
                                };

                                if expanded.is_dir() {
                                    self.welcome_prompt_modal = WelcomePromptModal::None;
                                    let _ = self.switch_workspace(expanded);
                                } else {
                                    self.status_message = Some((
                                        format!("Directory not found: {}", raw_path),
                                        std::time::Instant::now(),
                                    ));
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            input.pop();
                        }
                        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) => {
                            input.push(c);
                        }
                        _ => {}
                    }
                }
                WelcomePromptModal::CloneRepo { url_input, dest_input, active_field } => {
                    match key.code {
                        KeyCode::Esc => {
                            self.welcome_prompt_modal = WelcomePromptModal::None;
                        }
                        KeyCode::Tab | KeyCode::BackTab => {
                            *active_field = if *active_field == 0 { 1 } else { 0 };
                        }
                        KeyCode::Enter => {
                            let url = url_input.trim().to_string();
                            let dest = dest_input.trim().to_string();
                            if !url.is_empty() {
                                let target_dir = if dest.is_empty() {
                                    let repo_name = url.rsplit('/').next().unwrap_or("repo").trim_end_matches(".git");
                                    if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
                                        PathBuf::from(home).join(repo_name)
                                    } else {
                                        PathBuf::from(repo_name)
                                    }
                                } else if dest.starts_with('~') {
                                    if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
                                        let rest = dest.trim_start_matches('~').trim_start_matches(['/', '\\']);
                                        PathBuf::from(home).join(rest)
                                    } else {
                                        PathBuf::from(&dest)
                                    }
                                } else {
                                    PathBuf::from(&dest)
                                };

                                self.welcome_prompt_modal = WelcomePromptModal::None;
                                self.status_message = Some((
                                    format!("Cloning {} into {}...", url, target_dir.display()),
                                    std::time::Instant::now(),
                                ));

                                let url_clone = url.clone();
                                let target_clone = target_dir.clone();
                                let bg_tx = self.bg_notify_tx.clone();
                                std::thread::spawn(move || {
                                    let output = std::process::Command::new("git")
                                        .arg("clone")
                                        .arg(&url_clone)
                                        .arg(&target_clone)
                                        .output();

                                    match output {
                                        Ok(out) if out.status.success() => {
                                            let _ = bg_tx.send((format!("Cloned {} successfully!", url_clone), true));
                                        }
                                        Ok(out) => {
                                            let err = String::from_utf8_lossy(&out.stderr);
                                            let _ = bg_tx.send((format!("Git clone failed: {}", err.lines().next().unwrap_or("error")), false));
                                        }
                                        Err(e) => {
                                            let _ = bg_tx.send((format!("Failed to run git: {}", e), false));
                                        }
                                    }
                                });

                                let _ = self.switch_workspace(target_dir);
                            }
                        }
                        KeyCode::Backspace => {
                            if *active_field == 0 {
                                url_input.pop();
                            } else {
                                dest_input.pop();
                            }
                        }
                        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) => {
                            if *active_field == 0 {
                                url_input.push(c);
                            } else {
                                dest_input.push(c);
                            }
                        }
                        _ => {}
                    }
                }
                WelcomePromptModal::NewProject { name_input } => {
                    match key.code {
                        KeyCode::Esc => {
                            self.welcome_prompt_modal = WelcomePromptModal::None;
                        }
                        KeyCode::Enter => {
                            let name = name_input.trim().to_string();
                            if !name.is_empty() {
                                let new_dir = if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
                                    PathBuf::from(home).join(&name)
                                } else {
                                    PathBuf::from(&name)
                                };

                                let _ = std::fs::create_dir_all(&new_dir);
                                self.welcome_prompt_modal = WelcomePromptModal::None;
                                let _ = self.switch_workspace(new_dir);
                            }
                        }
                        KeyCode::Backspace => {
                            name_input.pop();
                        }
                        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) => {
                            name_input.push(c);
                        }
                        _ => {}
                    }
                }
            }
            return;
        }

        match key.code {
            KeyCode::Esc => {
                if !Self::is_home_or_root_dir(&self.workspace_dir) {
                    self.show_welcome_screen = false;
                }
            }
            KeyCode::F(1) => {
                self.show_help = true;
            }
            KeyCode::F(12) => {
                self.show_theme_modal = true;
            }
            KeyCode::Up | KeyCode::BackTab => {
                self.welcome_selected_idx = self.welcome_selected_idx.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Tab => {
                let count = self.filtered_welcome_workspaces().len();
                if count > 0 && self.welcome_selected_idx + 1 < count {
                    self.welcome_selected_idx += 1;
                }
            }
            KeyCode::Enter => {
                let filtered = self.filtered_welcome_workspaces();
                if let Some(entry) = filtered.get(self.welcome_selected_idx) {
                    let path = entry.path.clone();
                    let _ = self.switch_workspace(path);
                }
            }
            KeyCode::Char('o') | KeyCode::Char('O') if key.modifiers.contains(KeyModifiers::CONTROL) || self.welcome_filter_query.is_empty() => {
                self.welcome_prompt_modal = WelcomePromptModal::OpenFolder { input: String::new() };
            }
            KeyCode::Char('c') | KeyCode::Char('C') if key.modifiers.contains(KeyModifiers::CONTROL) || self.welcome_filter_query.is_empty() => {
                self.welcome_prompt_modal = WelcomePromptModal::CloneRepo {
                    url_input: String::new(),
                    dest_input: String::new(),
                    active_field: 0,
                };
            }
            KeyCode::Char('n') | KeyCode::Char('N') if key.modifiers.contains(KeyModifiers::CONTROL) || self.welcome_filter_query.is_empty() => {
                self.welcome_prompt_modal = WelcomePromptModal::NewProject { name_input: String::new() };
            }
            KeyCode::Char(digit @ '1'..='9') if self.welcome_filter_query.is_empty() => {
                let idx = (digit as usize) - ('1' as usize);
                let filtered = self.filtered_welcome_workspaces();
                if let Some(entry) = filtered.get(idx) {
                    let path = entry.path.clone();
                    let _ = self.switch_workspace(path);
                }
            }
            KeyCode::Backspace => {
                self.welcome_filter_query.pop();
                self.welcome_selected_idx = 0;
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT) => {
                self.welcome_filter_query.push(c);
                self.welcome_selected_idx = 0;
            }
            _ => {}
        }
    }

    pub fn handle_welcome_mouse_event(&mut self, mouse: crossterm::event::MouseEvent) {
        let (x, y) = (mouse.column, mouse.row);

        match mouse.kind {
            crossterm::event::MouseEventKind::ScrollUp => {
                self.welcome_selected_idx = self.welcome_selected_idx.saturating_sub(1);
                return;
            }
            crossterm::event::MouseEventKind::ScrollDown => {
                let count = self.filtered_welcome_workspaces().len();
                if self.welcome_selected_idx + 1 < count {
                    self.welcome_selected_idx += 1;
                }
                return;
            }
            crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) => {}
            _ => return,
        }

        if self.welcome_prompt_modal != WelcomePromptModal::None {
            return;
        }

        let area = self.welcome_area;
        if area.width < 50 || area.height < 14 {
            return;
        }

        let is_wide = area.width >= 95;
        let content_area = if is_wide {
            let container_w = (area.width.saturating_sub(4)).min(130);
            let container_h = (area.height.saturating_sub(2)).min(46);
            let pad_x = (area.width.saturating_sub(container_w)) / 2;
            let pad_y = (area.height.saturating_sub(container_h)) / 2;
            ratatui::layout::Rect::new(area.x + pad_x, area.y + pad_y, container_w, container_h)
        } else {
            area
        };

        if is_wide {
            let cols = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Length(46),
                    ratatui::layout::Constraint::Min(40),
                ])
                .split(content_area);

            if cols[0].contains(ratatui::layout::Position { x, y }) {
                let hero_inner = ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .inner(cols[0]);
                let logo_lines = crate::logo::wave_art_for_height(hero_inner.height.saturating_sub(10));
                let logo_height = logo_lines.len() as u16;
                let chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints([
                        ratatui::layout::Constraint::Length(logo_height),
                        ratatui::layout::Constraint::Length(3),
                        ratatui::layout::Constraint::Min(0),
                    ])
                    .split(hero_inner);

                let actions_start_y = chunks[2].y + 2;
                if y == actions_start_y {
                    self.welcome_prompt_modal = WelcomePromptModal::OpenFolder { input: String::new() };
                } else if y == actions_start_y + 1 {
                    self.welcome_prompt_modal = WelcomePromptModal::CloneRepo {
                        url_input: String::new(),
                        dest_input: String::new(),
                        active_field: 0,
                    };
                } else if y == actions_start_y + 2 {
                    self.welcome_prompt_modal = WelcomePromptModal::NewProject { name_input: String::new() };
                } else if y == actions_start_y + 3 && !Self::is_home_or_root_dir(&self.workspace_dir) {
                    self.show_welcome_screen = false;
                }
                return;
            }

            if cols[1].contains(ratatui::layout::Position { x, y }) {
                let right_inner = ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .inner(cols[1]);
                let chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints([
                        ratatui::layout::Constraint::Length(3),
                        ratatui::layout::Constraint::Min(0),
                        ratatui::layout::Constraint::Length(2),
                    ])
                    .split(right_inner);

                let list_area = chunks[1];
                if list_area.contains(ratatui::layout::Position { x, y }) {
                    let clicked_row = (y.saturating_sub(list_area.y)) as usize;
                    let filtered = self.filtered_welcome_workspaces();
                    let max_visible = list_area.height as usize;
                    let scroll_offset = if self.welcome_selected_idx >= max_visible {
                        self.welcome_selected_idx.saturating_sub(max_visible - 1)
                    } else {
                        0
                    };
                    let actual_idx = scroll_offset + clicked_row;
                    if actual_idx < filtered.len() {
                        let path = filtered[actual_idx].path.clone();
                        let _ = self.switch_workspace(path);
                    }
                }
            }
        }
    }
}

fn key_event_to_bytes(key: KeyEvent) -> Vec<u8> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('c') => return vec![3],
            KeyCode::Char('d') => return vec![4],
            KeyCode::Char(c) => {
                let ascii = c as u8;
                if ascii.is_ascii_lowercase() {
                    return vec![ascii - b'a' + 1];
                } else if ascii.is_ascii_uppercase() {
                    return vec![ascii - b'A' + 1];
                }
            }
            _ => {}
        }
    }

    match key.code {
        KeyCode::Char(c) => {
            let mut buf = [0u8; 4];
            c.encode_utf8(&mut buf).as_bytes().to_vec()
        }
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Backspace => vec![8],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::Esc => vec![27],
        KeyCode::Up => vec![27, b'[', b'A'],
        KeyCode::Down => vec![27, b'[', b'B'],
        KeyCode::Right => vec![27, b'[', b'C'],
        KeyCode::Left => vec![27, b'[', b'D'],
        KeyCode::Home => vec![27, b'[', b'H'],
        KeyCode::End => vec![27, b'[', b'F'],
        KeyCode::Delete => vec![27, b'[', b'3', b'~'],
        KeyCode::PageUp => vec![27, b'[', b'5', b'~'],
        KeyCode::PageDown => vec![27, b'[', b'6', b'~'],
        _ => Vec::new(),
    }
}

pub fn format_terminal_tab_title(title: &str) -> String {
    if let Some(open_paren) = title.find('(') {
        let prefix = title[..open_paren].trim();
        let session = title[open_paren + 1..].trim_end_matches(')').trim();
        if let Some(dash) = session.rfind('-') {
            let num = &session[dash + 1..];
            if num.chars().all(|c| c.is_ascii_digit()) {
                return format!("{} #{}", prefix, num);
            }
        }
        return format!("{}: {}", prefix, session);
    }
    title.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_terminal_tab_title() {
        assert_eq!(format_terminal_tab_title("PowerShell (session-1)"), "PowerShell #1");
        assert_eq!(format_terminal_tab_title("Claude Code (agent-worktree-2)"), "Claude Code #2");
        assert_eq!(format_terminal_tab_title("Plain Title"), "Plain Title");
    }

    #[test]
    fn test_editor_tabs_switching_and_closing() {
        let mut app = App::new(PathBuf::from(".")).expect("Failed to initialize App");
        
        let path1 = PathBuf::from("Cargo.toml");
        let path2 = PathBuf::from("src/main.rs");
        let path3 = PathBuf::from("src/app.rs");

        // Open path1 as pinned
        app.open_file(&path1, true);
        assert_eq!(app.editor_tabs.len(), 1);
        assert_eq!(app.editor_tabs[0].file_name, "Cargo.toml");
        assert!(app.editor_tabs[0].is_pinned);

        // Open path2 as pinned
        app.open_file(&path2, true);
        assert_eq!(app.editor_tabs.len(), 2);
        assert_eq!(app.active_editor_tab_idx, 1);
        assert_eq!(app.editor_tabs[1].file_name, "main.rs");

        // Cycle next and prev
        app.next_editor_tab();
        assert_eq!(app.active_editor_tab_idx, 0);
        app.prev_editor_tab();
        assert_eq!(app.active_editor_tab_idx, 1);

        // Open path3
        app.open_file(&path3, true);
        assert_eq!(app.editor_tabs.len(), 3);
        assert_eq!(app.active_editor_tab_idx, 2);

        // Close tab 2 (app.rs)
        app.close_editor_tab(2);
        assert_eq!(app.editor_tabs.len(), 2);
        assert_eq!(app.active_editor_tab_idx, 1);
        assert_eq!(app.editor_tabs[app.active_editor_tab_idx].file_name, "main.rs");

        // Close all tabs
        app.close_editor_tab(0);
        app.close_editor_tab(0);
        assert_eq!(app.editor_tabs.len(), 0);
        assert!(app.preview_content.is_none());
    }

    #[test]
    fn test_find_in_file_matches_and_navigation() {
        let mut app = App::new(PathBuf::from(".")).expect("Failed to initialize App");

        // Set test preview content with known matches
        let test_lines = vec![
            "fn calculate_total() -> i32 {".to_string(),
            "    let total = 100;".to_string(),
            "    let sub_total = 50;".to_string(),
            "    total + sub_total".to_string(),
            "}".to_string(),
        ];
        app.preview_content = Some(("calc.rs".to_string(), test_lines));
        app.editor_cursor = (0, 0);

        // Search for "total" (case-insensitive)
        app.find_query = "TOTAL".to_string();
        app.update_find_matches();

        // Line 0: "total" in "calculate_total"
        // Line 1: "total" in "let total"
        // Line 2: "total" in "sub_total"
        // Line 3: "total" in "total + sub_total" (2 matches!)
        // Total matches = 1 + 1 + 1 + 2 = 5
        assert_eq!(app.find_matches.len(), 5);
        assert_eq!(app.active_find_match_idx, 0);

        // Test navigation next
        app.next_find_match();
        assert_eq!(app.active_find_match_idx, 1);
        app.next_find_match();
        assert_eq!(app.active_find_match_idx, 2);

        // Test wraparound next
        app.active_find_match_idx = 4;
        app.next_find_match();
        assert_eq!(app.active_find_match_idx, 0);

        // Test navigation prev
        app.prev_find_match();
        assert_eq!(app.active_find_match_idx, 4);
        app.prev_find_match();
        assert_eq!(app.active_find_match_idx, 3);
    }

    #[test]
    fn test_global_search_workspace_filtering() {
        let mut app = App::new(PathBuf::from(".")).expect("Failed to initialize App");

        // Search for "TIDE" across the workspace
        app.global_search_query = "TIDE".to_string();
        app.run_global_search();

        assert!(!app.global_search_results.is_empty(), "Expected results for search query 'TIDE'");

        // Verify that ignored directories (.git, target, node_modules) are never included
        for res in &app.global_search_results {
            let path_str = res.file_path.to_string_lossy().to_lowercase();
            assert!(!path_str.contains("\\target\\"), "Target directory should be excluded from search");
            assert!(!path_str.contains("/target/"), "Target directory should be excluded from search");
            assert!(!path_str.contains("\\.git\\"), "Git directory should be excluded from search");
            assert!(!path_str.contains("/.git/"), "Git directory should be excluded from search");
            assert!(res.line_num > 0, "Line numbers must be 1-indexed");
            assert!(!res.rel_path.is_empty(), "Relative path must be non-empty");
        }
    }

    #[test]
    fn test_explorer_new_file_and_folder() {
        let temp_dir = std::env::temp_dir().join("tide_test_explorer_new");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let mut app = App::new(temp_dir.clone()).expect("Failed to initialize App");

        // Create new file with subdirectories
        app.explorer_op_modal = ExplorerOpModal::NewItem {
            target_dir: temp_dir.clone(),
            input: "nested/test_module.rs".to_string(),
            is_folder: false,
        };
        app.execute_new_item();

        let created_file = temp_dir.join("nested").join("test_module.rs");
        assert!(created_file.is_file(), "File should be created on disk");
        assert_eq!(app.editor_tabs.len(), 1);
        assert_eq!(app.editor_tabs[0].file_name, "test_module.rs");
        assert_eq!(app.active_pane, ActivePane::FilePreview);

        // Create new folder
        app.explorer_op_modal = ExplorerOpModal::NewItem {
            target_dir: temp_dir.clone(),
            input: "components".to_string(),
            is_folder: true,
        };
        app.execute_new_item();

        let created_dir = temp_dir.join("components");
        assert!(created_dir.is_dir(), "Folder should be created on disk");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_explorer_rename_and_delete() {
        let temp_dir = std::env::temp_dir().join("tide_test_explorer_rename_del");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let initial_file = temp_dir.join("initial.rs");
        std::fs::write(&initial_file, b"// test content").expect("Write initial file");

        let mut app = App::new(temp_dir.clone()).expect("Failed to initialize App");
        app.open_file(&initial_file, true);
        assert_eq!(app.editor_tabs.len(), 1);
        assert_eq!(app.editor_tabs[0].file_name, "initial.rs");

        // Rename file
        app.explorer_op_modal = ExplorerOpModal::Rename {
            target_path: initial_file.clone(),
            input: "renamed.rs".to_string(),
            is_dir: false,
        };
        app.execute_rename_item();

        let renamed_file = temp_dir.join("renamed.rs");
        assert!(!initial_file.exists(), "Old file should not exist after rename");
        assert!(renamed_file.is_file(), "New file should exist after rename");
        assert_eq!(app.editor_tabs[0].file_name, "renamed.rs");
        assert_eq!(app.editor_tabs[0].file_path, renamed_file);

        // Delete file
        app.explorer_op_modal = ExplorerOpModal::DeleteConfirm {
            target_path: renamed_file.clone(),
            is_dir: false,
            name: "renamed.rs".to_string(),
        };
        app.execute_delete_item();

        assert!(!renamed_file.exists(), "File should be deleted from disk");
        assert_eq!(app.editor_tabs.len(), 0, "Deleted file should be removed from editor tabs");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_git_pane_actions_and_commit_modal() {
        let temp_dir = std::env::temp_dir().join("tide_test_git_pane");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let mut app = App::new(temp_dir.clone()).expect("Failed to initialize App");

        assert_eq!(app.git_selected_idx, 0);
        assert_eq!(app.show_commit_modal, false);
        assert!(app.commit_message_input.is_empty());

        // Test empty commit validation
        app.show_commit_modal = true;
        app.commit_message_input = "   ".to_string();
        app.execute_commit();
        assert!(app.show_commit_modal, "Modal should remain open on empty commit message");
        assert!(app.status_message.as_ref().unwrap().0.contains("empty"));

        // Test commit with no staged files validation
        app.commit_message_input = "feat: initial commit".to_string();
        app.execute_commit();
        assert!(app.show_commit_modal, "Modal should remain open when no files are staged");
        assert!(app.status_message.as_ref().unwrap().0.contains("no staged changes"));

        assert_eq!(app.total_git_items(), app.git_info.staged_files.len() + app.git_info.unstaged_files.len());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_theme_palette_and_switching() {
        let temp_dir = std::env::temp_dir().join("tide_test_theme");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let mut app = App::new(temp_dir.clone()).expect("Failed to initialize App");

        assert_eq!(app.color_theme, crate::theme::ColorTheme::TideOcean);
        let default_palette = app.theme();
        assert_eq!(default_palette, crate::theme::ColorTheme::TideOcean.palette());

        // Test cycling themes with next()
        app.color_theme = app.color_theme.next();
        assert_eq!(app.color_theme, crate::theme::ColorTheme::TokyoNight);

        app.color_theme = app.color_theme.next();
        assert_eq!(app.color_theme, crate::theme::ColorTheme::CatppuccinMocha);

        // Test cycling backward with prev()
        app.color_theme = app.color_theme.prev();
        assert_eq!(app.color_theme, crate::theme::ColorTheme::TokyoNight);

        // Test all 10 themes have valid swatches and names
        assert_eq!(crate::theme::ColorTheme::ALL.len(), 10);
        for theme in crate::theme::ColorTheme::ALL {
            assert!(!theme.name().is_empty());
            assert!(!theme.description().is_empty());
            let (bg, a1, a2, success) = theme.swatches();
            let pal = theme.palette();
            assert_eq!(bg, pal.surface);
            assert_eq!(a1, pal.accent);
            assert_eq!(a2, pal.accent_secondary);
            assert_eq!(success, pal.success);
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_terminal_zoom_and_mouse_scroll() {
        let temp_dir = std::env::temp_dir().join("tide_test_terminal_enhancements");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let mut app = App::new(temp_dir.clone()).expect("Failed to initialize App");

        // Test zoom operations
        assert_eq!(app.terminal_zoom_level, 0);
        app.zoom_in_terminal();
        assert_eq!(app.terminal_zoom_level, 1);
        app.zoom_in_terminal();
        assert_eq!(app.terminal_zoom_level, 2);
        app.zoom_out_terminal();
        assert_eq!(app.terminal_zoom_level, 1);
        app.zoom_reset_terminal();
        assert_eq!(app.terminal_zoom_level, 0);

        // Test session scroll operations
        if let Some(session) = app.terminals.get(0) {
            assert_eq!(session.scrollback(), 0);
            session.scroll_up(10);
            session.scroll_down(5);
            session.scroll_reset();
            assert_eq!(session.scrollback(), 0);
        }

        // Test Ctrl+= and Ctrl+- hotkeys
        app.handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('+'),
            crossterm::event::KeyModifiers::CONTROL,
        ));
        assert_eq!(app.terminal_zoom_level, 1);

        app.handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('-'),
            crossterm::event::KeyModifiers::CONTROL,
        ));
        assert_eq!(app.terminal_zoom_level, 0);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_git_branch_modal_and_filtering() {
        let temp_dir = std::env::temp_dir().join("tide_test_git_branch");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let mut app = App::new(temp_dir.clone()).expect("Failed to initialize App");

        assert!(!app.show_branch_modal);
        assert!(!app.branch_is_creating);

        // Populate dummy branches
        app.branch_items = vec![
            crate::git::GitBranchItem {
                name: "main".to_string(),
                is_current: true,
                is_worktree: false,
                is_remote: false,
                commit_hash: "8efeb68".to_string(),
                commit_msg: "initial commit".to_string(),
            },
            crate::git::GitBranchItem {
                name: "feat/search".to_string(),
                is_current: false,
                is_worktree: false,
                is_remote: false,
                commit_hash: "1a2b3c4".to_string(),
                commit_msg: "add global search".to_string(),
            },
            crate::git::GitBranchItem {
                name: "remotes/origin/feat/agent".to_string(),
                is_current: false,
                is_worktree: false,
                is_remote: true,
                commit_hash: "9f8e7d6".to_string(),
                commit_msg: "agent integration".to_string(),
            },
        ];

        app.show_branch_modal = true;
        assert_eq!(app.get_filtered_branches().len(), 3);

        // Filter by query "search"
        app.branch_filter_query = "search".to_string();
        let filtered = app.get_filtered_branches();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "feat/search");

        // Filter by commit hash "9f8e"
        app.branch_filter_query = "9f8e".to_string();
        let filtered_hash = app.get_filtered_branches();
        assert_eq!(filtered_hash.len(), 1);
        assert_eq!(filtered_hash[0].display_name(), "feat/agent");

        // Test branch creation state transition
        app.branch_is_creating = true;
        app.branch_new_name = "feat/new-modal".to_string();
        assert_eq!(app.branch_new_name, "feat/new-modal");

        // Test [Esc] exits creation mode back to browse mode
        app.handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::empty(),
        ));
        assert!(!app.branch_is_creating);
        assert!(app.show_branch_modal);

        // Second [Esc] closes the branch modal
        app.handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::empty(),
        ));
        assert!(!app.show_branch_modal);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_git_stash_modal_and_toggle() {
        let temp_dir = std::env::temp_dir().join("tide_test_git_stash");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let mut app = App::new(temp_dir.clone()).expect("Failed to initialize App");

        assert!(!app.show_stash_modal);
        assert!(app.stash_include_untracked);

        // Populate dummy stashes
        app.stash_items = vec![
            crate::git::GitStashItem {
                index: 0,
                selector: "stash@{0}".to_string(),
                relative_time: "5 minutes ago".to_string(),
                message: "WIP on main: work in progress".to_string(),
            },
            crate::git::GitStashItem {
                index: 1,
                selector: "stash@{1}".to_string(),
                relative_time: "1 hour ago".to_string(),
                message: "WIP on feat: saved experiments".to_string(),
            },
        ];

        app.show_stash_modal = true;
        assert_eq!(app.stash_items.len(), 2);

        // Test navigation
        assert_eq!(app.stash_selected_idx, 0);
        app.handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Down,
            crossterm::event::KeyModifiers::empty(),
        ));
        assert_eq!(app.stash_selected_idx, 1);

        // Test [s] switches to stash create mode
        app.handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('s'),
            crossterm::event::KeyModifiers::empty(),
        ));
        assert!(app.stash_is_creating);

        // Test typing stash message
        app.handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('m'),
            crossterm::event::KeyModifiers::empty(),
        ));
        assert_eq!(app.stash_new_message, "m");

        // Test Ctrl+u toggles include untracked
        assert!(app.stash_include_untracked);
        app.handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('u'),
            crossterm::event::KeyModifiers::CONTROL,
        ));
        assert!(!app.stash_include_untracked);

        // Test [Esc] exits create mode
        app.handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::empty(),
        ));
        assert!(!app.stash_is_creating);
        assert!(app.show_stash_modal);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_editor_undo_redo_stack() {
        let temp_dir = std::env::temp_dir().join("tide_test_undo_redo");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
        let test_file = temp_dir.join("main.rs");
        std::fs::write(&test_file, "fn main() {\n    println!(\"Hello\");\n}").expect("write test file");

        let mut app = App::new(temp_dir.clone()).expect("Failed to initialize App");
        app.open_file(&test_file, true);
        app.active_pane = ActivePane::FilePreview;

        assert_eq!(app.editor_tabs.len(), 1);
        assert!(!app.editor_dirty);

        // Make an edit (push undo snapshot)
        app.push_editor_undo_snapshot();
        if let Some((_, lines)) = &mut app.preview_content {
            lines[0] = "pub fn main() {".to_string();
        }
        app.mark_editor_dirty();
        assert!(app.editor_dirty);

        // Perform undo
        app.editor_undo();
        if let Some((_, lines)) = &app.preview_content {
            assert_eq!(lines[0], "fn main() {");
        }

        // Perform redo
        app.editor_redo();
        if let Some((_, lines)) = &app.preview_content {
            assert_eq!(lines[0], "pub fn main() {");
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_editor_line_actions() {
        let temp_dir = std::env::temp_dir().join("tide_test_line_actions");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
        let test_file = temp_dir.join("script.py");
        std::fs::write(&test_file, "line 1\nline 2\nline 3").expect("write test file");

        let mut app = App::new(temp_dir.clone()).expect("Failed to initialize App");
        app.open_file(&test_file, true);
        app.active_pane = ActivePane::FilePreview;

        // Test duplicate line
        app.editor_cursor = (0, 0);
        app.duplicate_current_line();
        if let Some((_, lines)) = &app.preview_content {
            assert_eq!(lines.len(), 4);
            assert_eq!(lines[0], "line 1");
            assert_eq!(lines[1], "line 1");
        }
        assert_eq!(app.editor_cursor.0, 1);

        // Test move line down
        app.editor_cursor = (0, 0);
        app.move_current_line_down();
        if let Some((_, lines)) = &app.preview_content {
            assert_eq!(lines[0], "line 1");
            assert_eq!(lines[1], "line 1");
        }

        // Test move line up
        app.editor_cursor = (2, 0);
        app.move_current_line_up();
        assert_eq!(app.editor_cursor.0, 1);

        // Test delete line
        app.editor_cursor = (1, 0);
        app.delete_current_line();
        if let Some((_, lines)) = &app.preview_content {
            assert_eq!(lines.len(), 3);
        }

        // Test toggle comment on Python file
        app.editor_cursor = (0, 0);
        app.toggle_comment_on_current_line();
        if let Some((_, lines)) = &app.preview_content {
            assert_eq!(lines[0], "# line 1");
        }
        // Toggle again to remove comment
        app.toggle_comment_on_current_line();
        if let Some((_, lines)) = &app.preview_content {
            assert_eq!(lines[0], "line 1");
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

