use crate::error::CompanyCiError;
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerEngine {
    Docker,
    Podman,
}

impl ContainerEngine {
    pub fn detect() -> Result<Self, CompanyCiError> {
        match env::var("COMPANY_CI_CONTAINER_ENGINE") {
            Ok(value) => Self::parse(&value),
            Err(env::VarError::NotPresent) => Ok(Self::Docker),
            Err(env::VarError::NotUnicode(_)) => Err(CompanyCiError::InvalidArgument(
                "COMPANY_CI_CONTAINER_ENGINE must be valid UTF-8".to_string(),
            )),
        }
    }

    pub fn parse(value: &str) -> Result<Self, CompanyCiError> {
        match value.trim() {
            "docker" => Ok(Self::Docker),
            "podman" => Ok(Self::Podman),
            other => Err(CompanyCiError::InvalidArgument(format!(
                "unsupported container engine: {other}. Expected docker or podman"
            ))),
        }
    }

    pub fn binary(self) -> &'static str {
        match self {
            Self::Docker => "docker",
            Self::Podman => "podman",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_engines() {
        assert_eq!(
            ContainerEngine::parse("docker").unwrap(),
            ContainerEngine::Docker
        );
        assert_eq!(
            ContainerEngine::parse("podman").unwrap(),
            ContainerEngine::Podman
        );
    }

    #[test]
    fn rejects_unknown_engine() {
        let error = ContainerEngine::parse("nerdctl").unwrap_err();
        assert!(error.to_string().contains("unsupported container engine"));
    }
}
