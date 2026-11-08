//! Command Palette actions, fuzzy search filtering, and execution dispatch.

use std::path::PathBuf;

use crate::app::{ActivePane, App};
use crate::git::GitRepoInfo;

#[derive(Clone, Debug)]
pub enum PaletteAction {
    OpenFile(PathBuf),
    SpawnTerminal(String, String, Vec<String>),
    LaunchAgentModal,
    StopAgent,
    VerifyAgentWorktree,
    JumpToDiagnostic,
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
    EditorCopy,
    EditorCut,
    EditorPaste,
    EditorSelectAll,
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

#[derive(Clone, Debug)]
pub struct PaletteItem {
    pub icon: String,
    pub title: String,
    pub subtitle: String,
    pub action: PaletteAction,
}

impl App {
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
                    icon: p_icon("search"),
                    title: "Terminal: Jump to Compiler Diagnostic (Ctrl+G)".to_string(),
                    subtitle: "Scan terminal output for file:line:col and open in Editor".to_string(),
                    action: PaletteAction::JumpToDiagnostic,
                },
                PaletteItem {
                    icon: p_icon("agent"),
                    title: "Agents: Launch New AI Agent".to_string(),
                    subtitle: "Start Antigravity, Claude Code, or isolated shell (Ctrl+A)".to_string(),
                    action: PaletteAction::LaunchAgentModal,
                },
                PaletteItem {
                    icon: p_icon("agent"),
                    title: "Agents: Verify Worktree Build/Tests (v)".to_string(),
                    subtitle: "Run verification command in selected agent's isolated worktree".to_string(),
                    action: PaletteAction::VerifyAgentWorktree,
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
                    icon: p_icon("copy"),
                    title: "Edit: Copy (Ctrl+C)".to_string(),
                    subtitle: "Copy selection or current line to clipboard".to_string(),
                    action: PaletteAction::EditorCopy,
                },
                PaletteItem {
                    icon: p_icon("cut"),
                    title: "Edit: Cut (Ctrl+X)".to_string(),
                    subtitle: "Cut selection or current line to clipboard".to_string(),
                    action: PaletteAction::EditorCut,
                },
                PaletteItem {
                    icon: p_icon("paste"),
                    title: "Edit: Paste (Ctrl+V)".to_string(),
                    subtitle: "Paste text from system clipboard at cursor".to_string(),
                    action: PaletteAction::EditorPaste,
                },
                PaletteItem {
                    icon: p_icon("select_all"),
                    title: "Edit: Select All (Ctrl+A)".to_string(),
                    subtitle: "Select all text in active editor buffer".to_string(),
                    action: PaletteAction::EditorSelectAll,
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
            PaletteAction::VerifyAgentWorktree => {
                self.verify_active_agent();
            }
            PaletteAction::JumpToDiagnostic => {
                let _ = self.jump_to_terminal_diagnostic();
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
            PaletteAction::EditorCopy => {
                self.editor_copy();
                self.active_pane = ActivePane::FilePreview;
            }
            PaletteAction::EditorCut => {
                self.editor_cut();
                self.active_pane = ActivePane::FilePreview;
            }
            PaletteAction::EditorPaste => {
                self.editor_paste();
                self.active_pane = ActivePane::FilePreview;
            }
            PaletteAction::EditorSelectAll => {
                self.editor_select_all();
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
}

