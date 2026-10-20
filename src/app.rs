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
    pub preview_area: ratatui::layout::Rect,
    pub terminal_area: ratatui::layout::Rect,
    pub terminal_tabs_area: ratatui::layout::Rect,
    pub git_area: ratatui::layout::Rect,
    pub agents_area: ratatui::layout::Rect,
    pub header_area: ratatui::layout::Rect,
    pub dirty_tx: Sender<()>,
    pub dirty_rx: Receiver<()>,
    pub show_help: bool,
    pub show_command_palette: bool,
    pub show_agent_launch_modal: bool,
    pub show_diff_modal: bool,
    pub diff_modal_content: Option<(String, String)>,
    pub status_message: Option<(String, std::time::Instant)>,
    pub should_quit: bool,
}

impl App {
    pub fn new(workspace_dir: PathBuf) -> anyhow::Result<Self> {
        let (dirty_tx, dirty_rx) = unbounded();

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
            preview_area: ratatui::layout::Rect::default(),
            terminal_area: ratatui::layout::Rect::default(),
            terminal_tabs_area: ratatui::layout::Rect::default(),
            git_area: ratatui::layout::Rect::default(),
            agents_area: ratatui::layout::Rect::default(),
            header_area: ratatui::layout::Rect::default(),
            dirty_tx: dirty_tx.clone(),
            dirty_rx,
            show_help: false,
            show_command_palette: false,
            show_agent_launch_modal: false,
            show_diff_modal: false,
            diff_modal_content: None,
            status_message: None,
            should_quit: false,
        };

        app.update_file_preview();

        // Spawn initial shell session (PowerShell on Windows, fallback to cmd)
        #[cfg(windows)]
        let shell_cmd = "powershell.exe";
        #[cfg(not(windows))]
        let shell_cmd = "bash";

        app.spawn_terminal("PowerShell", shell_cmd, &[])?;

