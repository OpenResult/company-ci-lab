use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

#[test]
fn repository_verifier_accepts_container_maven_and_npm_repositories() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let test_dir = std::env::temp_dir().join(format!("company-ci-repository-{stamp}"));
    fs::create_dir_all(&test_dir).expect("temp dir should exist");

    let repositories_file = test_dir.join("repositories.json");
    fs::write(
        &repositories_file,
        r#"
[
  {"name":"container-hosted"},
  {"name":"maven-snapshots"},
  {"name":"npm-hosted"}
]
"#,
    )
    .expect("test repositories should be written");

    let output = Command::new("sh")
        .current_dir(repo_root())
        .arg("testbeds/repository/verify-repositories.sh")
        .arg(repositories_file.to_str().unwrap())
        .args(["container-hosted", "maven-snapshots", "npm-hosted"])
        .output()
        .expect("repository verifier should run");

    fs::remove_dir_all(&test_dir).ok();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("verified repositories"));
}
