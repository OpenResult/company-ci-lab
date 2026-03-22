use crate::container_engine::ContainerEngine;
use crate::context::ExecutionContext;
use crate::error::CompanyCiError;
use crate::impact::Area;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub description: String,
    pub command: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub name: String,
    pub required_tools: Vec<String>,
    pub dry_run_notes: Vec<String>,
    pub steps: Vec<Step>,
}

impl Plan {
    pub fn new(name: impl Into<String>, steps: Vec<Step>) -> Self {
        Self {
            name: name.into(),
            required_tools: Vec::new(),
            dry_run_notes: Vec::new(),
            steps,
        }
    }

    pub fn with_required_tools<I, S>(mut self, tools: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for tool in tools {
            let tool = tool.into();
            if !self.required_tools.contains(&tool) {
                self.required_tools.push(tool);
            }
        }
        self
    }

    pub fn with_dry_run_notes<I, S>(mut self, notes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.dry_run_notes.extend(notes.into_iter().map(Into::into));
        self
    }
}

pub fn verify_plan(context: &ExecutionContext) -> Plan {
    let mut steps = vec![
        step(
            "validate deployment manifests exist",
            ["test", "-f", "deploy/base/next-web/kustomization.yaml"],
        ),
        step(
            "validate spring api containerfile exists",
            ["test", "-f", "apps/spring-api/Containerfile"],
        ),
    ];
    steps.extend(component_steps(context, Mode::Verify));
    Plan::new("verify", steps).with_required_tools(component_required_tools(context, Mode::Verify))
}

pub fn build_plan(context: &ExecutionContext) -> Plan {
    Plan::new("build", component_steps(context, Mode::Build))
        .with_required_tools(component_required_tools(context, Mode::Build))
}

pub fn test_plan(context: &ExecutionContext) -> Plan {
    Plan::new("test", component_steps(context, Mode::Test))
        .with_required_tools(component_required_tools(context, Mode::Test))
}

pub fn package_plan(context: &ExecutionContext) -> Plan {
    Plan::new("package", component_steps(context, Mode::Package))
        .with_required_tools(component_required_tools(context, Mode::Package))
}

pub fn publish_maven_lib_plan(project_dir: &str) -> Result<Plan, CompanyCiError> {
    let resolved = resolve_publish_project(project_dir, "pom.xml", "maven-lib")?;
    let deploy_url = env::var("MAVEN_DEPLOY_URL")
        .unwrap_or_else(|_| "http://localhost:8081/repository/maven-snapshots/".to_string());
    let server_id = env::var("MAVEN_SERVER_ID").unwrap_or_else(|_| "local".to_string());

    Ok(Plan::new(
        "publish-maven-lib",
        vec![owned_step(
            format!("publish maven-lib from {}", resolved.project_dir_display),
            vec![
                "sh".to_string(),
                "testbeds/repo/nexus/maven-deploy.sh".to_string(),
                resolved.manifest_path_display.clone(),
            ],
        )],
    )
    .with_required_tools(["java", "mvn"])
    .with_dry_run_notes([
        "publish contract: maven-lib".to_string(),
        format!("publish path: {}", resolved.project_dir_display),
        format!("maven deploy url: {deploy_url}"),
        format!("maven server id: {server_id}"),
    ]))
}

