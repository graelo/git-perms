use std::process;

use clap::{CommandFactory, Parser};

use git_perms::config::{Command, Config, HookSubcommand};
use git_perms::{git, hooks, perms};

fn main() {
    let config = Config::parse();
    match run(config) {
        Ok(exit_code) => process::exit(exit_code),
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    }
}

fn run(config: Config) -> git_perms::Result<i32> {
    match config.command {
        Command::Save => {
            let root = git::repo_root()?;
            let files = git::tracked_files(&root)?;
            let entries = perms::scan(&root, &files)?;
            let content = perms::serialize(&entries);
            let perms_path = root.join(".gitperms");
            std::fs::write(&perms_path, content)?;
            println!("saved permissions for {} files to .gitperms", entries.len());
            Ok(0)
        }
        Command::Restore => {
            let root = git::repo_root()?;
            let perms_path = root.join(".gitperms");
            let content = std::fs::read_to_string(&perms_path)?;
            let entries = perms::parse(&content)?;
            let result = perms::apply(&root, &entries)?;
            for path in &result.skipped {
                eprintln!("warning: {} not found on disk, skipping", path.display());
            }
            println!(
                "restored permissions for {} files ({} skipped)",
                result.applied,
                result.skipped.len()
            );
            Ok(0)
        }
        Command::Diff { quiet } => {
            let root = git::repo_root()?;
            let files = git::tracked_files(&root)?;
            let actual = perms::scan(&root, &files)?;
            let perms_path = root.join(".gitperms");
            let stored = if perms_path.exists() {
                let content = std::fs::read_to_string(&perms_path)?;
                perms::parse(&content)?
            } else {
                Vec::new()
            };
            let diffs = perms::diff(&stored, &actual);
            if diffs.is_empty() {
                if !quiet {
                    println!("no permission differences");
                }
                Ok(0)
            } else {
                if !quiet {
                    for d in &diffs {
                        match d {
                            perms::DiffEntry::Modified {
                                path,
                                stored,
                                actual,
                            } => {
                                println!("M {:04o} -> {:04o} {}", stored, actual, path.display());
                            }
                            perms::DiffEntry::Added { path, mode } => {
                                println!("A {:04o} {}", mode, path.display());
                            }
                            perms::DiffEntry::Removed { path, mode } => {
                                println!("R {:04o} {}", mode, path.display());
                            }
                        }
                    }
                }
                Ok(1)
            }
        }
        Command::Hook { command } => match command {
            HookSubcommand::Install => {
                let result = hooks::install()?;
                for name in &result.installed {
                    println!("installed {name} hook");
                }
                Ok(0)
            }
            HookSubcommand::Uninstall => {
                let result = hooks::uninstall()?;
                for name in &result.removed {
                    println!("removed {name} hook");
                }
                for name in &result.skipped {
                    eprintln!("skipping {name}: not installed by git-perms");
                }
                Ok(0)
            }
        },
        Command::GenerateCompletion { shell } => {
            clap_complete::generate(
                shell,
                &mut Config::command(),
                "git-perms",
                &mut std::io::stdout(),
            );
            Ok(0)
        }
    }
}
