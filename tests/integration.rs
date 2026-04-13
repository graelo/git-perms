use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

/// Binary path resolved at compile time.
const BIN: &str = env!("CARGO_BIN_EXE_git-perms");

/// Create a temp directory with an initialised git repo and return the `TempDir`.
fn setup_repo() -> TempDir {
    let tmp = TempDir::new().expect("failed to create tempdir");
    let dir = tmp.path();

    git(dir, &["init"]);
    git(dir, &["config", "user.name", "Test"]);
    git(dir, &["config", "user.email", "test@test.com"]);

    tmp
}

/// Run a git command inside `dir`, panicking on failure.
fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .status()
        .expect("failed to launch git");
    assert!(status.success(), "git {args:?} failed");
}

/// Run `git-perms` inside `dir` with extra `args`, returning the `Output`.
fn run(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(BIN)
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .expect("failed to launch git-perms")
}

/// Helper: create a file, set its permissions, and return its path.
fn create_file(dir: &Path, relative: &str, mode: u32, content: &str) {
    let path = dir.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
}

/// Read the mode bits for a file (masked to permission bits only).
fn file_mode(path: &Path) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & 0o7777
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_save_creates_gitperms() {
    let tmp = setup_repo();
    let dir = tmp.path();

    create_file(dir, "hello.txt", 0o644, "hello\n");
    create_file(dir, "run.sh", 0o755, "#!/bin/sh\n");

    git(dir, &["add", "hello.txt", "run.sh"]);
    git(dir, &["commit", "-m", "init"]);

    let output = run(dir, &["save"]);
    assert!(output.status.success(), "save failed: {output:?}");

    let perms_path = dir.join(".gitperms");
    assert!(perms_path.exists(), ".gitperms should exist after save");

    let content = fs::read_to_string(&perms_path).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    // Sorted by path: hello.txt < run.sh
    assert_eq!(lines.len(), 2);
    assert!(lines[0].ends_with(" hello.txt"), "first line: {}", lines[0]);
    assert!(lines[1].ends_with(" run.sh"), "second line: {}", lines[1]);
    assert!(lines[0].starts_with("0644"), "hello.txt should be 0644");
    assert!(lines[1].starts_with("0755"), "run.sh should be 0755");
}

#[test]
fn test_save_prunes_deleted_files() {
    let tmp = setup_repo();
    let dir = tmp.path();

    create_file(dir, "keep.txt", 0o644, "keep\n");
    create_file(dir, "gone.txt", 0o644, "gone\n");

    git(dir, &["add", "keep.txt", "gone.txt"]);
    git(dir, &["commit", "-m", "init"]);

    // Remove the file from the working tree and the index.
    git(dir, &["rm", "gone.txt"]);

    let output = run(dir, &["save"]);
    assert!(output.status.success(), "save failed: {output:?}");

    let content = fs::read_to_string(dir.join(".gitperms")).unwrap();
    assert!(content.contains("keep.txt"), "should contain keep.txt");
    assert!(!content.contains("gone.txt"), "should not contain gone.txt");
}

#[test]
fn test_restore_applies_permissions() {
    let tmp = setup_repo();
    let dir = tmp.path();

    create_file(dir, "a.txt", 0o644, "a\n");
    create_file(dir, "b.sh", 0o755, "b\n");

    git(dir, &["add", "a.txt", "b.sh"]);
    git(dir, &["commit", "-m", "init"]);

    // Save the original permissions.
    let output = run(dir, &["save"]);
    assert!(output.status.success());

    // Mess up the permissions.
    fs::set_permissions(dir.join("a.txt"), fs::Permissions::from_mode(0o600)).unwrap();
    fs::set_permissions(dir.join("b.sh"), fs::Permissions::from_mode(0o644)).unwrap();

    assert_eq!(file_mode(&dir.join("a.txt")), 0o600);
    assert_eq!(file_mode(&dir.join("b.sh")), 0o644);

    // Restore.
    let output = run(dir, &["restore"]);
    assert!(output.status.success(), "restore failed: {output:?}");

    assert_eq!(file_mode(&dir.join("a.txt")), 0o644);
    assert_eq!(file_mode(&dir.join("b.sh")), 0o755);
}