pub fn publish_npm_lib_plan(project_dir: &str, tag: &str) -> Result<Plan, CompanyCiError> {
    validate_npm_tag(tag)?;
    let resolved = resolve_publish_project(project_dir, "package.json", "npm-lib")?;
    let registry_url = env::var("NPM_REGISTRY_URL")
        .unwrap_or_else(|_| "http://localhost:8081/repository/npm-hosted/".to_string());

    Ok(Plan::new(
        "publish-npm-lib",
        vec![
            owned_step(
                format!("build npm-lib at {}", resolved.project_dir_display),
                vec![
                    "sh".to_string(),
                    "-c".to_string(),
                    "cd \"$1\" && npm run build".to_string(),
                    "sh".to_string(),
                    resolved.project_dir_display.clone(),
                ],
            ),
            owned_step(
                format!("publish npm-lib from {}", resolved.project_dir_display),
                vec![
                    "sh".to_string(),
                    "testbeds/repo/nexus/npm-publish.sh".to_string(),
                    resolved.project_dir_display.clone(),
                    tag.to_string(),
                ],
            ),
        ],
    )
    .with_required_tools(["node", "npm"])
    .with_dry_run_notes([
        "publish contract: npm-lib".to_string(),
        format!("publish path: {}", resolved.project_dir_display),
        format!("npm registry url: {registry_url}"),
        format!("npm dist-tag: {tag}"),
    ]))
}

pub fn image_build_plan(context: &ExecutionContext) -> Plan {
    let mut steps = Vec::new();
    let mut required_tools = Vec::new();
    let engine = context.container_engine;
    if context.affects(Area::NextWeb) {
        steps.push(step(
            "build next-web image inputs",
            ["sh", "-c", "cd apps/next-web && npm run build"],
        ));
        steps.push(shell_step(
            "build next-web image",
            &format!(
                "image_ref=${{NEXT_WEB_IMAGE_REF:-localhost:5001/next-web:dev}} && {} build -f apps/next-web/Containerfile -t ${{image_ref}} apps/next-web",
                engine.binary()
            ),
        ));
        push_tool(&mut required_tools, "node");
        push_tool(&mut required_tools, "npm");
        push_tool(&mut required_tools, engine.binary());
    }
    if context.affects(Area::SpringApi) {
        steps.push(step(
            "package spring-api image inputs",
            [
                "mvn",
                "-B",
                "-ntp",
                "-f",
                "apps/spring-api/pom.xml",
                "-DskipTests",
                "package",
            ],
        ));
        steps.push(shell_step(
            "build spring-api image",
            &format!(
                "image_ref=${{SPRING_API_IMAGE_REF:-localhost:5001/spring-api:dev}} && {} build -f apps/spring-api/Containerfile -t ${{image_ref}} apps/spring-api",
                engine.binary()
            ),
        ));
        push_tool(&mut required_tools, "java");
        push_tool(&mut required_tools, "mvn");
        push_tool(&mut required_tools, engine.binary());
    }
    if steps.is_empty() {
        steps.push(noop_step("no impacted application images detected"));
    }
    Plan::new("image-build", steps).with_required_tools(required_tools)
}

pub fn image_publish_plan(context: &ExecutionContext) -> Plan {
    let mut steps = Vec::new();
    let mut required_tools = Vec::new();
    let engine = context.container_engine;
    if context.affects(Area::NextWeb) {
        steps.push(shell_step(
            "push next-web image",
            &image_push_command(engine, "NEXT_WEB_IMAGE_REF", "localhost:5001/next-web:dev"),
        ));
        push_tool(&mut required_tools, engine.binary());
    }
    if context.affects(Area::SpringApi) {
        steps.push(shell_step(
            "push spring-api image",
            &image_push_command(
                engine,
                "SPRING_API_IMAGE_REF",
                "localhost:5001/spring-api:dev",
            ),
        ));
        push_tool(&mut required_tools, engine.binary());
    }
    if steps.is_empty() {
        steps.push(noop_step("no impacted application images detected"));
    }
    Plan::new("image-publish", steps).with_required_tools(required_tools)
}

