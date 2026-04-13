use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Debug, Parser)]
#[command(
    name = "git-perms",
    about = "Store and restore Unix file permissions across git operations."
)]
pub struct Config {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Scan git-tracked files and write permissions to .gitperms
    Save,
    /// Read .gitperms and apply permissions to the working tree
    Restore,
    /// Show permission differences between .gitperms and the filesystem
    Diff {
        /// Exit with status 1 if differences exist, print nothing
        #[arg(long)]
        quiet: bool,
    },
    /// Manage git hooks for automatic permission save/restore
    Hook {
        #[command(subcommand)]
        command: HookSubcommand,
    },
    /// Generate shell completions
    GenerateCompletion {
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Debug, Subcommand)]
pub enum HookSubcommand {
    /// Install git hooks (post-checkout, post-merge, post-rewrite, pre-commit)
    Install,
    /// Remove git hooks installed by git-perms
    Uninstall,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse(args: &[&str]) -> Config {
        Config::parse_from(args)
    }

    #[test]
    fn test_save() {
        let config = parse(&["git-perms", "save"]);
        assert!(matches!(config.command, Command::Save));
    }

    #[test]
    fn test_restore() {
        let config = parse(&["git-perms", "restore"]);
        assert!(matches!(config.command, Command::Restore));
    }

    #[test]
    fn test_diff() {
        let config = parse(&["git-perms", "diff"]);
        assert!(matches!(config.command, Command::Diff { quiet: false }));
    }

    #[test]
    fn test_diff_quiet() {
        let config = parse(&["git-perms", "diff", "--quiet"]);
        assert!(matches!(config.command, Command::Diff { quiet: true }));
    }

    #[test]
    fn test_hook_install() {
        let config = parse(&["git-perms", "hook", "install"]);
        assert!(matches!(
            config.command,
            Command::Hook {
                command: HookSubcommand::Install
            }
        ));
    }

    #[test]
    fn test_hook_uninstall() {
        let config = parse(&["git-perms", "hook", "uninstall"]);
        assert!(matches!(
            config.command,
            Command::Hook {
                command: HookSubcommand::Uninstall
            }
        ));
    }

    #[test]
    fn test_generate_completion_bash() {
        let config = parse(&["git-perms", "generate-completion", "bash"]);
        assert!(matches!(
            config.command,
            Command::GenerateCompletion { shell: Shell::Bash }
        ));
    }
}
