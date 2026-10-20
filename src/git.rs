//! Git status and commit history integration.

use std::process::Command;

#[derive(Clone, Debug)]
pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub relative_time: String,
}

#[derive(Clone, Debug)]
pub struct GitRepoInfo {
    pub branch: String,
    pub is_clean: bool,
    pub changed_files: Vec<String>,
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

        let changed_files: Vec<String> = status_output
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        let is_clean = changed_files.is_empty();

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
            recent_commits,
        }
    }
}
