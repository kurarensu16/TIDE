//! Git Worktree lifecycle management, bootstrapping, and diff inspection.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug)]
pub struct WorktreeInfo {
    #[allow(dead_code)]
    pub id: String,
    pub branch: String,
    pub path: PathBuf,
    pub base_branch: String,
}

impl WorktreeInfo {
    /// Creates a new isolated Git worktree and branch for an agent session.
    pub fn create(repo_root: &Path, session_id: &str, base_branch: &str) -> anyhow::Result<Self> {
        let worktree_dir = repo_root.join(".tide").join("worktrees").join(session_id);
        let branch = format!("tide/{}", session_id);

        if let Some(parent) = worktree_dir.parent() {
            fs::create_dir_all(parent)?;
        }

        // Clean up any stale worktree or branch with same name
        let _ = Command::new("git")
            .args(["worktree", "remove", "--force", &worktree_dir.to_string_lossy()])
            .current_dir(repo_root)
            .output();

        let _ = Command::new("git")
            .args(["branch", "-D", &branch])
            .current_dir(repo_root)
            .output();

        // git worktree add -b <branch> <path> <base_branch>
        let output = Command::new("git")
            .args([
                "worktree",
                "add",
                "-b",
                &branch,
                &worktree_dir.to_string_lossy(),
                base_branch,
            ])
            .current_dir(repo_root)
            .output()?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("git worktree add failed: {}", err_msg));
        }

        let info = Self {
            id: session_id.to_string(),
            branch,
            path: worktree_dir.clone(),
            base_branch: base_branch.to_string(),
        };

        // Run bootstrap hooks (e.g. copy .env, share target dir)
        info.bootstrap(repo_root)?;

        Ok(info)
    }

    /// Bootstraps the isolated worktree with untracked configs and shared build caches.
    fn bootstrap(&self, repo_root: &Path) -> anyhow::Result<()> {
        // Copy .env if present in root
        let root_env = repo_root.join(".env");
        if root_env.exists() {
            let target_env = self.path.join(".env");
            let _ = fs::copy(&root_env, &target_env);
        }

        // Create .cargo/config.toml with shared target-dir to avoid re-compiling all crates
        let root_cargo = repo_root.join("Cargo.toml");
        if root_cargo.exists() {
            let cargo_dir = self.path.join(".cargo");
            fs::create_dir_all(&cargo_dir)?;
            let config_content = format!(
                "[build]\ntarget-dir = \"{}\"\n",
                repo_root.join("target").to_string_lossy().replace('\\', "/")
            );
            fs::write(cargo_dir.join("config.toml"), config_content)?;
        }

        Ok(())
    }

    /// Fetches changed files and unified diff between the worktree and its base branch.
    pub fn get_diff(&self, _repo_root: &Path) -> anyhow::Result<(Vec<String>, String)> {
        // Uncommitted / staged changes in worktree
        let status_out = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.path)
            .output()?;

        let changed_files: Vec<String> = String::from_utf8_lossy(&status_out.stdout)
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        // Unified diff against base branch
        let diff_out = Command::new("git")
            .args(["diff", &format!("{}...HEAD", self.base_branch)])
            .current_dir(&self.path)
            .output()?;

        let mut diff_text = String::from_utf8_lossy(&diff_out.stdout).to_string();
        if diff_text.is_empty() {
            // Also check uncommitted diff inside worktree
            let uncommitted = Command::new("git")
                .args(["diff", "HEAD"])
                .current_dir(&self.path)
                .output()?;
            diff_text = String::from_utf8_lossy(&uncommitted.stdout).to_string();
        }

        if diff_text.is_empty() {
            diff_text = "No code differences detected between worktree and base branch.".to_string();
        }

        Ok((changed_files, diff_text))
    }

    /// Merges the worktree branch into the base branch.
    pub fn merge_into_base(&self, repo_root: &Path) -> anyhow::Result<()> {
        let output = Command::new("git")
            .args(["merge", &self.branch, "--no-edit"])
            .current_dir(repo_root)
            .output()?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Merge failed: {}", err));
        }
        Ok(())
    }

    /// Tears down and deletes the worktree and temporary branch.
    pub fn cleanup(&self, repo_root: &Path) -> anyhow::Result<()> {
        let _ = Command::new("git")
            .args(["worktree", "remove", "--force", &self.path.to_string_lossy()])
            .current_dir(repo_root)
            .output();

        let _ = Command::new("git")
            .args(["branch", "-D", &self.branch])
            .current_dir(repo_root)
            .output();

        Ok(())
    }
}
