use std::path::{Path, PathBuf};
use std::process::Command;

use crate::Result;
use crate::error::Error;

/// Mode for the pre-commit hook behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreCommitMode {
    #[default]
    Auto,
    Warn,
    Block,
}

impl std::str::FromStr for PreCommitMode {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s.trim() {
            "warn" => Self::Warn,
            "block" => Self::Block,
            _ => Self::Auto,
        })
    }
}

/// Get the repository root directory.
pub fn repo_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| Error::GitCommandFailed(format!("failed to run git: {e}")))?;

    if !output.status.success() {
        return Err(Error::NotAGitRepo);
    }

    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(root))
}

/// Get sorted list of git-tracked file paths (relative to repo root).
pub fn tracked_files(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .args(["ls-files"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| Error::GitCommandFailed(format!("failed to run git ls-files: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::GitCommandFailed(stderr));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut files: Vec<PathBuf> = text
        .lines()
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect();
    files.sort();
    Ok(files)
}

/// Read the pre-commit mode from git config.
pub fn pre_commit_config() -> PreCommitMode {
    let output = Command::new("git")
        .args(["config", "--get", "perms.preCommit"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let value = String::from_utf8_lossy(&o.stdout);
            value.trim().parse().unwrap_or_default()
        }
        _ => PreCommitMode::Auto,
    }
}

/// Stage a file in the git index.
pub fn stage_file(path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["add"])
        .arg(path)
        .output()
        .map_err(|e| Error::GitCommandFailed(format!("failed to run git add: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::GitCommandFailed(stderr));
    }

    Ok(())
}

/// Get the git hooks directory.
pub fn hook_dir() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map_err(|e| Error::GitCommandFailed(format!("failed to run git rev-parse: {e}")))?;

    if !output.status.success() {
        return Err(Error::NotAGitRepo);
    }

    let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(git_dir).join("hooks"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> PreCommitMode {
        s.parse().unwrap()
    }

    #[test]
    fn test_pre_commit_mode_auto() {
        assert_eq!(parse("auto"), PreCommitMode::Auto);
    }

    #[test]
    fn test_pre_commit_mode_warn() {
        assert_eq!(parse("warn"), PreCommitMode::Warn);
    }

    #[test]
    fn test_pre_commit_mode_block() {
        assert_eq!(parse("block"), PreCommitMode::Block);
    }

    #[test]
    fn test_pre_commit_mode_unknown_defaults_to_auto() {
        assert_eq!(parse("unknown"), PreCommitMode::Auto);
    }

    #[test]
    fn test_pre_commit_mode_empty_defaults_to_auto() {
        assert_eq!(parse(""), PreCommitMode::Auto);
    }

    #[test]
    fn test_pre_commit_mode_with_whitespace() {
        assert_eq!(parse("  warn  "), PreCommitMode::Warn);
    }
}
