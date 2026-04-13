use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::Result;
use crate::error::Error;

/// A single permissions entry.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PermsEntry {
    pub path: PathBuf,
    pub mode: u32,
}

/// A difference between stored and actual permissions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffEntry {
    Modified {
        path: PathBuf,
        stored: u32,
        actual: u32,
    },
    Added {
        path: PathBuf,
        mode: u32,
    },
    Removed {
        path: PathBuf,
        mode: u32,
    },
}

/// Result of applying permissions.
#[derive(Debug, Default)]
pub struct ApplyResult {
    pub applied: usize,
    pub skipped: Vec<PathBuf>,
}

/// Parse .gitperms file content into entries.
pub fn parse(content: &str) -> Result<Vec<PermsEntry>> {
    let mut entries = Vec::new();

    for (i, line) in content.lines().enumerate() {
        if line.is_empty() {
            continue;
        }

        let line_num = i + 1;

        let Some((mode_str, path_str)) = line.split_once(' ') else {
            return Err(Error::ParseError {
                line: line_num,
                reason: "expected '<mode> <path>'".to_string(),
            });
        };

        if mode_str.len() != 4 {
            return Err(Error::ParseError {
                line: line_num,
                reason: format!("mode must be exactly 4 octal digits, got '{mode_str}'"),
            });
        }

        let mode = u32::from_str_radix(mode_str, 8).map_err(|_| Error::ParseError {
            line: line_num,
            reason: format!("invalid octal mode '{mode_str}'"),
        })?;

        if path_str.is_empty() {
            return Err(Error::ParseError {
                line: line_num,
                reason: "path is empty".to_string(),
            });
        }

        entries.push(PermsEntry {
            path: PathBuf::from(path_str),
            mode,
        });
    }

    entries.sort();
    Ok(entries)
}

/// Serialize entries to .gitperms format.
///
/// Entries must be sorted by path (all public functions in this module
/// return sorted entries).
pub fn serialize(entries: &[PermsEntry]) -> String {
    let mut output = String::new();
    for entry in entries {
        output.push_str(&format!("{:04o} {}\n", entry.mode, entry.path.display()));
    }
    output
}

/// Scan the working tree for current permissions.
/// Only includes files that actually exist on disk (prune stale).
pub fn scan(repo_root: &Path, tracked_files: &[PathBuf]) -> Result<Vec<PermsEntry>> {
    let mut entries = Vec::new();

    for file in tracked_files {
        let full_path = repo_root.join(file);
        if let Ok(metadata) = full_path.symlink_metadata() {
            let mode = metadata.permissions().mode() & 0o7777;
            entries.push(PermsEntry {
                path: file.clone(),
                mode,
            });
        }
    }

    entries.sort();
    Ok(entries)
}

/// Apply stored permissions to the filesystem.
pub fn apply(repo_root: &Path, entries: &[PermsEntry]) -> Result<ApplyResult> {
    let mut result = ApplyResult::default();

    for entry in entries {
        let full_path = repo_root.join(&entry.path);
        if full_path.exists() {
            fs::set_permissions(&full_path, fs::Permissions::from_mode(entry.mode))?;
            result.applied += 1;
        } else {
            result.skipped.push(entry.path.clone());
        }
    }

    Ok(result)
}