#[test]
fn test_diff_detects_changes() {
    let tmp = setup_repo();
    let dir = tmp.path();

    create_file(dir, "file.txt", 0o644, "content\n");

    git(dir, &["add", "file.txt"]);
    git(dir, &["commit", "-m", "init"]);

    let output = run(dir, &["save"]);
    assert!(output.status.success());

    // Change mode.
    fs::set_permissions(dir.join("file.txt"), fs::Permissions::from_mode(0o755)).unwrap();

    let output = run(dir, &["diff"]);
    assert!(!output.status.success(), "diff should exit 1 when dirty");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("M"), "diff output should contain 'M'");
}

#[test]
fn test_diff_quiet_no_output() {
    let tmp = setup_repo();
    let dir = tmp.path();

    create_file(dir, "file.txt", 0o644, "content\n");

    git(dir, &["add", "file.txt"]);
    git(dir, &["commit", "-m", "init"]);

    let output = run(dir, &["save"]);
    assert!(output.status.success());

    // Change mode.
    fs::set_permissions(dir.join("file.txt"), fs::Permissions::from_mode(0o755)).unwrap();

    let output = run(dir, &["diff", "--quiet"]);
    assert!(!output.status.success(), "diff --quiet should exit 1");
    assert!(
        output.stdout.is_empty(),
        "diff --quiet should produce no stdout"
    );
}

#[test]
fn test_diff_clean() {
    let tmp = setup_repo();
    let dir = tmp.path();

    create_file(dir, "file.txt", 0o644, "content\n");

    git(dir, &["add", "file.txt"]);
    git(dir, &["commit", "-m", "init"]);

    let output = run(dir, &["save"]);
    assert!(output.status.success());

    let output = run(dir, &["diff"]);
    assert!(output.status.success(), "diff should exit 0 when clean");
}

#[test]
fn test_hook_install() {
    let tmp = setup_repo();
    let dir = tmp.path();

    let output = run(dir, &["hook", "install"]);
    assert!(output.status.success(), "hook install failed: {output:?}");

    let hooks_dir = dir.join(".git/hooks");
    let expected = ["post-checkout", "post-merge", "post-rewrite", "pre-commit"];

    for name in &expected {
        let path = hooks_dir.join(name);
        assert!(path.exists(), "hook {name} should exist");

        let mode = file_mode(&path);
        assert_eq!(mode & 0o111, 0o111, "hook {name} should be executable");

        let content = fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("# Installed by git-perms"),
            "hook {name} should contain the marker"
        );
    }
}

#[test]
fn test_hook_uninstall() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Install first.
    let output = run(dir, &["hook", "install"]);
    assert!(output.status.success());

    // Uninstall.
    let output = run(dir, &["hook", "uninstall"]);
    assert!(output.status.success(), "hook uninstall failed: {output:?}");

    let hooks_dir = dir.join(".git/hooks");
    let expected = ["post-checkout", "post-merge", "post-rewrite", "pre-commit"];

    for name in &expected {
        let path = hooks_dir.join(name);
        assert!(!path.exists(), "hook {name} should be removed");
    }
}

#[test]
fn test_hook_install_preserves_existing() {
    let tmp = setup_repo();
    let dir = tmp.path();

    let hooks_dir = dir.join(".git/hooks");
    fs::create_dir_all(&hooks_dir).unwrap();

    // Create a custom hook WITHOUT the marker.
    let custom_hook = hooks_dir.join("post-checkout");
    let custom_content = "#!/bin/sh\necho custom\n";
    fs::write(&custom_hook, custom_content).unwrap();
    fs::set_permissions(&custom_hook, fs::Permissions::from_mode(0o755)).unwrap();

    // Install should fail because a foreign hook exists.
    let output = run(dir, &["hook", "install"]);
    assert!(
        !output.status.success(),
        "hook install should fail when a foreign hook exists"
    );

    // The custom hook must be untouched.
    let after = fs::read_to_string(&custom_hook).unwrap();
    assert_eq!(after, custom_content, "custom hook must not be overwritten");
}
