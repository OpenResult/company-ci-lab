use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

#[test]
fn openshift_services_use_named_http_ports() {
    let next_web = fs::read_to_string(repo_root().join("deploy/base/next-web/service.yaml"))
        .expect("next-web service manifest should exist");
    let spring_api = fs::read_to_string(repo_root().join("deploy/base/spring-api/service.yaml"))
        .expect("spring-api service manifest should exist");

    assert!(next_web.contains("name: http"));
    assert!(spring_api.contains("name: http"));
}

#[test]
fn openshift_routes_target_named_http_service_ports() {
    let next_web =
        fs::read_to_string(repo_root().join("deploy/openshift/overlays/dev/next-web-route.yaml"))
            .expect("next-web route manifest should exist");
    let spring_api =
        fs::read_to_string(repo_root().join("deploy/openshift/overlays/dev/spring-api-route.yaml"))
            .expect("spring-api route manifest should exist");

    assert!(next_web.contains("targetPort: http"));
    assert!(spring_api.contains("targetPort: http"));
}
