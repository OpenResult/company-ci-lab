use std::process::Command;

#[test]
fn verify_dry_run_prints_planned_steps() {
    let output = Command::new(env!("CARGO_BIN_EXE_company-ci"))
        .args(["verify", "--dry-run"])
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[dry-run] run next-web quality checks"));
}

#[test]
fn verify_can_noop_when_only_docs_changed() {
    let output = Command::new(env!("CARGO_BIN_EXE_company-ci"))
        .env("COMPANY_CI_CHANGED_FILES", "docs/architecture.md")
        .args(["build", "--dry-run"])
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("no impacted component work detected"));
}
