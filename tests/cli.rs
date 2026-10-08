use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tbeep"))
        .args(args)
        .output()
        .expect("run tbeep")
}

#[test]
fn help_and_version_work_without_a_terminal() {
    let help = run(&["--help"]);
    assert!(help.status.success());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("--plain"));
    assert!(text.contains("--verbose"));
    let version = run(&["--version"]);
    assert!(version.status.success());
    assert!(String::from_utf8_lossy(&version.stdout).contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn plain_mode_and_redirected_output_require_valid_durations() {
    for args in [vec![], vec!["--plain"], vec!["-v"]] {
        let result = run(&args);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("duration is required"));
        assert!(!result.stdout.contains(&0x1b));
    }
    for invalid in ["0", "nonsense", "18446744073709551615h"] {
        let result = run(&["--plain", invalid]);
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
    }
}
