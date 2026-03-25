use crate::error::CompanyCiError;
use crate::requirements::EnvRequirement;
use std::env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenshiftConfig {
    skip_tls_verify: bool,
}

impl OpenshiftConfig {
    pub fn from_env(plan_name: &str) -> Result<Self, CompanyCiError> {
        Self::from_lookup(plan_name, |key| env::var(key).ok())
    }

    fn from_lookup<F>(plan_name: &str, mut lookup: F) -> Result<Self, CompanyCiError>
    where
        F: FnMut(&str) -> Option<String>,
    {
        Ok(Self {
            skip_tls_verify: parse_optional_bool(
                plan_name,
                "COMPANY_CI_OPENSHIFT_SKIP_TLS_VERIFY",
                lookup("COMPANY_CI_OPENSHIFT_SKIP_TLS_VERIFY"),
            )?
            .unwrap_or(false),
        })
    }

    pub fn auth_requirements() -> Vec<EnvRequirement> {
        vec![
            EnvRequirement::variable("COMPANY_CI_OPENSHIFT_API_URL"),
            EnvRequirement::secret("COMPANY_CI_OPENSHIFT_TOKEN"),
        ]
    }

    pub fn login_command(&self) -> String {
        let mut command =
            "oc login \"$COMPANY_CI_OPENSHIFT_API_URL\" --token \"$COMPANY_CI_OPENSHIFT_TOKEN\""
                .to_string();
        if self.skip_tls_verify {
            command.push_str(" --insecure-skip-tls-verify=true");
        }
        command
    }

    pub fn skip_tls_verify(&self) -> bool {
        self.skip_tls_verify
    }
}

fn parse_optional_bool(
    plan_name: &str,
    env_name: &str,
    value: Option<String>,
) -> Result<Option<bool>, CompanyCiError> {
    let Some(value) = value else {
        return Ok(None);
    };

    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "" => Ok(None),
        "1" | "true" | "yes" => Ok(Some(true)),
        "0" | "false" | "no" => Ok(Some(false)),
        _ => Err(CompanyCiError::InvalidEnvValue {
            plan: plan_name.to_string(),
            name: env_name.to_string(),
            message: "expected one of: true, false, 1, 0, yes, no".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn from_map(entries: &[(&str, &str)]) -> Result<OpenshiftConfig, CompanyCiError> {
        let values: HashMap<String, String> = entries
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();
        OpenshiftConfig::from_lookup("deploy-openshift", |key| values.get(key).cloned())
    }

    #[test]
    fn defaults_skip_tls_verify_to_false() {
        let config = from_map(&[]).unwrap();
        assert!(!config.skip_tls_verify());
    }

    #[test]
    fn accepts_truthy_skip_tls_verify() {
        let config = from_map(&[("COMPANY_CI_OPENSHIFT_SKIP_TLS_VERIFY", "yes")]).unwrap();
        assert!(config.skip_tls_verify());
    }

    #[test]
    fn rejects_invalid_skip_tls_verify_values() {
        let error = from_map(&[("COMPANY_CI_OPENSHIFT_SKIP_TLS_VERIFY", "sometimes")]).unwrap_err();
        assert_eq!(
            error.to_string(),
            "invalid env value for plan deploy-openshift: COMPANY_CI_OPENSHIFT_SKIP_TLS_VERIFY (expected one of: true, false, 1, 0, yes, no)"
        );
    }
}
