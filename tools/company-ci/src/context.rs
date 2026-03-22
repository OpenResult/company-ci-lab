use crate::container_engine::ContainerEngine;
use crate::error::CompanyCiError;
use crate::impact::{infer_areas, Area};
use std::env;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub company_ci_binary: String,
    pub container_engine: ContainerEngine,
    pub impacted_areas: Vec<Area>,
}

impl ExecutionContext {
    pub fn detect() -> Result<Self, CompanyCiError> {
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

        let company_ci_binary = env::current_exe()
            .map_err(|error| {
                CompanyCiError::Runtime(format!("failed to resolve company-ci executable: {error}"))
            })?
            .to_string_lossy()
            .into_owned();

        Ok(Self {
            company_ci_binary,
            container_engine: ContainerEngine::detect()?,
            impacted_areas,
        })
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
            company_ci_binary: "company-ci".to_string(),
            container_engine: ContainerEngine::Docker,
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
