use std::fs;
use std::os::unix::fs::PermissionsExt;

use crate::error::Error;
use crate::{Result, git};

const MARKER: &str = "# Installed by git-perms";

const RESTORE_HOOK: &str = r#"#!/usr/bin/env sh
# Installed by git-perms
if command -v git-perms >/dev/null 2>&1; then
    git-perms restore
fi
"#;

const PRE_COMMIT_HOOK: &str = r#"#!/usr/bin/env sh
# Installed by git-perms
if command -v git-perms >/dev/null 2>&1; then
    MODE=$(git config --get perms.preCommit || echo "auto")
    case "$MODE" in
        auto)
            git-perms save
            git add .gitperms
            ;;
        warn)
            if ! git-perms diff --quiet 2>/dev/null; then
                echo "git-perms: permissions differ from .gitperms (run 'git perms save')"
            fi
            ;;
        block)
            if ! git-perms diff --quiet 2>/dev/null; then
                echo "git-perms: permissions differ from .gitperms (run 'git perms save')" >&2
                exit 1
            fi
            ;;
    esac
fi
"#;

const HOOK_NAMES: &[(&str, &str)] = &[
    ("post-checkout", RESTORE_HOOK),
    ("post-merge", RESTORE_HOOK),
    ("post-rewrite", RESTORE_HOOK),
    ("pre-commit", PRE_COMMIT_HOOK),
];

/// Result of installing hooks.
pub struct InstallResult {
    pub installed: Vec<&'static str>,
}

/// Result of uninstalling hooks.
pub struct UninstallResult {
    pub removed: Vec<&'static str>,
    pub skipped: Vec<&'static str>,
}

/// Install all git-perms hooks.
pub fn install() -> Result<InstallResult> {
    let hook_dir = git::hook_dir()?;

    if !hook_dir.exists() {
        fs::create_dir_all(&hook_dir)?;
    }

    let mut result = InstallResult {
        installed: Vec::new(),
    };

    for (name, content) in HOOK_NAMES {
        let path = hook_dir.join(name);

        if path.exists() {
            let existing = fs::read_to_string(&path)?;
            if !existing.contains(MARKER) {
                return Err(Error::HookExists(name.to_string()));
            }
        }

        fs::write(&path, content)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
        result.installed.push(name);
    }

    Ok(result)
}

/// Uninstall all git-perms hooks.
pub fn uninstall() -> Result<UninstallResult> {
    let hook_dir = git::hook_dir()?;

    let mut result = UninstallResult {
        removed: Vec::new(),
        skipped: Vec::new(),
    };

    for (name, _) in HOOK_NAMES {
        let path = hook_dir.join(name);

        if path.exists() {
            let content = fs::read_to_string(&path)?;
            if content.contains(MARKER) {
                fs::remove_file(&path)?;
                result.removed.push(name);
            } else {
                result.skipped.push(name);
            }
        }
    }

    Ok(result)
}