pub fn deploy_kubernetes_plan(_context: &ExecutionContext) -> Plan {
    Plan::new(
        "deploy-kubernetes",
        vec![
            step(
                "apply kind overlay",
                ["kubectl", "apply", "-k", "deploy/kind/overlays/ci"],
            ),
            step(
                "set next-web image",
                [
                    "sh",
                    "-c",
                    "image_ref=${NEXT_WEB_IMAGE_REF:-localhost:5001/next-web:dev} && kubectl set image deployment/next-web next-web=${image_ref}",
                ],
            ),
            step(
                "set spring-api image",
                [
                    "sh",
                    "-c",
                    "image_ref=${SPRING_API_IMAGE_REF:-localhost:5001/spring-api:dev} && kubectl set image deployment/spring-api spring-api=${image_ref}",
                ],
            ),
            step(
                "verify next-web rollout",
                ["kubectl", "rollout", "status", "deployment/next-web"],
            ),
            step(
                "verify spring-api rollout",
                ["kubectl", "rollout", "status", "deployment/spring-api"],
            ),
            step(
                "check next-web homepage",
                [
                    "sh",
                    "testbeds/kind/check-service.sh",
                    "next-web",
                    "18080",
                    "80",
                    "/",
                    "company-ci next-web",
                ],
            ),
            step(
                "check spring-api health endpoint",
                [
                    "sh",
                    "testbeds/kind/check-service.sh",
                    "spring-api",
                    "18081",
                    "80",
                    "/api/health",
                    "ok",
                ],
            ),
        ],
    )
    .with_required_tools(["kubectl", "curl"])
}

pub fn deploy_openshift_plan(_context: &ExecutionContext) -> Plan {
    Plan::new(
        "deploy-openshift",
        vec![
            step(
                "apply openshift dev overlay",
                ["oc", "apply", "-k", "deploy/openshift/overlays/dev"],
            ),
            step(
                "set next-web image",
                [
                    "sh",
                    "-c",
                    "image_ref=${NEXT_WEB_IMAGE_REF:-localhost:5001/next-web:dev} && oc set image deployment/next-web next-web=${image_ref}",
                ],
            ),
            step(
                "set spring-api image",
                [
                    "sh",
                    "-c",
                    "image_ref=${SPRING_API_IMAGE_REF:-localhost:5001/spring-api:dev} && oc set image deployment/spring-api spring-api=${image_ref}",
                ],
            ),
            step(
                "verify next-web rollout",
                ["oc", "rollout", "status", "deployment/next-web"],
            ),
            step(
                "verify spring-api rollout",
                ["oc", "rollout", "status", "deployment/spring-api"],
            ),
        ],
    )
    .with_required_tools(["oc"])
}

pub fn env_up_kind_plan(context: &ExecutionContext) -> Plan {
    let create_cluster_command = kind_command(
        context.container_engine,
        "create cluster --config testbeds/kind/cluster-config.yaml",
    );
    Plan::new(
        "env-up-kind",
        vec![
            shell_step("create kind cluster", &create_cluster_command),
            step(
                "start local registry helper",
                ["sh", "testbeds/kind/registry.sh", "up"],
            ),
        ],
    )
    .with_required_tools(["kind", context.container_engine.binary(), "kubectl"])
}

pub fn env_down_kind_plan(context: &ExecutionContext) -> Plan {
    let delete_cluster_command = format!(
        "{} || true",
        kind_command(context.container_engine, "delete cluster")
    );
    Plan::new(
        "env-down-kind",
        vec![
            shell_step("delete kind cluster", &delete_cluster_command),
            step(
                "stop local registry helper",
                ["sh", "testbeds/kind/registry.sh", "down"],
            ),
        ],
    )
    .with_required_tools(["kind", context.container_engine.binary()])
}

pub fn env_up_nexus_plan(context: &ExecutionContext) -> Plan {
    let compose_up_command = compose_command(
        context.container_engine,
        "testbeds/repo/nexus/compose.yaml",
        "up -d",
    );
    Plan::new(
        "env-up-nexus",
        vec![
            shell_step("start nexus", &compose_up_command),
            step(
                "wait for nexus and capture runtime state",
                ["sh", "testbeds/repo/nexus/bootstrap.sh"],
            ),
        ],
    )
    .with_required_tools([context.container_engine.binary(), "curl"])
}

