use crate::image_config::ApplicationImage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationLayout {
    pub name: &'static str,
    pub project_dir: &'static str,
    pub manifest_path: &'static str,
    pub containerfile_path: &'static str,
    pub image: ApplicationImage,
    pub deployment_name: &'static str,
    pub container_name: &'static str,
    pub route_path: &'static str,
    pub route_expected_text: &'static str,
    pub image_override_env: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryLayout {
    pub name: &'static str,
    pub project_dir: &'static str,
    pub manifest_path: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoLayout {
    pub next_web: ApplicationLayout,
    pub spring_api: ApplicationLayout,
    pub node_lib: LibraryLayout,
    pub java_lib: LibraryLayout,
    pub deploy_root: &'static str,
    pub docs_root: &'static str,
    pub scripts_root: &'static str,
    pub testbeds_root: &'static str,
    pub tooling_root: &'static str,
    pub workflows_root: &'static str,
    pub issue_template_root: &'static str,
    pub pull_request_template: &'static str,
    pub readme_path: &'static str,
    pub cargo_toml_path: &'static str,
    pub cargo_lock_path: &'static str,
    pub next_web_kustomization_path: &'static str,
    pub openshift_overlay_path: &'static str,
    pub repository_compose_file_path: &'static str,
    pub repository_bootstrap_helper_path: &'static str,
    pub repository_runtime_dir: &'static str,
    pub maven_publish_helper_path: &'static str,
    pub npm_publish_helper_path: &'static str,
    pub container_registry_login_helper_path: &'static str,
    pub openshift_pull_secret_helper_path: &'static str,
    pub openshift_route_check_helper_path: &'static str,
}

impl RepoLayout {
    pub fn company_ci_lab() -> Self {
        Self {
            next_web: ApplicationLayout {
                name: "next-web",
                project_dir: "apps/next-web",
                manifest_path: "apps/next-web/package.json",
                containerfile_path: "apps/next-web/Containerfile",
                image: ApplicationImage::NextWeb,
                deployment_name: "next-web",
                container_name: "next-web",
                route_path: "/",
                route_expected_text: "company-ci next-web",
                image_override_env: "NEXT_WEB_IMAGE_REF",
            },
            spring_api: ApplicationLayout {
                name: "spring-api",
                project_dir: "apps/spring-api",
                manifest_path: "apps/spring-api/pom.xml",
                containerfile_path: "apps/spring-api/Containerfile",
                image: ApplicationImage::SpringApi,
                deployment_name: "spring-api",
                container_name: "spring-api",
                route_path: "/api/health",
                route_expected_text: "ok",
                image_override_env: "SPRING_API_IMAGE_REF",
            },
            node_lib: LibraryLayout {
                name: "node-lib",
                project_dir: "libs/node-lib",
                manifest_path: "libs/node-lib/package.json",
            },
            java_lib: LibraryLayout {
                name: "java-lib",
                project_dir: "libs/java-lib",
                manifest_path: "libs/java-lib/pom.xml",
            },
            deploy_root: "deploy/",
            docs_root: "docs/",
            scripts_root: "scripts/",
            testbeds_root: "testbeds/",
            tooling_root: "tools/company-ci/",
            workflows_root: ".github/workflows/",
            issue_template_root: ".github/ISSUE_TEMPLATE/",
            pull_request_template: ".github/pull_request_template.md",
            readme_path: "README.md",
            cargo_toml_path: "Cargo.toml",
            cargo_lock_path: "Cargo.lock",
            next_web_kustomization_path: "deploy/base/next-web/kustomization.yaml",
            openshift_overlay_path: "deploy/openshift/overlays/dev",
            repository_compose_file_path: "testbeds/repository/compose.yaml",
            repository_bootstrap_helper_path: "testbeds/repository/bootstrap.sh",
            repository_runtime_dir: "testbeds/repository/.runtime",
            maven_publish_helper_path: "testbeds/repository/maven-deploy.sh",
            npm_publish_helper_path: "testbeds/repository/npm-publish.sh",
            container_registry_login_helper_path: "testbeds/lib/container-registry-login.sh",
            openshift_pull_secret_helper_path: "testbeds/openshift/apply-registry-pull-secret.sh",
            openshift_route_check_helper_path: "testbeds/openshift/check-route.sh",
        }
    }
}
