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
    pub steps: Vec<Step>,
}

impl Plan {
    pub fn new(name: impl Into<String>, steps: Vec<Step>) -> Self {
        Self { name: name.into(), steps }
    }
}

pub fn verify_plan(context: &ExecutionContext) -> Plan {
    let mut steps = vec![
        step("validate deployment manifests exist", ["test", "-f", "deploy/base/next-web/kustomization.yaml"]),
        step("validate spring api containerfile exists", ["test", "-f", "apps/spring-api/Containerfile"]),
    ];
    steps.extend(component_steps(context, Mode::Verify));
    Plan::new("verify", steps)
}

pub fn build_plan(context: &ExecutionContext) -> Plan {
    Plan::new("build", component_steps(context, Mode::Build))
}

pub fn test_plan(context: &ExecutionContext) -> Plan {
    Plan::new("test", component_steps(context, Mode::Test))
}

pub fn package_plan(context: &ExecutionContext) -> Plan {
    Plan::new("package", component_steps(context, Mode::Package))
}

pub fn publish_plan(context: &ExecutionContext) -> Plan {
    Plan::new("publish", component_steps(context, Mode::Publish))
}

pub fn image_build_plan(context: &ExecutionContext) -> Plan {
    let mut steps = Vec::new();
    if context.affects(Area::NextWeb) {
        steps.push(step("build next-web image", ["podman", "build", "-f", "apps/next-web/Containerfile", "-t", "localhost:5000/next-web:dev", "apps/next-web"]));
    }
    if context.affects(Area::SpringApi) {
        steps.push(step("build spring-api image", ["podman", "build", "-f", "apps/spring-api/Containerfile", "-t", "localhost:5000/spring-api:dev", "apps/spring-api"]));
    }
    if steps.is_empty() {
        steps.push(noop_step("no impacted application images detected"));
    }
    Plan::new("image-build", steps)
}

pub fn image_publish_plan(context: &ExecutionContext) -> Plan {
    let mut steps = Vec::new();
    if context.affects(Area::NextWeb) {
        steps.push(step("push next-web image", ["podman", "push", "localhost:5000/next-web:dev"]));
    }
    if context.affects(Area::SpringApi) {
        steps.push(step("push spring-api image", ["podman", "push", "localhost:5000/spring-api:dev"]));
    }
    if steps.is_empty() {
        steps.push(noop_step("no impacted application images detected"));
    }
    Plan::new("image-publish", steps)
}

pub fn deploy_kubernetes_plan() -> Plan {
    Plan::new("deploy-kubernetes", vec![
        step("apply kind overlay", ["kubectl", "apply", "-k", "deploy/kind/overlays/ci"]),
        step("verify next-web rollout", ["kubectl", "rollout", "status", "deployment/next-web"]),
        step("verify spring-api rollout", ["kubectl", "rollout", "status", "deployment/spring-api"]),
    ])
}

pub fn deploy_openshift_plan() -> Plan {
    Plan::new("deploy-openshift", vec![
        step("apply openshift dev overlay", ["oc", "apply", "-k", "deploy/openshift/overlays/dev"]),
        step("verify next-web rollout", ["oc", "rollout", "status", "deployment/next-web"]),
        step("verify spring-api rollout", ["oc", "rollout", "status", "deployment/spring-api"]),
    ])
}

pub fn env_up_kind_plan() -> Plan {
    Plan::new("env-up-kind", vec![
        step("create kind cluster", ["kind", "create", "cluster", "--config", "testbeds/kind/cluster-config.yaml"]),
        step("start local registry helper", ["sh", "testbeds/kind/registry.sh"]),
    ])
}

pub fn env_down_kind_plan() -> Plan {
    Plan::new("env-down-kind", vec![step("delete kind cluster", ["kind", "delete", "cluster"])])
}

pub fn env_up_nexus_plan() -> Plan {
    Plan::new("env-up-nexus", vec![step("start nexus", ["docker", "compose", "-f", "testbeds/repo/nexus/compose.yaml", "up", "-d"])])
}

pub fn env_down_nexus_plan() -> Plan {
    Plan::new("env-down-nexus", vec![step("stop nexus", ["docker", "compose", "-f", "testbeds/repo/nexus/compose.yaml", "down", "-v"])])
}

pub fn e2e_emulated_plan() -> Plan {
    Plan::new("e2e-emulated", vec![
        step("start nexus", ["docker", "compose", "-f", "testbeds/repo/nexus/compose.yaml", "up", "-d"]),
        step("create kind cluster", ["kind", "create", "cluster", "--config", "testbeds/kind/cluster-config.yaml"]),
        step("verify all components", ["cargo", "run", "-p", "company-ci", "--", "verify"]),
        step("package artifacts", ["cargo", "run", "-p", "company-ci", "--", "package"]),
        step("publish libraries", ["cargo", "run", "-p", "company-ci", "--", "publish"]),
        step("build images", ["cargo", "run", "-p", "company-ci", "--", "image", "build"]),
        step("publish images", ["cargo", "run", "-p", "company-ci", "--", "image", "publish"]),
        step("deploy to kind", ["cargo", "run", "-p", "company-ci", "--", "deploy", "kubernetes"]),
        step("tear down kind", ["kind", "delete", "cluster"]),
        step("tear down nexus", ["docker", "compose", "-f", "testbeds/repo/nexus/compose.yaml", "down", "-v"]),
    ])
}