pub fn env_down_nexus_plan(context: &ExecutionContext) -> Plan {
    let compose_down_command = compose_command(
        context.container_engine,
        "testbeds/repo/nexus/compose.yaml",
        "down -v",
    );
    Plan::new(
        "env-down-nexus",
        vec![
            shell_step("stop nexus", &compose_down_command),
            step(
                "remove nexus runtime state",
                ["sh", "-c", "rm -rf testbeds/repo/nexus/.runtime"],
            ),
        ],
    )
    .with_required_tools([context.container_engine.binary()])
}

pub fn e2e_emulated_plan(context: &ExecutionContext) -> Plan {
    Plan::new(
        "e2e-emulated",
        vec![
            step(
                "start nexus",
                [
                    "cargo",
                    "run",
                    "-p",
                    "company-ci",
                    "--",
                    "env",
                    "up",
                    "nexus",
                ],
            ),
            step(
                "create kind cluster",
                [
                    "cargo",
                    "run",
                    "-p",
                    "company-ci",
                    "--",
                    "env",
                    "up",
                    "kind",
                ],
            ),
            step(
                "verify all components",
                ["cargo", "run", "-p", "company-ci", "--", "verify"],
            ),
            step(
                "package artifacts",
                ["cargo", "run", "-p", "company-ci", "--", "package"],
            ),
            step(
                "publish node-lib",
                [
                    "cargo",
                    "run",
                    "-p",
                    "company-ci",
                    "--",
                    "publish",
                    "npm-lib",
                    "libs/node-lib",
                    "--tag",
                    "ci",
                ],
            ),
            step(
                "publish java-lib",
                [
                    "cargo",
                    "run",
                    "-p",
                    "company-ci",
                    "--",
                    "publish",
                    "maven-lib",
                    "libs/java-lib",
                ],
            ),
            step(
                "build images",
                ["cargo", "run", "-p", "company-ci", "--", "image", "build"],
            ),
            step(
                "publish images",
                ["cargo", "run", "-p", "company-ci", "--", "image", "publish"],
            ),
            step(
                "deploy to kind",
                [
                    "cargo",
                    "run",
                    "-p",
                    "company-ci",
                    "--",
                    "deploy",
                    "kubernetes",
                ],
            ),
            step(
                "tear down kind",
                [
                    "cargo",
                    "run",
                    "-p",
                    "company-ci",
                    "--",
                    "env",
                    "down",
                    "kind",
                ],
            ),
            step(
                "tear down nexus",
                [
                    "cargo",
                    "run",
                    "-p",
                    "company-ci",
                    "--",
                    "env",
                    "down",
                    "nexus",
                ],
            ),
        ],
    )
    .with_required_tools([
        "cargo",
        "curl",
        context.container_engine.binary(),
        "kind",
        "kubectl",
        "java",
        "mvn",
        "node",
        "npm",
    ])
}

pub fn e2e_openshift_local_plan(context: &ExecutionContext) -> Plan {
    Plan::new(
        "e2e-openshift-local",
        vec![
            step(
                "assume OpenShift Local login",
                ["sh", "testbeds/openshift-local/scripts/login.sh"],
            ),
            step(
                "verify all components",
                ["cargo", "run", "-p", "company-ci", "--", "verify"],
            ),
            step(
                "build images",
                ["cargo", "run", "-p", "company-ci", "--", "image", "build"],
            ),
            step(
                "deploy openshift overlays",
                [
                    "cargo",
                    "run",
                    "-p",
                    "company-ci",
                    "--",
                    "deploy",
                    "openshift",
                ],
            ),
        ],
    )
    .with_required_tools([
        "cargo",
        "oc",
        "java",
        "mvn",
        "node",
        "npm",
        context.container_engine.binary(),
    ])
}

#[derive(Clone, Copy)]
enum Mode {
    Verify,
    Build,
    Test,
    Package,
}

