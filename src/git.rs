//! Git status, staging, commits, and history integration.

use std::process::Command;

#[derive(Clone, Debug)]
pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub relative_time: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GitFileStatusCode {
    Added,
    Modified,
    Deleted,
    Renamed,
    Untracked,
    Unknown,
}

impl GitFileStatusCode {
    pub fn badge(&self) -> (&'static str, ratatui::style::Color) {
        match self {
            GitFileStatusCode::Added => ("[+ Added]", ratatui::style::Color::Rgb(34, 197, 94)),
            GitFileStatusCode::Modified => ("[~ Mod]", ratatui::style::Color::Rgb(56, 189, 248)),
            GitFileStatusCode::Deleted => ("[- Del]", ratatui::style::Color::Rgb(244, 63, 94)),
            GitFileStatusCode::Renamed => ("[→ Ren]", ratatui::style::Color::Rgb(168, 85, 247)),
            GitFileStatusCode::Untracked => ("[? New]", ratatui::style::Color::Rgb(234, 179, 8)),
            GitFileStatusCode::Unknown => ("[• Raw]", ratatui::style::Color::Rgb(148, 163, 184)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitFileItem {
    pub path: String,
    pub status: GitFileStatusCode,
    pub is_staged: bool,
}

#[derive(Clone, Debug)]
pub struct GitRepoInfo {
    pub branch: String,
    pub is_clean: bool,
    #[allow(dead_code)]
    pub changed_files: Vec<String>,
    pub staged_files: Vec<GitFileItem>,
    pub unstaged_files: Vec<GitFileItem>,
    pub recent_commits: Vec<GitCommit>,
}

impl GitRepoInfo {
    pub fn fetch(repo_path: &str) -> Self {
        let mut branch = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(repo_path)
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        if branch.is_empty() || branch == "HEAD" {
            branch = Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .current_dir(repo_path)
                .output()
                .ok()
                .and_then(|out| String::from_utf8(out.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
        }

        if branch.is_empty() || branch == "HEAD" {
            branch = "main".to_string();
        }

        let status_output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(repo_path)
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .unwrap_or_default();

        let mut changed_files: Vec<String> = Vec::new();
        let mut staged_files: Vec<GitFileItem> = Vec::new();
        let mut unstaged_files: Vec<GitFileItem> = Vec::new();

        for line in status_output.lines() {
            if line.len() < 3 {
                continue;
            }
            let index_char = line.chars().next().unwrap_or(' ');
            let worktree_char = line.chars().nth(1).unwrap_or(' ');
            let raw_path = line[3..].trim().to_string();
            let clean_path = raw_path.trim_matches('"').to_string();

            if !clean_path.is_empty() {
                changed_files.push(clean_path.clone());
            }

            // Untracked files
            if index_char == '?' && worktree_char == '?' {
                unstaged_files.push(GitFileItem {
                    path: clean_path,
                    status: GitFileStatusCode::Untracked,
                    is_staged: false,
                });
                continue;
            }

            // Staged changes
            if index_char != ' ' && index_char != '?' {
                let status = match index_char {
                    'A' => GitFileStatusCode::Added,
                    'M' => GitFileStatusCode::Modified,
                    'D' => GitFileStatusCode::Deleted,
                    'R' => GitFileStatusCode::Renamed,
                    _ => GitFileStatusCode::Unknown,
                };
                staged_files.push(GitFileItem {
                    path: clean_path.clone(),
                    status,
                    is_staged: true,
                });
            }

            // Unstaged changes
            if worktree_char != ' ' && worktree_char != '?' {
                let status = match worktree_char {
                    'M' => GitFileStatusCode::Modified,
                    'D' => GitFileStatusCode::Deleted,
                    _ => GitFileStatusCode::Unknown,
                };
                unstaged_files.push(GitFileItem {
                    path: clean_path,
                    status,
                    is_staged: false,
                });
            }
        }

        let is_clean = staged_files.is_empty() && unstaged_files.is_empty();

        let log_output = Command::new("git")
            .args(["log", "-n", "4", "--pretty=format:%h|%s|%cr"])
            .current_dir(repo_path)
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .unwrap_or_default();

        let recent_commits = log_output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 3 {
                    Some(GitCommit {
                        hash: parts[0].to_string(),
                        message: parts[1].to_string(),
                        relative_time: parts[2].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Self {
            branch,
            is_clean,
            changed_files,
            staged_files,
            unstaged_files,
            recent_commits,
        }
    }

    /// Stage a specific file (`git add <file>`)
    pub fn stage_file(repo_path: &str, file_path: &str) -> anyhow::Result<()> {
        let output = Command::new("git")
            .args(["add", "--", file_path])
            .current_dir(repo_path)
            .output()?;
        if output.status.success() {
            Ok(())
        } else {
            anyhow::bail!(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Unstage a specific file (`git restore --staged <file>`)
    pub fn unstage_file(repo_path: &str, file_path: &str) -> anyhow::Result<()> {
        let output = Command::new("git")
            .args(["restore", "--staged", "--", file_path])
            .current_dir(repo_path)
            .output()?;
        if output.status.success() {
            Ok(())
        } else {
            anyhow::bail!(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Stage all files in working tree (`git add -A`)
    pub fn stage_all(repo_path: &str) -> anyhow::Result<()> {
        let output = Command::new("git")
            .args(["add", "-A"])
            .current_dir(repo_path)
            .output()?;
        if output.status.success() {
            Ok(())
        } else {
            anyhow::bail!(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Unstage all files (`git restore --staged .`)
    pub fn unstage_all(repo_path: &str) -> anyhow::Result<()> {
        let output = Command::new("git")
            .args(["restore", "--staged", "."])
            .current_dir(repo_path)
            .output()?;
        if output.status.success() {
            Ok(())
        } else {
            anyhow::bail!(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Commit staged changes (`git commit -m <msg>`)
    pub fn commit(repo_path: &str, message: &str) -> anyhow::Result<String> {
        let output = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(repo_path)
            .output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    /// Push current branch to remote (`git push`)
    pub fn push(repo_path: &str) -> anyhow::Result<String> {
        let output = Command::new("git")
            .args(["push"])
            .current_dir(repo_path)
            .output()?;
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Ok(if !s.is_empty() { s } else { err })
        } else {
            anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    /// Pull from remote (`git pull`)
    pub fn pull(repo_path: &str) -> anyhow::Result<String> {
        let output = Command::new("git")
            .args(["pull"])
            .current_dir(repo_path)
            .output()?;
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Ok(if !s.is_empty() { s } else { err })
        } else {
            anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    /// View git diff for a specific file (staged or unstaged)
    pub fn diff_file(repo_path: &str, file_path: &str, staged: bool) -> anyhow::Result<String> {
        let mut cmd = Command::new("git");
        cmd.arg("diff");
        if staged {
            cmd.arg("--staged");
        }
        cmd.args(["--", file_path]).current_dir(repo_path);
        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if stdout.trim().is_empty() {
            // For untracked files, render preview of new file content
            let full_path = std::path::Path::new(repo_path).join(file_path);
            if let Ok(content) = std::fs::read_to_string(&full_path) {
                let lines: Vec<String> = content.lines().map(|l| format!("+{}", l)).collect();
                return Ok(format!("--- /dev/null\n+++ b/{}\n@@ -0,0 +1,{} @@\n{}", file_path, lines.len(), lines.join("\n")));
            }
        }
        Ok(stdout)
    }

    /// List all local and remote branches with status and commit info
    pub fn list_branches(repo_path: &str) -> anyhow::Result<Vec<GitBranchItem>> {
        let output = Command::new("git")
            .args(["branch", "-a", "-v", "--no-color"])
            .current_dir(repo_path)
            .output()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(parse_branch_output(&stdout))
        } else {
            anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    /// Switch to an existing branch (`git checkout <branch>`)
    pub fn checkout_branch(repo_path: &str, branch_name: &str) -> anyhow::Result<String> {
        let clean_name = if branch_name.starts_with("remotes/origin/") {
            branch_name.trim_start_matches("remotes/origin/")
        } else if branch_name.starts_with("remotes/") {
            branch_name.trim_start_matches("remotes/")
        } else {
            branch_name
        };

        // Try checking out by clean name first
        let output = Command::new("git")
            .args(["checkout", clean_name])
            .current_dir(repo_path)
            .output()?;

        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Ok(if !s.is_empty() { s } else { err })
        } else if branch_name.starts_with("remotes/") {
            // Fallback to tracking remote ref
            let track_output = Command::new("git")
                .args(["checkout", "--track", branch_name.trim_start_matches("remotes/")])
                .current_dir(repo_path)
                .output()?;
            if track_output.status.success() {
                let s = String::from_utf8_lossy(&track_output.stdout).trim().to_string();
                let err = String::from_utf8_lossy(&track_output.stderr).trim().to_string();
                Ok(if !s.is_empty() { s } else { err })
            } else {
                anyhow::bail!(String::from_utf8_lossy(&track_output.stderr).trim().to_string())
            }
        } else {
            anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    /// Create and checkout a new branch (`git checkout -b <branch>`)
    pub fn create_and_checkout_branch(repo_path: &str, branch_name: &str) -> anyhow::Result<String> {
        let output = Command::new("git")
            .args(["checkout", "-b", branch_name])
            .current_dir(repo_path)
            .output()?;
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Ok(if !s.is_empty() { s } else { err })
        } else {
            anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    /// Delete a branch (`git branch -d` or `-D`)
    pub fn delete_branch(repo_path: &str, branch_name: &str, force: bool) -> anyhow::Result<String> {
        let flag = if force { "-D" } else { "-d" };
        let output = Command::new("git")
            .args(["branch", flag, branch_name])
            .current_dir(repo_path)
            .output()?;
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Ok(if !s.is_empty() { s } else { err })
        } else {
            anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    /// List all git stashes
    pub fn list_stashes(repo_path: &str) -> anyhow::Result<Vec<GitStashItem>> {
        let output = Command::new("git")
            .args(["stash", "list", "--format=%gd%x09%cr%x09%gs"])
            .current_dir(repo_path)
            .output()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(parse_stash_output(&stdout))
        } else {
            anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    /// Stash changes with optional message and untracked files option
    pub fn stash_save(repo_path: &str, message: Option<&str>, include_untracked: bool) -> anyhow::Result<String> {
        let mut cmd = Command::new("git");
        cmd.args(["stash", "push"]);
        if include_untracked {
            cmd.arg("-u");
        }
        if let Some(msg) = message {
            let trimmed = msg.trim();
            if !trimmed.is_empty() {
                cmd.args(["-m", trimmed]);
            }
        }
        cmd.current_dir(repo_path);
        let output = cmd.output()?;
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Ok(if !s.is_empty() { s } else { err })
        } else {
            anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    /// Pop a stash by index (`git stash pop stash@{idx}`)
    pub fn stash_pop(repo_path: &str, index: usize) -> anyhow::Result<String> {
        let selector = format!("stash@{{{}}}", index);
        let output = Command::new("git")
            .args(["stash", "pop", &selector])
            .current_dir(repo_path)
            .output()?;
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Ok(if !s.is_empty() { s } else { err })
        } else {
            anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    /// Apply a stash without dropping it (`git stash apply stash@{idx}`)
    pub fn stash_apply(repo_path: &str, index: usize) -> anyhow::Result<String> {
        let selector = format!("stash@{{{}}}", index);
        let output = Command::new("git")
            .args(["stash", "apply", &selector])
            .current_dir(repo_path)
            .output()?;
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Ok(if !s.is_empty() { s } else { err })
        } else {
            anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    /// Drop a stash by index (`git stash drop stash@{idx}`)
    pub fn stash_drop(repo_path: &str, index: usize) -> anyhow::Result<String> {
        let selector = format!("stash@{{{}}}", index);
        let output = Command::new("git")
            .args(["stash", "drop", &selector])
            .current_dir(repo_path)
            .output()?;
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Ok(if !s.is_empty() { s } else { err })
        } else {
            anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    /// View full diff of a stash (`git stash show -p stash@{idx}`)
    pub fn stash_diff(repo_path: &str, index: usize) -> anyhow::Result<String> {
        let selector = format!("stash@{{{}}}", index);
        let output = Command::new("git")
            .args(["stash", "show", "-p", &selector])
            .current_dir(repo_path)
            .output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }
}

/// A git branch item representing local, remote, or worktree branches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitBranchItem {
    pub name: String,
    pub is_current: bool,
    pub is_worktree: bool,
    pub is_remote: bool,
    pub commit_hash: String,
    pub commit_msg: String,
}

impl GitBranchItem {
    /// Return short display name without remotes/ or origin/ prefix
    pub fn display_name(&self) -> &str {
        if let Some(stripped) = self.name.strip_prefix("remotes/origin/") {
            stripped
        } else if let Some(stripped) = self.name.strip_prefix("remotes/") {
            stripped
        } else {
            &self.name
        }
    }
}

/// A git stash item representing a saved stash entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitStashItem {
    pub index: usize,
    pub selector: String,
    pub relative_time: String,
    pub message: String,
}

/// Parse raw output of `git branch -a -v --no-color`
pub fn parse_branch_output(raw: &str) -> Vec<GitBranchItem> {
    let mut branches = Vec::new();
    for line in raw.lines() {
        if line.len() < 2 {
            continue;
        }
        let prefix = &line[..2];
        let rest = line[2..].trim_start();
        if rest.contains(" -> ") {
            // Skip symbolic refs like remotes/origin/HEAD -> origin/main
            continue;
        }
        let is_current = prefix.starts_with('*');
        let is_worktree = prefix.starts_with('+');

        let mut tokens = rest.split_whitespace();
        let name = match tokens.next() {
            Some(n) => n.to_string(),
            None => continue,
        };
        let is_remote = name.starts_with("remotes/");
        let commit_hash = tokens.next().unwrap_or("").to_string();
        let commit_msg: String = tokens.collect::<Vec<_>>().join(" ");

        branches.push(GitBranchItem {
            name,
            is_current,
            is_worktree,
            is_remote,
            commit_hash,
            commit_msg,
        });
    }
    branches
}

/// Parse raw output of `git stash list --format=%gd%x09%cr%x09%gs`
pub fn parse_stash_output(raw: &str) -> Vec<GitStashItem> {
    let mut stashes = Vec::new();
    for (idx, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split('\t').collect();
        if parts.len() >= 3 {
            stashes.push(GitStashItem {
                index: idx,
                selector: parts[0].to_string(),
                relative_time: parts[1].to_string(),
                message: parts[2].to_string(),
            });
        } else if let Some((selector, rest)) = trimmed.split_once(':') {
            stashes.push(GitStashItem {
                index: idx,
                selector: selector.trim().to_string(),
                relative_time: String::new(),
                message: rest.trim().to_string(),
            });
        }
    }
    stashes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_branch_output() {
        let sample = "\
* main                  8efeb68 feat: interactive agent modal
+ tide/antigravity-1    8efeb68 feat: interactive agent modal
  feature/login         a1b2c3d implement oauth2 login flow
  remotes/origin/HEAD -> origin/main
  remotes/origin/main   8efeb68 feat: interactive agent modal
";
        let branches = parse_branch_output(sample);
        assert_eq!(branches.len(), 4);

        assert_eq!(branches[0].name, "main");
        assert!(branches[0].is_current);
        assert!(!branches[0].is_worktree);
        assert!(!branches[0].is_remote);
        assert_eq!(branches[0].commit_hash, "8efeb68");
        assert_eq!(branches[0].commit_msg, "feat: interactive agent modal");

        assert_eq!(branches[1].name, "tide/antigravity-1");
        assert!(!branches[1].is_current);
        assert!(branches[1].is_worktree);
        assert_eq!(branches[1].display_name(), "tide/antigravity-1");

        assert_eq!(branches[2].name, "feature/login");
        assert!(!branches[2].is_current);
        assert!(!branches[2].is_worktree);
        assert!(!branches[2].is_remote);

        assert_eq!(branches[3].name, "remotes/origin/main");
        assert!(branches[3].is_remote);
        assert_eq!(branches[3].display_name(), "main");
    }

    #[test]
    fn test_parse_stash_output() {
        let sample = "\
stash@{0}\t10 minutes ago\tWIP on main: 8efeb68 feat: interactive agent modal
stash@{1}\t2 hours ago\tcustom stash message: save progress before refactor
";
        let stashes = parse_stash_output(sample);
        assert_eq!(stashes.len(), 2);

        assert_eq!(stashes[0].index, 0);
        assert_eq!(stashes[0].selector, "stash@{0}");
        assert_eq!(stashes[0].relative_time, "10 minutes ago");
        assert_eq!(stashes[0].message, "WIP on main: 8efeb68 feat: interactive agent modal");

        assert_eq!(stashes[1].index, 1);
        assert_eq!(stashes[1].selector, "stash@{1}");
        assert_eq!(stashes[1].relative_time, "2 hours ago");
        assert_eq!(stashes[1].message, "custom stash message: save progress before refactor");
    }

    #[test]
    fn test_parse_stash_output_colon_format() {
        let sample = "stash@{0}: WIP on main: 8efeb68 feat: something\n";
        let stashes = parse_stash_output(sample);
        assert_eq!(stashes.len(), 1);
        assert_eq!(stashes[0].selector, "stash@{0}");
        assert_eq!(stashes[0].message, "WIP on main: 8efeb68 feat: something");
    }
}