pub fn e2e_openshift_local_plan() -> Plan {
    Plan::new("e2e-openshift-local", vec![
        step("assume OpenShift Local login", ["sh", "testbeds/openshift-local/scripts/login.sh"]),
        step("verify all components", ["cargo", "run", "-p", "company-ci", "--", "verify"]),
        step("build images", ["cargo", "run", "-p", "company-ci", "--", "image", "build"]),
        step("deploy openshift overlays", ["cargo", "run", "-p", "company-ci", "--", "deploy", "openshift"]),
    ])
}

#[derive(Clone, Copy)]
enum Mode { Verify, Build, Test, Package, Publish }

fn component_steps(context: &ExecutionContext, mode: Mode) -> Vec<Step> {
    let mut steps = Vec::new();

    if context.affects(Area::NextWeb) {
        steps.extend(match mode {
            Mode::Verify => vec![step("run next-web quality checks", ["sh", "-c", "cd apps/next-web && npm run lint && npm test && npm run build"])],
            Mode::Build => vec![step("build next-web", ["sh", "-c", "cd apps/next-web && npm run build"])],
            Mode::Test => vec![step("test next-web", ["sh", "-c", "cd apps/next-web && npm test"])],
            Mode::Package | Mode::Publish => vec![noop_step("next-web packaging/publishing is handled through image commands")],
        });
    }

    if context.affects(Area::SpringApi) {
        steps.extend(match mode {
            Mode::Verify | Mode::Build | Mode::Test | Mode::Package => vec![step("run spring-api scaffold checks", ["sh", "-c", "cd apps/spring-api && ./ci/verify.sh"])],
            Mode::Publish => vec![noop_step("spring-api publishing is handled through image commands")],
        });
    }

    if context.affects(Area::NodeLib) {
        steps.extend(match mode {
            Mode::Verify => vec![step("run node-lib checks", ["sh", "-c", "cd libs/node-lib && npm run build && npm test && npm run package"])],
            Mode::Build => vec![step("build node-lib", ["sh", "-c", "cd libs/node-lib && npm run build"])],
            Mode::Test => vec![step("test node-lib", ["sh", "-c", "cd libs/node-lib && npm test"])],
            Mode::Package => vec![step("package node-lib", ["sh", "-c", "cd libs/node-lib && npm run build && npm pack --dry-run"])],
            Mode::Publish => vec![step("publish node-lib to npm-style repo", ["sh", "-c", "cd libs/node-lib && npm publish --registry ${NPM_REGISTRY_URL:-http://localhost:8081/repository/npm-hosted/} --dry-run"])],
        });
    }

    if context.affects(Area::JavaLib) {
        steps.extend(match mode {
            Mode::Verify | Mode::Build | Mode::Test | Mode::Package | Mode::Publish => vec![step("run java-lib scaffold checks", ["sh", "-c", "cd libs/java-lib && ./ci/verify.sh"])],
        });
    }

    if steps.is_empty() {
        steps.push(noop_step("no impacted component work detected"));
    }

    steps
}

fn noop_step(description: &str) -> Step {
    step(description, ["true"])
}

fn step<const N: usize>(description: &str, command: [&str; N]) -> Step {
    Step { description: description.to_string(), command: command.iter().map(|p| p.to_string()).collect() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(areas: &[Area]) -> ExecutionContext {
        ExecutionContext { impacted_areas: areas.to_vec() }
    }

    #[test]
    fn verify_plan_contains_requested_component_checks() {
        let plan = verify_plan(&context(&[Area::NextWeb, Area::NodeLib]));
        assert_eq!(plan.name, "verify");
        assert!(plan.steps.iter().any(|s| s.description.contains("next-web")));
        assert!(plan.steps.iter().any(|s| s.description.contains("node-lib")));
        assert!(!plan.steps.iter().any(|s| s.description.contains("java-lib")));
    }

    #[test]
    fn e2e_emulated_plan_orders_environment_before_deploy() {
        let plan = e2e_emulated_plan();
        assert_eq!(plan.steps.first().unwrap().description, "start nexus");
        assert_eq!(plan.steps.last().unwrap().description, "tear down nexus");
    }

    #[test]
    fn build_plan_noops_when_nothing_is_impacted() {
        let plan = build_plan(&ExecutionContext { impacted_areas: vec![Area::Docs] });
        assert_eq!(plan.steps, vec![noop_step("no impacted component work detected")]);
    }
}
