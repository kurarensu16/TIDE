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
    pub dirty_tx: Sender<()>,
    pub dirty_rx: Receiver<()>,
    pub show_help: bool,
    pub show_command_palette: bool,
    pub show_agent_launch_modal: bool,
    pub show_diff_modal: bool,
    pub diff_modal_content: Option<(String, String)>,
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

        // Pre-populate demonstrative agent sessions matching TIDE.png
        let mut codex = AgentSession::new("codex-1", "Codex", "src/agents/codex.rs", "tide/codex-worktree");
        codex.start_time = std::time::Instant::now() - std::time::Duration::from_secs(12 * 60);

        let mut claude = AgentSession::new("claude-1", "Claude Code", "src/app/state.rs", "tide/claude-worktree");
        claude.start_time = std::time::Instant::now() - std::time::Duration::from_secs(8 * 60);

        let mut antigravity = AgentSession::new("antigravity-1", "Antigravity", "docs/plan.md", "tide/antigravity-worktree");
        antigravity.start_time = std::time::Instant::now() - std::time::Duration::from_secs(3 * 60);
        antigravity.state = crate::agents::AgentState::WaitingForInput;

        let agents = vec![codex, claude, antigravity];

        let mut app = Self {
            workspace_dir: workspace_dir.clone(),
            workspace_name,
            active_pane: ActivePane::Explorer,
            explorer,
            git_info,
            terminals: Vec::new(),
            active_terminal_idx: 0,
            agents,
            active_agent_idx: 0,
            preview_content: None,
            preview_scroll: 0,
            dirty_tx: dirty_tx.clone(),
            dirty_rx,
            show_help: false,
            show_command_palette: false,
            show_agent_launch_modal: false,
            show_diff_modal: false,
            diff_modal_content: None,
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
                    self.preview_scroll = 0;
                    return;
                }
            }
        }
        self.preview_content = None;
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
        let wt = WorktreeInfo::create(&self.workspace_dir, &session_id, base_branch)?;

        // 2. Spawn Terminal in the isolated worktree directory
        let title = format!("{} (Agent)", agent_name);
        self.spawn_terminal_in_dir(&title, cmd, args, Some(&wt.path.to_string_lossy()))?;
        let term_idx = self.terminals.len() - 1;

        // 3. Register AgentSession
        let mut session = AgentSession::new(&session_id, agent_name, task, &wt.branch);
        session.worktree = Some(wt);
        session.terminal_idx = Some(term_idx);
        self.agents.push(session);
        self.active_agent_idx = self.agents.len() - 1;

        // Switch to terminal pane to show live output
        self.active_pane = ActivePane::Terminal;
        Ok(())
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
                    if self.active_terminal_idx >= self.terminals.len() && !self.terminals.is_empty() {
                        self.active_terminal_idx = self.terminals.len() - 1;
                    }
                }
            }
            if self.active_agent_idx >= self.agents.len() && !self.agents.is_empty() {
                self.active_agent_idx = self.agents.len() - 1;
            }
        }
        self.show_diff_modal = false;
        Ok(())
    }

    pub fn cycle_active_pane(&mut self) {
        self.active_pane = match self.active_pane {
            ActivePane::Explorer => ActivePane::FilePreview,
            ActivePane::FilePreview => ActivePane::Terminal,
            ActivePane::Terminal => ActivePane::Git,
            ActivePane::Git => ActivePane::Agents,
            ActivePane::Agents => ActivePane::Explorer,
        };
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

        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('p') {
            self.show_command_palette = true;
            return;
        }

        if key.code == KeyCode::F(1) {
            self.show_help = true;
            return;
        }

        // Pane switching
        if key.code == KeyCode::Tab {
            self.cycle_active_pane();
            return;
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
                KeyCode::Enter => {
                    self.active_pane = ActivePane::FilePreview;
                }
                _ => {}
            },
            ActivePane::FilePreview => match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    if let Some((_, lines)) = &self.preview_content {
                        if self.preview_scroll + 1 < lines.len() {
                            self.preview_scroll += 1;
                        }
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if self.preview_scroll > 0 {
                        self.preview_scroll -= 1;
                    }
                }
                KeyCode::Char('i') | KeyCode::Enter => {
                    self.active_pane = ActivePane::Terminal;
                }
                _ => {}
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
                        self.terminals.remove(self.active_terminal_idx);
                        if self.active_terminal_idx >= self.terminals.len() {
                            self.active_terminal_idx = self.terminals.len() - 1;
                        }
                    }
                    return;
                }

                // Forward raw keystrokes to active PTY
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
