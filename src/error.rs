use std::io;

/// Errors that can occur in git-perms.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not a git repository (or any parent up to mount point)")]
    NotAGitRepo,

    #[error("git command failed: {0}")]
    GitCommandFailed(String),

    #[error("failed to parse .gitperms at line {line}: {reason}")]
    ParseError { line: usize, reason: String },

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("hook {0} already exists and was not installed by git-perms")]
    HookExists(String),
}