fn component_steps(context: &ExecutionContext, mode: Mode) -> Vec<Step> {
    let mut steps = Vec::new();

    if context.affects(Area::NextWeb) {
        steps.extend(match mode {
            Mode::Verify => vec![step(
                "run next-web quality checks",
                [
                    "sh",
                    "-c",
                    "cd apps/next-web && npm run lint && npm test && npm run build",
                ],
            )],
            Mode::Build => vec![step(
                "build next-web",
                ["sh", "-c", "cd apps/next-web && npm run build"],
            )],
            Mode::Test => vec![step(
                "test next-web",
                ["sh", "-c", "cd apps/next-web && npm test"],
            )],
            Mode::Package => vec![noop_step(
                "next-web packaging is handled through image commands",
            )],
        });
    }

    if context.affects(Area::SpringApi) {
        steps.extend(match mode {
            Mode::Verify => vec![step(
                "verify spring-api",
                [
                    "mvn",
                    "-B",
                    "-ntp",
                    "-f",
                    "apps/spring-api/pom.xml",
                    "verify",
                ],
            )],
            Mode::Build => vec![step(
                "build spring-api",
                [
                    "mvn",
                    "-B",
                    "-ntp",
                    "-f",
                    "apps/spring-api/pom.xml",
                    "-DskipTests",
                    "compile",
                ],
            )],
            Mode::Test => vec![step(
                "test spring-api",
                ["mvn", "-B", "-ntp", "-f", "apps/spring-api/pom.xml", "test"],
            )],
            Mode::Package => vec![step(
                "package spring-api",
                [
                    "mvn",
                    "-B",
                    "-ntp",
                    "-f",
                    "apps/spring-api/pom.xml",
                    "-DskipTests",
                    "package",
                ],
            )],
        });
    }

    if context.affects(Area::NodeLib) {
        steps.extend(match mode {
            Mode::Verify => vec![step("run node-lib checks", ["sh", "-c", "cd libs/node-lib && npm run lint && npm run typecheck && npm run build && npm test && npm run package"])],
            Mode::Build => vec![step("build node-lib", ["sh", "-c", "cd libs/node-lib && npm run lint && npm run typecheck && npm run build"])],
            Mode::Test => vec![step("test node-lib", ["sh", "-c", "cd libs/node-lib && npm run build && npm test"])],
            Mode::Package => vec![step("package node-lib", ["sh", "-c", "mkdir -p target/node-packages && cd libs/node-lib && npm run lint && npm run typecheck && npm run build && npm pack --pack-destination ../../target/node-packages"])],
        });
    }

    if context.affects(Area::JavaLib) {
        steps.extend(match mode {
            Mode::Verify => vec![step(
                "verify java-lib",
                ["mvn", "-B", "-ntp", "-f", "libs/java-lib/pom.xml", "verify"],
            )],
            Mode::Build => vec![step(
                "build java-lib",
                [
                    "mvn",
                    "-B",
                    "-ntp",
                    "-f",
                    "libs/java-lib/pom.xml",
                    "-DskipTests",
                    "compile",
                ],
            )],
            Mode::Test => vec![step(
                "test java-lib",
                ["mvn", "-B", "-ntp", "-f", "libs/java-lib/pom.xml", "test"],
            )],
            Mode::Package => vec![step(
                "package java-lib",
                [
                    "mvn",
                    "-B",
                    "-ntp",
                    "-f",
                    "libs/java-lib/pom.xml",
                    "-DskipTests",
                    "package",
                ],
            )],
        });
    }

    if steps.is_empty() {
        steps.push(noop_step("no impacted component work detected"));
    }

    steps
}

fn component_required_tools(context: &ExecutionContext, mode: Mode) -> Vec<&'static str> {
    let mut tools = Vec::new();

    if context.affects(Area::NextWeb) && matches!(mode, Mode::Verify | Mode::Build | Mode::Test) {
        add_node_tools(&mut tools);
    }

    if context.affects(Area::SpringApi)
        && matches!(
            mode,
            Mode::Verify | Mode::Build | Mode::Test | Mode::Package
        )
    {
        add_java_tools(&mut tools);
    }

    if context.affects(Area::NodeLib)
        && matches!(
            mode,
            Mode::Verify | Mode::Build | Mode::Test | Mode::Package
        )
    {
        add_node_tools(&mut tools);
    }

    if context.affects(Area::JavaLib)
        && matches!(
            mode,
            Mode::Verify | Mode::Build | Mode::Test | Mode::Package
        )
    {
        add_java_tools(&mut tools);
    }

    tools
}

