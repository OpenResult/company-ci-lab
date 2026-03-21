use crate::context::ExecutionContext;
use crate::impact::Area;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub description: String,
    pub command: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub name: String,
    pub required_tools: Vec<String>,
    pub steps: Vec<Step>,
}

impl Plan {
    pub fn new(name: impl Into<String>, steps: Vec<Step>) -> Self {
        Self {
            name: name.into(),
            required_tools: Vec::new(),
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

pub fn publish_plan(context: &ExecutionContext) -> Plan {
    Plan::new("publish", component_steps(context, Mode::Publish))
        .with_required_tools(component_required_tools(context, Mode::Publish))
}

pub fn image_build_plan(context: &ExecutionContext) -> Plan {
    let mut steps = Vec::new();
    let mut required_tools = Vec::new();
    if context.affects(Area::NextWeb) {
        steps.push(step(
            "build next-web image inputs",
            ["sh", "-c", "cd apps/next-web && npm run build"],
        ));
        steps.push(step(
            "build next-web image",
            [
                "sh",
                "-c",
                "image_ref=${NEXT_WEB_IMAGE_REF:-localhost:5001/next-web:dev} && podman build -f apps/next-web/Containerfile -t ${image_ref} apps/next-web",
            ],
        ));
        push_tool(&mut required_tools, "node");
        push_tool(&mut required_tools, "npm");
        push_tool(&mut required_tools, "podman");
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
        steps.push(step(
            "build spring-api image",
            [
                "sh",
                "-c",
                "image_ref=${SPRING_API_IMAGE_REF:-localhost:5001/spring-api:dev} && podman build -f apps/spring-api/Containerfile -t ${image_ref} apps/spring-api",
            ],
        ));
        push_tool(&mut required_tools, "java");
        push_tool(&mut required_tools, "mvn");
        push_tool(&mut required_tools, "podman");
    }
    if steps.is_empty() {
        steps.push(noop_step("no impacted application images detected"));
    }
    Plan::new("image-build", steps).with_required_tools(required_tools)
}

pub fn image_publish_plan(context: &ExecutionContext) -> Plan {
    let mut steps = Vec::new();
    let mut required_tools = Vec::new();
    if context.affects(Area::NextWeb) {
        steps.push(step(
            "push next-web image",
            [
                "sh",
                "-c",
                "image_ref=${NEXT_WEB_IMAGE_REF:-localhost:5001/next-web:dev} && podman push --tls-verify=${COMPANY_CI_IMAGE_TLS_VERIFY:-false} ${image_ref}",
            ],
        ));
        push_tool(&mut required_tools, "podman");
    }
    if context.affects(Area::SpringApi) {
        steps.push(step(
            "push spring-api image",
            [
                "sh",
                "-c",
                "image_ref=${SPRING_API_IMAGE_REF:-localhost:5001/spring-api:dev} && podman push --tls-verify=${COMPANY_CI_IMAGE_TLS_VERIFY:-false} ${image_ref}",
            ],
        ));
        push_tool(&mut required_tools, "podman");
    }
    if steps.is_empty() {
        steps.push(noop_step("no impacted application images detected"));
    }
    Plan::new("image-publish", steps).with_required_tools(required_tools)
}

pub fn deploy_kubernetes_plan() -> Plan {
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

pub fn deploy_openshift_plan() -> Plan {
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

pub fn env_up_kind_plan() -> Plan {
    Plan::new(
        "env-up-kind",
        vec![
            step(
                "create kind cluster",
                [
                    "kind",
                    "create",
                    "cluster",
                    "--config",
                    "testbeds/kind/cluster-config.yaml",
                ],
            ),
            step(
                "start local registry helper",
                ["sh", "testbeds/kind/registry.sh", "up"],
            ),
        ],
    )
    .with_required_tools(["kind", "docker", "kubectl"])
}

pub fn env_down_kind_plan() -> Plan {
    Plan::new(
        "env-down-kind",
        vec![
            step(
                "delete kind cluster",
                ["sh", "-c", "kind delete cluster || true"],
            ),
            step(
                "stop local registry helper",
                ["sh", "testbeds/kind/registry.sh", "down"],
            ),
        ],
    )
    .with_required_tools(["kind", "docker"])
}

pub fn env_up_nexus_plan() -> Plan {
    Plan::new(
        "env-up-nexus",
        vec![
            step(
                "start nexus",
                [
                    "docker",
                    "compose",
                    "-f",
                    "testbeds/repo/nexus/compose.yaml",
                    "up",
                    "-d",
                ],
            ),
            step(
                "wait for nexus and capture runtime state",
                ["sh", "testbeds/repo/nexus/bootstrap.sh"],
            ),
        ],
    )
    .with_required_tools(["docker", "curl"])
}

pub fn env_down_nexus_plan() -> Plan {
    Plan::new(
        "env-down-nexus",
        vec![
            step(
                "stop nexus",
                [
                    "docker",
                    "compose",
                    "-f",
                    "testbeds/repo/nexus/compose.yaml",
                    "down",
                    "-v",
                ],
            ),
            step(
                "remove nexus runtime state",
                ["sh", "-c", "rm -rf testbeds/repo/nexus/.runtime"],
            ),
        ],
    )
    .with_required_tools(["docker"])
}

pub fn e2e_emulated_plan() -> Plan {
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
                "publish libraries",
                ["cargo", "run", "-p", "company-ci", "--", "publish"],
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
        "cargo", "curl", "docker", "kind", "kubectl", "java", "mvn", "node", "npm", "podman",
    ])
}

pub fn e2e_openshift_local_plan() -> Plan {
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
    .with_required_tools(["cargo", "oc", "java", "mvn", "node", "npm", "podman"])
}

#[derive(Clone, Copy)]
enum Mode {
    Verify,
    Build,
    Test,
    Package,
    Publish,
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
            Mode::Package | Mode::Publish => vec![noop_step(
                "next-web packaging/publishing is handled through image commands",
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
            Mode::Publish => vec![noop_step(
                "spring-api publishing is handled through image commands",
            )],
        });
    }

    if context.affects(Area::NodeLib) {
        steps.extend(match mode {
            Mode::Verify => vec![step("run node-lib checks", ["sh", "-c", "cd libs/node-lib && npm run lint && npm run typecheck && npm run build && npm test && npm run package"])],
            Mode::Build => vec![step("build node-lib", ["sh", "-c", "cd libs/node-lib && npm run lint && npm run typecheck && npm run build"])],
            Mode::Test => vec![step("test node-lib", ["sh", "-c", "cd libs/node-lib && npm run build && npm test"])],
            Mode::Package => vec![step("package node-lib", ["sh", "-c", "mkdir -p target/node-packages && cd libs/node-lib && npm run lint && npm run typecheck && npm run build && npm pack --pack-destination ../../target/node-packages"])],
            Mode::Publish => vec![step("publish node-lib to npm-style repo", ["sh", "testbeds/repo/nexus/npm-publish.sh", "libs/node-lib"])],
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
            Mode::Publish => vec![step(
                "publish java-lib to maven-style repo",
                [
                    "sh",
                    "testbeds/repo/nexus/maven-deploy.sh",
                    "libs/java-lib/pom.xml",
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
            Mode::Verify | Mode::Build | Mode::Test | Mode::Package | Mode::Publish
        )
    {
        add_node_tools(&mut tools);
    }

    if context.affects(Area::JavaLib)
        && matches!(
            mode,
            Mode::Verify | Mode::Build | Mode::Test | Mode::Package | Mode::Publish
        )
    {
        add_java_tools(&mut tools);
    }

    tools
}

fn noop_step(description: &str) -> Step {
    step(description, ["true"])
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

    fn context(areas: &[Area]) -> ExecutionContext {
        ExecutionContext {
            impacted_areas: areas.to_vec(),
        }
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
        let plan = e2e_emulated_plan();
        assert_eq!(plan.steps.first().unwrap().description, "start nexus");
        assert_eq!(plan.steps.last().unwrap().description, "tear down nexus");
        assert!(plan.required_tools.iter().any(|tool| tool == "curl"));
        assert!(plan.required_tools.iter().any(|tool| tool == "mvn"));
        assert!(plan.required_tools.iter().any(|tool| tool == "podman"));
    }

    #[test]
    fn build_plan_noops_when_nothing_is_impacted() {
        let plan = build_plan(&ExecutionContext {
            impacted_areas: vec![Area::Docs],
        });
        assert_eq!(
            plan.steps,
            vec![noop_step("no impacted component work detected")]
        );
        assert!(plan.required_tools.is_empty());
    }

    #[test]
    fn publish_plan_requires_java_tools_for_java_lib() {
        let plan = publish_plan(&context(&[Area::JavaLib]));
        assert!(plan
            .steps
            .iter()
            .any(|step| step.description.contains("publish java-lib")));
        assert_eq!(
            plan.required_tools,
            vec!["java".to_string(), "mvn".to_string()]
        );
    }

    #[test]
    fn deploy_kubernetes_plan_checks_live_services() {
        let plan = deploy_kubernetes_plan();
        assert!(plan
            .steps
            .iter()
            .any(|step| step.description.contains("check next-web homepage")));
        assert!(plan.required_tools.iter().any(|tool| tool == "curl"));
    }
}
