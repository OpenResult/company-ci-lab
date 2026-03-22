use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

#[test]
fn verify_dry_run_prints_planned_steps() {
    let output = Command::new(env!("CARGO_BIN_EXE_company-ci"))
        .args(["verify", "--dry-run"])
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[dry-run] verify required tool: java"));
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

#[test]
fn publish_maven_dry_run_prints_contract_path_and_destination() {
    let project_dir = repo_root().join("libs/java-lib");
    let output = Command::new(env!("CARGO_BIN_EXE_company-ci"))
        .args([
            "publish",
            "maven-lib",
            project_dir.to_str().unwrap(),
            "--dry-run",
        ])
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[dry-run] publish contract: maven-lib"));
    assert!(stdout.contains(&format!(
        "[dry-run] publish path: {}",
        project_dir.display()
    )));
    assert!(stdout
        .contains("[dry-run] maven deploy url: http://localhost:8081/repository/maven-snapshots/"));
    assert!(stdout.contains("[dry-run] verify required tool: mvn"));
}

#[test]
fn publish_npm_dry_run_prints_registry_and_tag() {
    let project_dir = repo_root().join("libs/node-lib");
    let output = Command::new(env!("CARGO_BIN_EXE_company-ci"))
        .args([
            "publish",
            "npm-lib",
            project_dir.to_str().unwrap(),
            "--tag",
            "beta",
            "--dry-run",
        ])
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[dry-run] publish contract: npm-lib"));
    assert!(
        stdout.contains("[dry-run] npm registry url: http://localhost:8081/repository/npm-hosted/")
    );
    assert!(stdout.contains("[dry-run] npm dist-tag: beta"));
    assert!(stdout.contains("[dry-run] build npm-lib"));
}

#[test]
fn publish_rejects_contract_and_path_mismatch() {
    let project_dir = repo_root().join("libs/node-lib");
    let output = Command::new(env!("CARGO_BIN_EXE_company-ci"))
        .args(["publish", "maven-lib", project_dir.to_str().unwrap()])
        .output()
        .expect("binary should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(&format!(
        "invalid publish target: maven-lib requires {}/pom.xml",
        project_dir.display()
    )));
}

#[test]
fn deploy_openshift_dry_run_includes_pull_secret_and_route_checks() {
    let output = Command::new(env!("CARGO_BIN_EXE_company-ci"))
        .args(["deploy", "openshift", "--dry-run"])
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[dry-run] verify required tool: oc"));
    assert!(stdout.contains("[dry-run] verify required tool: curl"));
    assert!(stdout.contains("[dry-run] apply registry pull secret"));
    assert!(stdout.contains("host.crc.testing:5002/company-ci/next-web:dev"));
    assert!(
        stdout.contains("testbeds/openshift-local/check-route.sh next-web / company-ci next-web")
    );
}

#[test]
fn e2e_openshift_local_dry_run_uses_resolved_registry_contract() {
    let output = Command::new(env!("CARGO_BIN_EXE_company-ci"))
        .env("COMPANY_CI_IMAGE_TAG", "qa")
        .args(["e2e", "openshift-local", "--dry-run"])
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[dry-run] openshift-local image tag: qa"));
    assert!(stdout.contains(
        "COMPANY_CI_IMAGE_PUSH_REGISTRY=${COMPANY_CI_IMAGE_PUSH_REGISTRY:-'localhost:5002'}"
    ));
    assert!(stdout.contains(" image publish"));
    assert!(stdout.contains(" deploy openshift"));
    assert!(!stdout.contains("cargo run -p company-ci"));
    assert!(!stdout.contains("[dry-run] verify required tool: cargo"));
}

#[test]
fn e2e_emulated_dry_run_invokes_company_ci_without_cargo() {
    let output = Command::new(env!("CARGO_BIN_EXE_company-ci"))
        .args(["e2e", "emulated", "--dry-run"])
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(" publish npm-lib libs/node-lib --tag ci"));
    assert!(stdout.contains(" deploy kubernetes"));
    assert!(!stdout.contains("cargo run -p company-ci"));
    assert!(!stdout.contains("[dry-run] verify required tool: cargo"));
}