        Ok(app)
    }

    pub fn update_file_preview(&mut self) {
        if let Some(entry) = self.explorer.selected_entry() {
            if !entry.is_dir {
                if let Ok(content) = std::fs::read_to_string(&entry.path) {
                    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                    self.preview_content = Some((entry.name.clone(), lines));
                    self.editor_file_path = Some(entry.path.clone());
                    self.preview_scroll = 0;
                    self.editor_cursor = (0, 0);
                    self.editor_dirty = false;
                    return;
                }
            }
        }
        self.preview_content = None;
        self.editor_file_path = None;
        self.editor_dirty = false;
    }

    pub fn save_editor_content(&mut self) {
        if let (Some(path), Some((_, lines))) = (&self.editor_file_path, &self.preview_content) {
            let content = lines.join("\r\n");
            match std::fs::write(path, content) {
                Ok(_) => {
                    self.editor_dirty = false;
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

    pub fn handle_mouse_event(&mut self, mouse: crossterm::event::MouseEvent) {
        let (x, y) = (mouse.column, mouse.row);

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

                // Check Terminal tab bar clicks
                if self.terminal_tabs_area.contains(ratatui::layout::Position { x, y }) {
                    self.active_pane = ActivePane::Terminal;
                    let rel_x = x.saturating_sub(self.terminal_tabs_area.x);
                    let tab_width = 24; // approximate average tab width
                    let clicked_tab = (rel_x as usize) / tab_width;
                    if clicked_tab < self.terminals.len() {
                        self.active_terminal_idx = clicked_tab;
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
                    if y >= inner_y {
                        let clicked_idx = (y - inner_y) as usize;
                        if clicked_idx < self.explorer.entries.len() {
                            self.explorer.selected = clicked_idx;
                            self.update_file_preview();
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
                    let inner_y = self.preview_area.y + 1;
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
                        let clicked_agent = (y - inner_y) as usize / 4;
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
                if self.preview_area.contains(ratatui::layout::Position { x, y }) {
                    if let Some((_, lines)) = &self.preview_content {
                        if self.preview_scroll + 2 < lines.len() {
                            self.preview_scroll += 2;
                        }
                    }
                } else if self.explorer_area.contains(ratatui::layout::Position { x, y }) {
                    self.explorer.next();
                    self.update_file_preview();
                }
            }
            crossterm::event::MouseEventKind::ScrollUp => {
                if self.preview_area.contains(ratatui::layout::Position { x, y }) {
                    self.preview_scroll = self.preview_scroll.saturating_sub(2);
                } else if self.explorer_area.contains(ratatui::layout::Position { x, y }) {
                    self.explorer.prev();
                    self.update_file_preview();
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

        if self.show_agent_launch_modal {
            match key.code {
                KeyCode::Esc => self.show_agent_launch_modal = false,
                KeyCode::Char('1') => {
                    let _ = self.launch_agent("Antigravity", "agy", &[], "Interactive Coding Agent");
                    self.show_agent_launch_modal = false;
                }
                KeyCode::Char('2') => {
                    let _ = self.launch_agent("Claude Code", "claude", &[], "Autonomous Agent Session");
                    self.show_agent_launch_modal = false;
                }
                KeyCode::Char('3') => {
                    let _ = self.launch_agent("Worktree Shell", "powershell.exe", &[], "Isolated Worktree Shell");
                    self.show_agent_launch_modal = false;
                }
                _ => {}
            }
            return;
        }

        if self.show_diff_modal {
            match key.code {
                KeyCode::Esc => self.show_diff_modal = false,
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    let _ = self.merge_active_agent();
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    let _ = self.discard_active_agent();
                }
                _ => {}
            }
            return;
        }

        if self.show_command_palette {
            match key.code {
                KeyCode::Esc => self.show_command_palette = false,
                KeyCode::Char('1') => {
                    let _ = self.spawn_terminal("PowerShell", "powershell.exe", &[]);
                    self.show_command_palette = false;
                }
                KeyCode::Char('2') => {
                    self.show_command_palette = false;
                    self.show_agent_launch_modal = true;
                }
                KeyCode::Char('3') => {
                    self.git_info = GitRepoInfo::fetch(&self.workspace_dir.to_string_lossy());
                    self.show_command_palette = false;
                }
                KeyCode::Char('4') => {
                    self.explorer.refresh();
                    self.update_file_preview();
                    self.show_command_palette = false;
                }
                KeyCode::Char('5') => {
                    self.should_quit = true;
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
            KeyCode::F(11) => {
                self.is_maximized = !self.is_maximized;
                return;
            }
            _ => {}
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
                KeyCode::Char('r') => {
                    self.explorer.refresh();
                    self.update_file_preview();
                }
                KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(entry) = self.explorer.selected_entry() {
                        let path = entry.path.clone();
                        self.open_in_external_editor(&path);
                    }
                }
                KeyCode::Enter => {
                    self.active_pane = ActivePane::FilePreview;
                }
                _ => {}
            },
            ActivePane::FilePreview => {
                // Save with Ctrl+s
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
                    self.save_editor_content();
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
                        self.editor_cursor.1 = 0;
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
            },
            ActivePane::Terminal => {
                // Escape chord: Ctrl+b returns focus to TIDE Explorer pane
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('b') {
                    self.active_pane = ActivePane::Explorer;
                    return;
                }

                // New tab: Ctrl+t
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('t') {
                    let _ = self.spawn_terminal("PowerShell", "powershell.exe", &[]);
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

                // Forward raw keystrokes to active PTY (including Tab, Ctrl+p, etc. for claude/agy)
                if let Some(session) = self.terminals.get(self.active_terminal_idx) {
                    let input_bytes = key_event_to_bytes(key);
                    if !input_bytes.is_empty() {
                        let _ = session.write_input(&input_bytes);
                    }
                }
            }
            ActivePane::Git => match key.code {
                KeyCode::Char('r') => {
                    self.git_info = GitRepoInfo::fetch(&self.workspace_dir.to_string_lossy());
                }
                _ => {}
            },
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
