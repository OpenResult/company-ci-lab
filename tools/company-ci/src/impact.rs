use crate::repo_layout::RepoLayout;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Area {
    NextWeb,
    SpringApi,
    NodeLib,
    JavaLib,
    Deploy,
    Tooling,
    Docs,
    Testbeds,
    Workflows,
}

pub fn infer_areas<'a>(layout: &RepoLayout, paths: impl IntoIterator<Item = &'a str>) -> Vec<Area> {
    let mut areas = Vec::new();
    for path in paths {
        for area in classify_path(layout, path) {
            if !areas.contains(&area) {
                areas.push(area);
            }
        }
    }
    if areas.is_empty() {
        areas.push(Area::Tooling);
    }
    areas
}

fn classify_path(layout: &RepoLayout, path: &str) -> Vec<Area> {
    let mut areas = Vec::new();
    if path.starts_with(&component_prefix(layout.next_web.project_dir)) {
        areas.push(Area::NextWeb);
    }
    if path.starts_with(&component_prefix(layout.spring_api.project_dir)) {
        areas.push(Area::SpringApi);
    }
    if path.starts_with(&component_prefix(layout.node_lib.project_dir)) {
        areas.push(Area::NodeLib);
    }
    if path.starts_with(&component_prefix(layout.java_lib.project_dir)) {
        areas.push(Area::JavaLib);
    }
    if path.starts_with(layout.deploy_root) {
        areas.push(Area::Deploy);
    }
    if path.starts_with(layout.tooling_root)
        || path == layout.cargo_toml_path
        || path == layout.cargo_lock_path
    {
        areas.push(Area::Tooling);
    }
    if path.starts_with(layout.docs_root) || path == layout.readme_path {
        areas.push(Area::Docs);
    }
    if path.starts_with(layout.testbeds_root) || path.starts_with(layout.scripts_root) {
        areas.push(Area::Testbeds);
    }
    if path.starts_with(layout.workflows_root)
        || path.starts_with(layout.issue_template_root)
        || path == layout.pull_request_template
    {
        areas.push(Area::Workflows);
    }
    areas
}

fn component_prefix(root_dir: &str) -> String {
    format!("{root_dir}/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_component_paths_to_areas() {
        let layout = RepoLayout::company_ci_lab();
        let areas = infer_areas(
            &layout,
            ["apps/next-web/src/app/page.tsx", "libs/java-lib/pom.xml"],
        );
        assert!(areas.contains(&Area::NextWeb));
        assert!(areas.contains(&Area::JavaLib));
    }

    #[test]
    fn defaults_to_tooling_when_no_paths_are_available() {
        let layout = RepoLayout::company_ci_lab();
        let areas = infer_areas(&layout, Vec::<&str>::new());
        assert_eq!(areas, vec![Area::Tooling]);
    }
}