/// Compare stored vs actual permissions.
/// Both inputs MUST be sorted by path.
pub fn diff(stored: &[PermsEntry], actual: &[PermsEntry]) -> Vec<DiffEntry> {
    let mut diffs = Vec::new();
    let mut si = 0;
    let mut ai = 0;

    while si < stored.len() && ai < actual.len() {
        let s = &stored[si];
        let a = &actual[ai];

        match s.path.cmp(&a.path) {
            std::cmp::Ordering::Less => {
                diffs.push(DiffEntry::Removed {
                    path: s.path.clone(),
                    mode: s.mode,
                });
                si += 1;
            }
            std::cmp::Ordering::Greater => {
                diffs.push(DiffEntry::Added {
                    path: a.path.clone(),
                    mode: a.mode,
                });
                ai += 1;
            }
            std::cmp::Ordering::Equal => {
                if s.mode != a.mode {
                    diffs.push(DiffEntry::Modified {
                        path: s.path.clone(),
                        stored: s.mode,
                        actual: a.mode,
                    });
                }
                si += 1;
                ai += 1;
            }
        }
    }

    while si < stored.len() {
        let s = &stored[si];
        diffs.push(DiffEntry::Removed {
            path: s.path.clone(),
            mode: s.mode,
        });
        si += 1;
    }

    while ai < actual.len() {
        let a = &actual[ai];
        diffs.push(DiffEntry::Added {
            path: a.path.clone(),
            mode: a.mode,
        });
        ai += 1;
    }

    diffs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid() {
        let content = "0644 src/main.rs\n0755 scripts/build.sh\n";
        let entries = parse(content).unwrap();
        assert_eq!(entries.len(), 2);
        // sorted by path
        assert_eq!(entries[0].path, PathBuf::from("scripts/build.sh"));
        assert_eq!(entries[0].mode, 0o755);
        assert_eq!(entries[1].path, PathBuf::from("src/main.rs"));
        assert_eq!(entries[1].mode, 0o644);
    }

    #[test]
    fn test_parse_empty() {
        let entries = parse("").unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_malformed_mode() {
        let content = "09xx src/main.rs\n";
        let result = parse(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::ParseError { line: 1, .. }));
    }

    #[test]
    fn test_parse_missing_path() {
        let content = "0644\n";
        let result = parse(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::ParseError { line: 1, .. }));
    }

    #[test]
    fn test_serialize() {
        let entries = vec![
            PermsEntry {
                path: PathBuf::from("build.sh"),
                mode: 0o755,
            },
            PermsEntry {
                path: PathBuf::from("src/main.rs"),
                mode: 0o644,
            },
        ];
        let output = serialize(&entries);
        assert_eq!(output, "0755 build.sh\n0644 src/main.rs\n");
    }

    #[test]
    fn test_serialize_empty() {
        let output = serialize(&[]);
        assert_eq!(output, "");
    }

    #[test]
    fn test_round_trip() {
        let entries = vec![
            PermsEntry {
                path: PathBuf::from("a.txt"),
                mode: 0o644,
            },
            PermsEntry {
                path: PathBuf::from("b.sh"),
                mode: 0o755,
            },
            PermsEntry {
                path: PathBuf::from("c/d.rs"),
                mode: 0o600,
            },
        ];
        let serialized = serialize(&entries);
        let parsed = parse(&serialized).unwrap();
        assert_eq!(entries, parsed);
    }

    #[test]
    fn test_diff_no_changes() {
        let entries = vec![
            PermsEntry {
                path: PathBuf::from("a.txt"),
                mode: 0o644,
            },
            PermsEntry {
                path: PathBuf::from("b.sh"),
                mode: 0o755,
            },
        ];
        let diffs = diff(&entries, &entries);
        assert!(diffs.is_empty());
    }

    #[test]
    fn test_diff_modified() {
        let stored = vec![PermsEntry {
            path: PathBuf::from("a.txt"),
            mode: 0o644,
        }];
        let actual = vec![PermsEntry {
            path: PathBuf::from("a.txt"),
            mode: 0o755,
        }];
        let diffs = diff(&stored, &actual);
        assert_eq!(diffs.len(), 1);
        assert_eq!(
            diffs[0],
            DiffEntry::Modified {
                path: PathBuf::from("a.txt"),
                stored: 0o644,
                actual: 0o755,
            }
        );
    }

    #[test]
    fn test_diff_added() {
        let stored = vec![];
        let actual = vec![PermsEntry {
            path: PathBuf::from("new.txt"),
            mode: 0o644,
        }];
        let diffs = diff(&stored, &actual);
        assert_eq!(diffs.len(), 1);
        assert_eq!(
            diffs[0],
            DiffEntry::Added {
                path: PathBuf::from("new.txt"),
                mode: 0o644,
            }
        );
    }

    #[test]
    fn test_diff_removed() {
        let stored = vec![PermsEntry {
            path: PathBuf::from("old.txt"),
            mode: 0o644,
        }];
        let actual = vec![];
        let diffs = diff(&stored, &actual);
        assert_eq!(diffs.len(), 1);
        assert_eq!(
            diffs[0],
            DiffEntry::Removed {
                path: PathBuf::from("old.txt"),
                mode: 0o644,
            }
        );
    }

    #[test]
    fn test_diff_mixed() {
        let stored = vec![
            PermsEntry {
                path: PathBuf::from("a.txt"),
                mode: 0o644,
            },
            PermsEntry {
                path: PathBuf::from("b.txt"),
                mode: 0o644,
            },
            PermsEntry {
                path: PathBuf::from("c.txt"),
                mode: 0o600,
            },
        ];
        let actual = vec![
            PermsEntry {
                path: PathBuf::from("b.txt"),
                mode: 0o755,
            },
            PermsEntry {
                path: PathBuf::from("c.txt"),
                mode: 0o600,
            },
            PermsEntry {
                path: PathBuf::from("d.txt"),
                mode: 0o644,
            },
        ];
        let diffs = diff(&stored, &actual);
        assert_eq!(diffs.len(), 3);
        assert_eq!(
            diffs[0],
            DiffEntry::Removed {
                path: PathBuf::from("a.txt"),
                mode: 0o644,
            }
        );
        assert_eq!(
            diffs[1],
            DiffEntry::Modified {
                path: PathBuf::from("b.txt"),
                stored: 0o644,
                actual: 0o755,
            }
        );
        assert_eq!(
            diffs[2],
            DiffEntry::Added {
                path: PathBuf::from("d.txt"),
                mode: 0o644,
            }
        );
    }
}
