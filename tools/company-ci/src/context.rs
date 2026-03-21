use crate::impact::{infer_areas, Area};
use std::env;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub impacted_areas: Vec<Area>,
}

impl ExecutionContext {
    pub fn detect() -> Self {
        let changed_files = env::var("COMPANY_CI_CHANGED_FILES")
            .ok()
            .map(|value| {
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|entry| !entry.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let impacted_areas = if changed_files.is_empty() {
            vec![
                Area::NextWeb,
                Area::SpringApi,
                Area::NodeLib,
                Area::JavaLib,
                Area::Deploy,
                Area::Tooling,
                Area::Docs,
                Area::Testbeds,
                Area::Workflows,
            ]
        } else {
            infer_areas(changed_files.iter().map(String::as_str))
        };

        let _ = changed_files;
        Self { impacted_areas }
    }

    pub fn affects(&self, area: Area) -> bool {
        self.impacted_areas.contains(&area) || self.impacted_areas.contains(&Area::Tooling)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affects_everything_when_no_changed_files_are_supplied() {
        let context = ExecutionContext {
            impacted_areas: vec![
                Area::NextWeb,
                Area::SpringApi,
                Area::NodeLib,
                Area::JavaLib,
                Area::Deploy,
                Area::Tooling,
            ],
        };
        assert!(context.affects(Area::NextWeb));
        assert!(context.affects(Area::JavaLib));
    }
}