struct ResolvedPublishProject {
    project_dir_display: String,
    manifest_path_display: String,
}

fn resolve_publish_project(
    project_dir: &str,
    manifest_name: &str,
    contract: &str,
) -> Result<ResolvedPublishProject, CompanyCiError> {
    let project_path = PathBuf::from(project_dir);
    if !project_path.exists() {
        return Err(CompanyCiError::InvalidArgument(format!(
            "publish project path does not exist: {project_dir}"
        )));
    }
    if !project_path.is_dir() {
        return Err(CompanyCiError::InvalidArgument(format!(
            "publish project path is not a directory: {project_dir}"
        )));
    }

    let manifest_path = project_path.join(manifest_name);
    if !manifest_path.is_file() {
        return Err(CompanyCiError::InvalidArgument(format!(
            "invalid publish target: {contract} requires {}/{}",
            project_path.display(),
            manifest_name
        )));
    }

    Ok(ResolvedPublishProject {
        project_dir_display: display_path(&project_path),
        manifest_path_display: display_path(&manifest_path),
    })
}

fn validate_npm_tag(tag: &str) -> Result<(), CompanyCiError> {
    if tag.trim().is_empty() {
        return Err(CompanyCiError::InvalidArgument(
            "npm publish tag must not be empty".to_string(),
        ));
    }
    if tag.chars().any(char::is_whitespace) {
        return Err(CompanyCiError::InvalidArgument(
            "npm publish tag must not contain whitespace".to_string(),
        ));
    }
    Ok(())
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn owned_step(description: impl Into<String>, command: Vec<String>) -> Step {
    Step {
        description: description.into(),
        command,
    }
}

fn noop_step(description: &str) -> Step {
    step(description, ["true"])
}

fn shell_step(description: &str, command: &str) -> Step {
    step(description, ["sh", "-c", command])
}

fn kind_command(engine: ContainerEngine, operation: &str) -> String {
    match engine.kind_provider_env() {
        Some(provider_env) => format!("{provider_env} kind {operation}"),
        None => format!("kind {operation}"),
    }
}

fn compose_command(engine: ContainerEngine, compose_file: &str, operation: &str) -> String {
    format!(
        "{} compose -f {} {}",
        engine.binary(),
        compose_file,
        operation
    )
}

fn image_push_command(engine: ContainerEngine, image_env: &str, default_image_ref: &str) -> String {
    let prefix = format!("image_ref=${{{image_env}:-{default_image_ref}}} && ");
    match engine {
        ContainerEngine::Docker => format!("{prefix}docker push ${{image_ref}}"),
        ContainerEngine::Podman => format!(
            "{prefix}podman push --tls-verify=${{COMPANY_CI_IMAGE_TLS_VERIFY:-false}} ${{image_ref}}"
        ),
    }
}

fn add_java_tools(tools: &mut Vec<&'static str>) {
    push_tool(tools, "java");
    push_tool(tools, "mvn");
}

fn add_node_tools(tools: &mut Vec<&'static str>) {
    push_tool(tools, "node");
    push_tool(tools, "npm");
}

fn push_tool(tools: &mut Vec<&'static str>, tool: &'static str) {
    if !tools.contains(&tool) {
        tools.push(tool);
    }
}

fn step<const N: usize>(description: &str, command: [&str; N]) -> Step {
    Step {
        description: description.to_string(),
        command: command.iter().map(|p| p.to_string()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn context(areas: &[Area]) -> ExecutionContext {
        ExecutionContext {
            container_engine: ContainerEngine::Docker,
            impacted_areas: areas.to_vec(),
        }
    }

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn verify_plan_contains_requested_component_checks() {
        let plan = verify_plan(&context(&[Area::NextWeb, Area::NodeLib]));
        assert_eq!(plan.name, "verify");
        assert!(plan
            .steps
            .iter()
            .any(|s| s.description.contains("next-web")));
        assert!(plan
            .steps
            .iter()
            .any(|s| s.description.contains("node-lib")));
        assert!(!plan
            .steps
            .iter()
            .any(|s| s.description.contains("java-lib")));
        assert_eq!(
            plan.required_tools,
            vec!["node".to_string(), "npm".to_string()]
        );
    }

    #[test]
    fn e2e_emulated_plan_orders_environment_before_deploy() {
        let plan = e2e_emulated_plan(&context(&[Area::Testbeds]));
        assert_eq!(plan.steps.first().unwrap().description, "start nexus");
        assert_eq!(plan.steps.last().unwrap().description, "tear down nexus");
        assert!(plan.steps.iter().any(|step| step
            .command
            .join(" ")
            .contains("publish npm-lib libs/node-lib --tag ci")));
        assert!(plan.steps.iter().any(|step| step
            .command
            .join(" ")
            .contains("publish maven-lib libs/java-lib")));
        assert!(plan.required_tools.iter().any(|tool| tool == "curl"));
        assert!(plan.required_tools.iter().any(|tool| tool == "mvn"));
        assert!(plan.required_tools.iter().any(|tool| tool == "docker"));
    }

    #[test]
    fn build_plan_noops_when_nothing_is_impacted() {
        let plan = build_plan(&ExecutionContext {
            container_engine: ContainerEngine::Docker,
            impacted_areas: vec![Area::Docs],
        });
        assert_eq!(
            plan.steps,
            vec![noop_step("no impacted component work detected")]
        );
        assert!(plan.required_tools.is_empty());
    }

    #[test]
    fn publish_maven_lib_plan_requires_java_tools_for_java_lib() {
        let project_dir = repo_root().join("libs/java-lib");
        let plan = publish_maven_lib_plan(project_dir.to_str().unwrap()).unwrap();
        assert!(plan
            .steps
            .iter()
            .any(|step| step.description.contains("publish maven-lib")));
        assert_eq!(
            plan.required_tools,
            vec!["java".to_string(), "mvn".to_string()]
        );
        assert!(plan
            .dry_run_notes
            .iter()
            .any(|note| note.contains("maven deploy url")));
    }

    #[test]
    fn publish_npm_lib_plan_requires_node_tools_for_node_lib() {
        let project_dir = repo_root().join("libs/node-lib");
        let plan = publish_npm_lib_plan(project_dir.to_str().unwrap(), "ci").unwrap();
        assert_eq!(
            plan.required_tools,
            vec!["node".to_string(), "npm".to_string()]
        );
        assert!(plan
            .steps
            .iter()
            .any(|step| step.description.contains("build npm-lib")));
        assert!(plan
            .dry_run_notes
            .iter()
            .any(|note| note.contains("npm dist-tag: ci")));
    }

    #[test]
    fn publish_maven_lib_plan_rejects_non_maven_project_directory() {
        let project_dir = repo_root().join("libs/node-lib");
        let error = publish_maven_lib_plan(project_dir.to_str().unwrap()).unwrap_err();
        assert_eq!(
            error.to_string(),
            format!(
                "invalid publish target: maven-lib requires {}/pom.xml",
                project_dir.display()
            )
        );
    }

    #[test]
    fn deploy_kubernetes_plan_checks_live_services() {
        let plan = deploy_kubernetes_plan(&context(&[Area::Deploy]));
        assert!(plan
            .steps
            .iter()
            .any(|step| step.description.contains("check next-web homepage")));
        assert!(plan.required_tools.iter().any(|tool| tool == "curl"));
    }
}
