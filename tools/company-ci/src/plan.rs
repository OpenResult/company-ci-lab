use crate::container_engine::ContainerEngine;
use crate::context::ExecutionContext;
use crate::error::CompanyCiError;
use crate::image_config::{ImageProfile, ImageSettings};
use crate::impact::Area;
use crate::openshift_config::OpenshiftConfig;
use crate::repo_layout::{ApplicationLayout, LibraryLayout, RepoLayout};
use crate::requirements::EnvRequirement;
use std::env;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub description: String,
    pub command: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub name: String,
    pub required_tools: Vec<String>,
    pub required_env: Vec<EnvRequirement>,
    pub dry_run_notes: Vec<String>,
    pub steps: Vec<Step>,
}

impl Plan {
    pub fn new(name: impl Into<String>, steps: Vec<Step>) -> Self {
        Self {
            name: name.into(),
            required_tools: Vec::new(),
            required_env: Vec::new(),
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

    pub fn with_required_env<I>(mut self, requirements: I) -> Self
    where
        I: IntoIterator<Item = EnvRequirement>,
    {
        for requirement in requirements {
            if !self.required_env.contains(&requirement) {
                self.required_env.push(requirement);
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
    let layout = &context.repo_layout;
    let mut steps = vec![
        step(
            "validate deployment manifests exist",
            ["test", "-f", layout.next_web_kustomization_path],
        ),
        step(
            "validate spring api containerfile exists",
            ["test", "-f", layout.spring_api.containerfile_path],
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

pub fn publish_maven_lib_plan(
    layout: &RepoLayout,
    project_dir: &str,
) -> Result<Plan, CompanyCiError> {
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
                layout.maven_publish_helper_path.to_string(),
                resolved.manifest_path_display.clone(),
            ],
        )],
    )
    .with_required_tools(["java", "./mvnw"])
    .with_dry_run_notes([
        "publish contract: maven-lib".to_string(),
        format!("publish path: {}", resolved.project_dir_display),
        format!("maven deploy url: {deploy_url}"),
        format!("maven server id: {server_id}"),
    ]))
}

pub fn publish_npm_lib_plan(
    layout: &RepoLayout,
    project_dir: &str,
    tag: &str,
) -> Result<Plan, CompanyCiError> {
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
                    layout.npm_publish_helper_path.to_string(),
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
    let layout = &context.repo_layout;
    let mut steps = Vec::new();
    let mut required_tools = Vec::new();
    let engine = context.container_engine;
    let image_settings = ImageSettings::from_env(ImageProfile::Local);
    let image_platform = env_var("COMPANY_CI_IMAGE_PLATFORM");

    if context.affects(Area::NextWeb) {
        steps.push(step(
            "build next-web image inputs",
            [
                "sh",
                "-c",
                &format!("cd {} && npm run build", layout.next_web.project_dir),
            ],
        ));
        steps.push(shell_step(
            "build next-web image",
            &image_build_command(
                engine,
                layout.next_web.containerfile_path,
                &image_settings.push_ref(layout.next_web.image),
                layout.next_web.project_dir,
                image_platform.as_deref(),
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
                "./mvnw",
                "-B",
                "-ntp",
                "-f",
                layout.spring_api.manifest_path,
                "-DskipTests",
                "package",
            ],
        ));
        steps.push(shell_step(
            "build spring-api image",
            &image_build_command(
                engine,
                layout.spring_api.containerfile_path,
                &image_settings.push_ref(layout.spring_api.image),
                layout.spring_api.project_dir,
                image_platform.as_deref(),
            ),
        ));
        push_tool(&mut required_tools, "java");
        push_tool(&mut required_tools, "./mvnw");
        push_tool(&mut required_tools, engine.binary());
    }

    if steps.is_empty() {
        steps.push(noop_step("no impacted application images detected"));
    }

    Plan::new("image-build", steps).with_required_tools(required_tools)
}

pub fn image_publish_plan(context: &ExecutionContext) -> Result<Plan, CompanyCiError> {
    let mut steps = Vec::new();
    let mut required_tools = Vec::new();
    let mut required_env = Vec::new();
    let engine = context.container_engine;
    let image_settings = ImageSettings::from_env(ImageProfile::Local);
    image_settings.validate_publish_contract("image-publish")?;

    if (context.affects(Area::NextWeb) || context.affects(Area::SpringApi))
        && image_settings.has_registry_auth()
    {
        steps.push(step(
            "authenticate to image registry",
            [
                "sh",
                context.repo_layout.container_registry_login_helper_path,
            ],
        ));
        push_tool(&mut required_tools, engine.binary());
        required_env.push(EnvRequirement::secret("COMPANY_CI_IMAGE_REGISTRY_USERNAME"));
        required_env.push(EnvRequirement::secret_or_file(
            "COMPANY_CI_IMAGE_REGISTRY_PASSWORD",
            "COMPANY_CI_IMAGE_REGISTRY_PASSWORD_FILE",
        ));
    }

    if context.affects(Area::NextWeb) {
        steps.push(shell_step(
            "push next-web image",
            &image_push_command(
                engine,
                &image_settings.push_ref(context.repo_layout.next_web.image),
            ),
        ));
        push_tool(&mut required_tools, engine.binary());
    }

    if context.affects(Area::SpringApi) {
        steps.push(shell_step(
            "push spring-api image",
            &image_push_command(
                engine,
                &image_settings.push_ref(context.repo_layout.spring_api.image),
            ),
        ));
        push_tool(&mut required_tools, engine.binary());
    }

    if steps.is_empty() {
        steps.push(noop_step("no impacted application images detected"));
    }

    Ok(Plan::new("image-publish", steps)
        .with_required_tools(required_tools)
        .with_required_env(required_env))
}

pub fn deploy_openshift_plan(context: &ExecutionContext) -> Result<Plan, CompanyCiError> {
    let openshift = OpenshiftConfig::from_env("deploy-openshift")?;
    let layout = &context.repo_layout;
    Ok(Plan::new(
        "deploy-openshift",
        openshift_deploy_steps(layout, &openshift),
    )
    .with_required_tools(["oc", "curl"])
    .with_required_env(openshift_deploy_requirements())
    .with_dry_run_notes([format!(
        "openshift skip tls verify: {}",
        openshift.skip_tls_verify()
    )]))
}

pub fn e2e_openshift_plan(context: &ExecutionContext) -> Result<Plan, CompanyCiError> {
    let image_settings = openshift_e2e_settings();
    let env_defaults = openshift_default_env(&image_settings);
    let openshift = OpenshiftConfig::from_env("e2e-openshift")?;
    Ok(Plan::new(
        "e2e-openshift",
        vec![
            company_ci_step(context, "verify all components", &["verify"]),
            shell_step(
                "build images",
                &command_with_default_env(
                    &env_defaults,
                    &company_ci_shell_command(context, &["image", "build"]),
                ),
            ),
            shell_step(
                "publish images",
                &command_with_default_env(
                    &env_defaults,
                    &company_ci_shell_command(context, &["image", "publish"]),
                ),
            ),
            shell_step(
                "deploy openshift overlays",
                &command_with_default_env(
                    &env_defaults,
                    &company_ci_shell_command(context, &["deploy", "openshift"]),
                ),
            ),
        ],
    )
    .with_required_tools([
        "curl",
        "oc",
        "java",
        "./mvnw",
        "node",
        "npm",
        context.container_engine.binary(),
    ])
    .with_required_env(OpenshiftConfig::auth_requirements())
    .with_dry_run_notes([
        format!("openshift image tag: {}", image_settings.tag()),
        format!(
            "openshift image platform: {}",
            env_var("COMPANY_CI_IMAGE_PLATFORM")
                .unwrap_or_else(|| default_openshift_image_platform().to_string())
        ),
        format!("openshift skip tls verify: {}", openshift.skip_tls_verify()),
    ]))
}

#[derive(Clone, Copy)]
enum Mode {
    Verify,
    Build,
    Test,
    Package,
}

fn component_steps(context: &ExecutionContext, mode: Mode) -> Vec<Step> {
    let layout = &context.repo_layout;
    let mut steps = Vec::new();

    if context.affects(Area::NextWeb) {
        steps.extend(node_component_steps(&layout.next_web, mode));
    }

    if context.affects(Area::SpringApi) {
        steps.extend(maven_component_steps(&layout.spring_api, mode));
    }

    if context.affects(Area::NodeLib) {
        steps.extend(node_library_steps(&layout.node_lib, mode));
    }

    if context.affects(Area::JavaLib) {
        steps.extend(maven_library_steps(&layout.java_lib, mode));
    }

    if steps.is_empty() {
        steps.push(noop_step("no impacted component work detected"));
    }

    steps
}

fn node_component_steps(component: &ApplicationLayout, mode: Mode) -> Vec<Step> {
    match mode {
        Mode::Verify => vec![step(
            &format!("run {} quality checks", component.name),
            [
                "sh",
                "-c",
                &format!(
                    "cd {} && npm run lint && npm test && npm run build",
                    component.project_dir
                ),
            ],
        )],
        Mode::Build => vec![step(
            &format!("build {}", component.name),
            [
                "sh",
                "-c",
                &format!("cd {} && npm run build", component.project_dir),
            ],
        )],
        Mode::Test => vec![step(
            &format!("test {}", component.name),
            [
                "sh",
                "-c",
                &format!("cd {} && npm test", component.project_dir),
            ],
        )],
        Mode::Package => vec![noop_step(
            "next-web packaging is handled through image commands",
        )],
    }
}

fn maven_component_steps(component: &ApplicationLayout, mode: Mode) -> Vec<Step> {
    match mode {
        Mode::Verify => vec![step(
            &format!("verify {}", component.name),
            [
                "./mvnw",
                "-B",
                "-ntp",
                "-f",
                component.manifest_path,
                "verify",
            ],
        )],
        Mode::Build => vec![step(
            &format!("build {}", component.name),
            [
                "./mvnw",
                "-B",
                "-ntp",
                "-f",
                component.manifest_path,
                "-DskipTests",
                "compile",
            ],
        )],
        Mode::Test => vec![step(
            &format!("test {}", component.name),
            [
                "./mvnw",
                "-B",
                "-ntp",
                "-f",
                component.manifest_path,
                "test",
            ],
        )],
        Mode::Package => vec![step(
            &format!("package {}", component.name),
            [
                "./mvnw",
                "-B",
                "-ntp",
                "-f",
                component.manifest_path,
                "-DskipTests",
                "package",
            ],
        )],
    }
}

fn node_library_steps(library: &LibraryLayout, mode: Mode) -> Vec<Step> {
    match mode {
        Mode::Verify => vec![step(
            &format!("run {} checks", library.name),
            [
                "sh",
                "-c",
                &format!(
                    "cd {} && npm run lint && npm run typecheck && npm run build && npm test && npm run package",
                    library.project_dir
                ),
            ],
        )],
        Mode::Build => vec![step(
            &format!("build {}", library.name),
            [
                "sh",
                "-c",
                &format!(
                    "cd {} && npm run lint && npm run typecheck && npm run build",
                    library.project_dir
                ),
            ],
        )],
        Mode::Test => vec![step(
            &format!("test {}", library.name),
            [
                "sh",
                "-c",
                &format!("cd {} && npm run build && npm test", library.project_dir),
            ],
        )],
        Mode::Package => vec![step(
            &format!("package {}", library.name),
            [
                "sh",
                "-c",
                &format!(
                    "mkdir -p target/node-packages && cd {} && npm run lint && npm run typecheck && npm run build && npm pack --pack-destination ../../target/node-packages",
                    library.project_dir
                ),
            ],
        )],
    }
}

fn maven_library_steps(library: &LibraryLayout, mode: Mode) -> Vec<Step> {
    match mode {
        Mode::Verify => vec![step(
            &format!("verify {}", library.name),
            [
                "./mvnw",
                "-B",
                "-ntp",
                "-f",
                library.manifest_path,
                "verify",
            ],
        )],
        Mode::Build => vec![step(
            &format!("build {}", library.name),
            [
                "./mvnw",
                "-B",
                "-ntp",
                "-f",
                library.manifest_path,
                "-DskipTests",
                "compile",
            ],
        )],
        Mode::Test => vec![step(
            &format!("test {}", library.name),
            ["./mvnw", "-B", "-ntp", "-f", library.manifest_path, "test"],
        )],
        Mode::Package => vec![step(
            &format!("package {}", library.name),
            [
                "./mvnw",
                "-B",
                "-ntp",
                "-f",
                library.manifest_path,
                "-DskipTests",
                "package",
            ],
        )],
    }
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

fn openshift_deploy_steps(layout: &RepoLayout, openshift: &OpenshiftConfig) -> Vec<Step> {
    vec![
        shell_step("log in to OpenShift", &openshift.login_command()),
        step(
            "apply registry pull secret",
            [
                "sh",
                layout.openshift_pull_secret_helper_path,
                "company-ci-registry",
            ],
        ),
        step(
            "apply openshift dev overlay",
            ["oc", "apply", "-k", layout.openshift_overlay_path],
        ),
        shell_step(
            "set next-web image",
            &format!(
                "oc set image deployment/{} {}={}",
                layout.next_web.deployment_name,
                layout.next_web.container_name,
                openshift_image_ref_expression(&layout.next_web)
            ),
        ),
        shell_step(
            "set spring-api image",
            &format!(
                "oc set image deployment/{} {}={}",
                layout.spring_api.deployment_name,
                layout.spring_api.container_name,
                openshift_image_ref_expression(&layout.spring_api)
            ),
        ),
        step(
            "verify next-web rollout",
            ["oc", "rollout", "status", "deployment/next-web"],
        ),
        step(
            "verify spring-api rollout",
            ["oc", "rollout", "status", "deployment/spring-api"],
        ),
        step(
            "check next-web route",
            [
                "sh",
                layout.openshift_route_check_helper_path,
                layout.next_web.name,
                layout.next_web.route_path,
                layout.next_web.route_expected_text,
            ],
        ),
        step(
            "check spring-api route",
            [
                "sh",
                layout.openshift_route_check_helper_path,
                layout.spring_api.name,
                layout.spring_api.route_path,
                layout.spring_api.route_expected_text,
            ],
        ),
    ]
}

fn openshift_deploy_requirements() -> Vec<EnvRequirement> {
    let mut requirements = OpenshiftConfig::auth_requirements();
    requirements.push(EnvRequirement::variable("COMPANY_CI_IMAGE_PULL_REGISTRY"));
    requirements.push(EnvRequirement::variable("COMPANY_CI_IMAGE_NAMESPACE"));
    requirements.push(EnvRequirement::variable("COMPANY_CI_IMAGE_TAG"));
    requirements.push(EnvRequirement::secret("COMPANY_CI_IMAGE_REGISTRY_USERNAME"));
    requirements.push(EnvRequirement::secret_or_file(
        "COMPANY_CI_IMAGE_REGISTRY_PASSWORD",
        "COMPANY_CI_IMAGE_REGISTRY_PASSWORD_FILE",
    ));
    requirements
}

fn openshift_image_ref_expression(app: &ApplicationLayout) -> String {
    format!(
        "\"${{{}:-${{COMPANY_CI_IMAGE_PULL_REGISTRY}}/${{COMPANY_CI_IMAGE_NAMESPACE}}/{}:${{COMPANY_CI_IMAGE_TAG}}}}\"",
        app.image_override_env, app.name
    )
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

fn company_ci_step(context: &ExecutionContext, description: &str, args: &[&str]) -> Step {
    let mut command = vec![context.company_ci_binary.clone()];
    command.extend(args.iter().map(|arg| (*arg).to_string()));
    owned_step(description, command)
}

fn noop_step(description: &str) -> Step {
    step(description, ["true"])
}

fn shell_step(description: &str, command: &str) -> Step {
    step(description, ["sh", "-c", command])
}

fn company_ci_shell_command(context: &ExecutionContext, args: &[&str]) -> String {
    let mut command = vec![sh_quote(&context.company_ci_binary)];
    if !args.is_empty() {
        command.push(args.join(" "));
    }
    command.join(" ")
}

fn image_build_command(
    engine: ContainerEngine,
    containerfile: &str,
    image_ref: &str,
    build_context: &str,
    image_platform: Option<&str>,
) -> String {
    let mut command = vec![engine.binary().to_string(), "build".to_string()];
    if let Some(platform) = image_platform {
        command.push("--platform".to_string());
        command.push(sh_quote(platform));
    }
    command.push("-f".to_string());
    command.push(sh_quote(containerfile));
    command.push("-t".to_string());
    command.push(sh_quote(image_ref));
    command.push(sh_quote(build_context));
    command.join(" ")
}

fn image_push_command(engine: ContainerEngine, image_ref: &str) -> String {
    match engine {
        ContainerEngine::Docker => format!("docker push {}", sh_quote(image_ref)),
        ContainerEngine::Podman => format!(
            "podman push --tls-verify=${{COMPANY_CI_IMAGE_TLS_VERIFY:-false}} {}",
            sh_quote(image_ref)
        ),
    }
}

fn openshift_e2e_settings() -> ImageSettings {
    let settings = ImageSettings::from_env(ImageProfile::OpenshiftLocal);
    if env_var_is_nonempty("COMPANY_CI_IMAGE_TAG") {
        settings
    } else {
        settings.with_tag(generate_unique_local_tag())
    }
}

fn openshift_default_env(settings: &ImageSettings) -> Vec<(&'static str, String)> {
    let mut defaults = vec![
        (
            "COMPANY_CI_IMAGE_PUSH_REGISTRY",
            settings.push_registry().to_string(),
        ),
        (
            "COMPANY_CI_IMAGE_PULL_REGISTRY",
            settings.pull_registry().to_string(),
        ),
        (
            "COMPANY_CI_IMAGE_NAMESPACE",
            settings.namespace().to_string(),
        ),
        ("COMPANY_CI_IMAGE_TAG", settings.tag().to_string()),
        (
            "COMPANY_CI_IMAGE_PLATFORM",
            default_openshift_image_platform().to_string(),
        ),
    ];

    if let Some(username) = settings.registry_username() {
        defaults.push(("COMPANY_CI_IMAGE_REGISTRY_USERNAME", username.to_string()));
    }
    if let Some(password_file) = settings.registry_password_file() {
        defaults.push((
            "COMPANY_CI_IMAGE_REGISTRY_PASSWORD_FILE",
            password_file.to_string(),
        ));
    }

    defaults
}

fn command_with_default_env(defaults: &[(&str, String)], command: &str) -> String {
    let mut parts = defaults
        .iter()
        .map(|(name, value)| format!("{name}=${{{name}:-{}}}", sh_quote(value)))
        .collect::<Vec<_>>();
    parts.push(command.to_string());
    parts.join(" ")
}

fn env_var_is_nonempty(name: &str) -> bool {
    env_var(name).is_some()
}

fn env_var(name: &str) -> Option<String> {
    env::var(name).ok().and_then(|value| {
        if value.trim().is_empty() {
            None
        } else {
            Some(value)
        }
    })
}

fn default_openshift_image_platform() -> &'static str {
    "linux/amd64"
}

fn generate_unique_local_tag() -> String {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("companyci.{}.{}", elapsed.as_secs(), elapsed.subsec_nanos())
}

fn sh_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn add_java_tools(tools: &mut Vec<&'static str>) {
    push_tool(tools, "java");
    push_tool(tools, "./mvnw");
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
        command: command.iter().map(|part| (*part).to_string()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo_layout::RepoLayout;
    use std::path::PathBuf;

    fn context(areas: &[Area]) -> ExecutionContext {
        ExecutionContext {
            company_ci_binary: "company-ci".to_string(),
            container_engine: ContainerEngine::Docker,
            impacted_areas: areas.to_vec(),
            repo_layout: RepoLayout::company_ci_lab(),
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
            .any(|step| step.description.contains("next-web")));
        assert!(plan
            .steps
            .iter()
            .any(|step| step.description.contains("node-lib")));
        assert!(!plan
            .steps
            .iter()
            .any(|step| step.description.contains("java-lib")));
        assert_eq!(
            plan.required_tools,
            vec!["node".to_string(), "npm".to_string()]
        );
    }

    #[test]
    fn build_plan_noops_when_nothing_is_impacted() {
        let plan = build_plan(&ExecutionContext {
            company_ci_binary: "company-ci".to_string(),
            container_engine: ContainerEngine::Docker,
            impacted_areas: vec![Area::Docs],
            repo_layout: RepoLayout::company_ci_lab(),
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
        let plan =
            publish_maven_lib_plan(&RepoLayout::company_ci_lab(), project_dir.to_str().unwrap())
                .unwrap();
        assert!(plan
            .steps
            .iter()
            .any(|step| step.description.contains("publish maven-lib")));
        assert_eq!(
            plan.required_tools,
            vec!["java".to_string(), "./mvnw".to_string()]
        );
        assert!(plan
            .dry_run_notes
            .iter()
            .any(|note| note.contains("maven deploy url")));
    }

    #[test]
    fn publish_npm_lib_plan_requires_node_tools_for_node_lib() {
        let project_dir = repo_root().join("libs/node-lib");
        let plan = publish_npm_lib_plan(
            &RepoLayout::company_ci_lab(),
            project_dir.to_str().unwrap(),
            "ci",
        )
        .unwrap();
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
        let error =
            publish_maven_lib_plan(&RepoLayout::company_ci_lab(), project_dir.to_str().unwrap())
                .unwrap_err();
        assert_eq!(
            error.to_string(),
            format!(
                "invalid publish target: maven-lib requires {}/pom.xml",
                project_dir.display()
            )
        );
    }

    #[test]
    fn deploy_openshift_plan_creates_pull_secret_and_checks_routes() {
        let plan = deploy_openshift_plan(&context(&[Area::Deploy])).unwrap();
        assert!(plan
            .steps
            .iter()
            .any(|step| step.description.contains("apply registry pull secret")));
        assert!(plan
            .steps
            .iter()
            .any(|step| step.command.join(" ").contains("check-route.sh next-web /")));
        assert!(plan.steps.iter().any(|step| step
            .command
            .join(" ")
            .contains("${COMPANY_CI_IMAGE_PULL_REGISTRY}")));
        assert!(plan.required_tools.iter().any(|tool| tool == "oc"));
        assert!(plan.required_tools.iter().any(|tool| tool == "curl"));
        assert!(plan.required_env.iter().any(|requirement| {
            matches!(
                requirement,
                EnvRequirement::Variable { name, .. } if name == "COMPANY_CI_OPENSHIFT_API_URL"
            )
        }));
    }

    #[test]
    fn e2e_openshift_plan_publishes_images_and_deploys() {
        let plan = e2e_openshift_plan(&context(&[Area::Testbeds])).unwrap();
        assert_eq!(
            plan.steps.first().unwrap().description,
            "verify all components"
        );
        assert!(plan
            .steps
            .iter()
            .any(|step| step.description.contains("publish images")));
        assert!(plan
            .steps
            .iter()
            .any(|step| step.command.join(" ").contains("deploy openshift")));
        assert!(plan.required_tools.iter().any(|tool| tool == "oc"));
        assert!(plan.required_tools.iter().any(|tool| tool == "curl"));
        assert!(!plan.required_tools.iter().any(|tool| tool == "cargo"));
        assert!(plan
            .dry_run_notes
            .iter()
            .any(|note| { note == "openshift image platform: linux/amd64" }));
        assert!(plan.steps.iter().any(|step| {
            step.command
                .join(" ")
                .contains("COMPANY_CI_IMAGE_PLATFORM=${COMPANY_CI_IMAGE_PLATFORM:-'linux/amd64'}")
        }));
    }

    #[test]
    fn image_build_command_adds_platform_when_requested() {
        let command = image_build_command(
            ContainerEngine::Docker,
            "apps/spring-api/Containerfile",
            "registry.example.test/company-ci/spring-api:dev",
            "apps/spring-api",
            Some("linux/amd64"),
        );
        assert!(command.contains("docker build --platform 'linux/amd64'"));
    }

    #[test]
    fn image_build_command_skips_platform_when_not_requested() {
        let command = image_build_command(
            ContainerEngine::Docker,
            "apps/spring-api/Containerfile",
            "registry.example.test/company-ci/spring-api:dev",
            "apps/spring-api",
            None,
        );
        assert!(!command.contains("--platform"));
    }
}
