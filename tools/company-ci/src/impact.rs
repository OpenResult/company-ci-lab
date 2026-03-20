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

pub fn infer_areas<'a>(paths: impl IntoIterator<Item = &'a str>) -> Vec<Area> {
    let mut areas = Vec::new();
    for path in paths {
        for area in classify_path(path) {
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

fn classify_path(path: &str) -> Vec<Area> {
    let mut areas = Vec::new();
    if path.starts_with("apps/next-web/") {
        areas.push(Area::NextWeb);
    }
    if path.starts_with("apps/spring-api/") {
        areas.push(Area::SpringApi);
    }
    if path.starts_with("libs/node-lib/") {
        areas.push(Area::NodeLib);
    }
    if path.starts_with("libs/java-lib/") {
        areas.push(Area::JavaLib);
    }
    if path.starts_with("deploy/") {
        areas.push(Area::Deploy);
    }
    if path.starts_with("tools/company-ci/") || path == "Cargo.toml" || path == "Cargo.lock" {
        areas.push(Area::Tooling);
    }
    if path.starts_with("docs/") || path == "README.md" {
        areas.push(Area::Docs);
    }
    if path.starts_with("testbeds/") || path.starts_with("scripts/") {
        areas.push(Area::Testbeds);
    }
    if path.starts_with(".github/workflows/") || path.starts_with(".github/ISSUE_TEMPLATE/") || path == ".github/pull_request_template.md" {
        areas.push(Area::Workflows);
    }
    areas
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_component_paths_to_areas() {
        let areas = infer_areas(["apps/next-web/src/app/page.tsx", "libs/java-lib/pom.xml"]);
        assert!(areas.contains(&Area::NextWeb));
        assert!(areas.contains(&Area::JavaLib));
    }

    #[test]
    fn defaults_to_tooling_when_no_paths_are_available() {
        let areas = infer_areas(Vec::<&str>::new());
        assert_eq!(areas, vec![Area::Tooling]);
    }
}
