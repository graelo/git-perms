# `git-perms`

[![crate](https://img.shields.io/crates/v/git-perms.svg)](https://crates.io/crates/git-perms)
[![documentation](https://docs.rs/git-perms/badge.svg)](https://docs.rs/git-perms)
[![minimum rustc 1.95](https://img.shields.io/badge/rustc-1.95+-red.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![rust 2024 edition](https://img.shields.io/badge/edition-2024-blue.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
[![build status](https://github.com/graelo/git-perms/actions/workflows/essentials.yml/badge.svg)](https://github.com/graelo/git-perms/actions/workflows/essentials.yml)

Store and restore Unix file permissions across git operations.

Version requirement: _rustc 1.95+_

:warning: **alpha** maturity level: **don't use this yet**

## Why

Git only tracks the executable bit. If your project relies on specific file
permissions (e.g. `0600` for secrets, `0755` for scripts), those modes are lost
after clone, checkout, merge, or rebase. `git-perms` solves this by recording
full `rwx` mode bits in a `.gitperms` file committed to the repo -- permissions
travel with the code, per branch, just like everything else.

## Usage

Once installed, `git-perms` is invoked as a git subcommand:

```console
git perms save                        # scan tracked files -> write .gitperms
git perms restore                     # read .gitperms -> apply modes
git perms diff [--quiet]              # show/detect mode differences
git perms hook install                # install git hooks
git perms hook uninstall              # remove git hooks
git perms generate-completion <shell> # shell completions (bash, zsh, fish)
```

## Getting started

```console
cd my-repo
git perms hook install     # set up auto-save/restore hooks
git perms save             # record current permissions
git add .gitperms && git commit -m "chore: track file permissions"
```

From now on, permissions are automatically saved on commit and restored on
checkout, merge, or rebase.

## The `.gitperms` file

A plain-text file, one entry per line -- the octal mode followed by the path:

```text
0644 src/lib.rs
0755 scripts/deploy.sh
```

Commit this file to your repository. Because it lives alongside your code, each
branch can carry its own set of permissions.

## Configuration

Pre-commit behavior is controlled via `git config perms.preCommit`:

| Value   | Behavior |
|---------|----------|
| `auto` (default) | Auto-save permissions and stage `.gitperms` before each commit. |
| `warn`  | Print a warning if permissions differ from `.gitperms`, but allow the commit. |
| `block` | Abort the commit if permissions differ from `.gitperms`. |

Example:

```console
git config perms.preCommit warn
```

## Installation

### From source

```console
cargo install git-perms
```

### Via Homebrew (macOS)

```console
brew install graelo/tap/git-perms
```

### From GitHub Releases

Download the appropriate binary from the
[releases page](https://github.com/graelo/git-perms/releases) and place it in
your `$PATH`.

To install shell completions:

```console
$ git perms generate-completion zsh|bash|fish > /path/to/your/completions/folder
```

## License

Licensed under either of

- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
