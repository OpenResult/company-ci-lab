use crate::error::CompanyCiError;
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageProfile {
    Local,
    OpenshiftLocal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationImage {
    NextWeb,
    SpringApi,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageSettings {
    push_registry: String,
    pull_registry: String,
    namespace: String,
    tag: String,
    registry_username: Option<String>,
    registry_password: Option<String>,
    registry_password_file: Option<String>,
    next_web_override: Option<String>,
    spring_api_override: Option<String>,
}

struct ImageDefaults {
    push_registry: &'static str,
    pull_registry: &'static str,
    namespace: &'static str,
    tag: &'static str,
    registry_username: Option<&'static str>,
    registry_password_file: Option<&'static str>,
}

impl ImageProfile {
    fn defaults(self) -> ImageDefaults {
        match self {
            Self::Local => ImageDefaults {
                push_registry: "localhost:5001",
                pull_registry: "localhost:5001",
                namespace: "",
                tag: "dev",
                registry_username: None,
                registry_password_file: None,
            },
            Self::OpenshiftLocal => ImageDefaults {
                push_registry: "localhost:5002",
                pull_registry: "host.crc.testing:5002",
                namespace: "company-ci",
                tag: "dev",
                registry_username: Some("admin"),
                registry_password_file: Some("testbeds/repository/.runtime/admin.password"),
            },
        }
    }
}

impl ApplicationImage {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::NextWeb => "next-web",
            Self::SpringApi => "spring-api",
        }
    }

    pub(crate) fn override_env(self) -> &'static str {
        match self {
            Self::NextWeb => "NEXT_WEB_IMAGE_REF",
            Self::SpringApi => "SPRING_API_IMAGE_REF",
        }
    }
}

impl ImageSettings {
    pub fn from_env(profile: ImageProfile) -> Self {
        Self::from_lookup(profile, |key| env::var(key).ok())
    }

    fn from_lookup<F>(profile: ImageProfile, mut lookup: F) -> Self
    where
        F: FnMut(&str) -> Option<String>,
    {
        let defaults = profile.defaults();
        let push_registry_override = normalize_env_value(lookup("COMPANY_CI_IMAGE_PUSH_REGISTRY"));
        let push_registry = push_registry_override
            .clone()
            .unwrap_or_else(|| defaults.push_registry.to_string());
        let pull_registry = normalize_env_value(lookup("COMPANY_CI_IMAGE_PULL_REGISTRY"))
            .or_else(|| push_registry_override.clone())
            .unwrap_or_else(|| defaults.pull_registry.to_string());

        Self {
            push_registry,
            pull_registry,
            namespace: normalize_env_value(lookup("COMPANY_CI_IMAGE_NAMESPACE"))
                .unwrap_or_else(|| defaults.namespace.to_string()),
            tag: normalize_env_value(lookup("COMPANY_CI_IMAGE_TAG"))
                .unwrap_or_else(|| defaults.tag.to_string()),
            registry_username: normalize_env_value(lookup("COMPANY_CI_IMAGE_REGISTRY_USERNAME"))
                .or_else(|| defaults.registry_username.map(str::to_string)),
            registry_password: normalize_env_value(lookup("COMPANY_CI_IMAGE_REGISTRY_PASSWORD")),
            registry_password_file: normalize_env_value(lookup(
                "COMPANY_CI_IMAGE_REGISTRY_PASSWORD_FILE",
            ))
            .or_else(|| defaults.registry_password_file.map(str::to_string)),
            next_web_override: normalize_env_value(lookup(
                ApplicationImage::NextWeb.override_env(),
            )),
            spring_api_override: normalize_env_value(lookup(
                ApplicationImage::SpringApi.override_env(),
            )),
        }
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }

    pub fn push_ref(&self, app: ApplicationImage) -> String {
        self.image_ref(app, &self.push_registry)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn pull_ref(&self, app: ApplicationImage) -> String {
        self.image_ref(app, &self.pull_registry)
    }

    pub fn push_registry(&self) -> &str {
        &self.push_registry
    }

    pub fn pull_registry(&self) -> &str {
        &self.pull_registry
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn registry_username(&self) -> Option<&str> {
        self.registry_username.as_deref()
    }

    pub fn registry_password_file(&self) -> Option<&str> {
        self.registry_password_file.as_deref()
    }

    pub fn has_registry_auth(&self) -> bool {
        self.registry_username.is_some()
            && (self.registry_password.is_some() || self.registry_password_file.is_some())
    }

    pub fn validate_publish_contract(&self, plan_name: &str) -> Result<(), CompanyCiError> {
        let has_any_auth = self.registry_username.is_some()
            || self.registry_password.is_some()
            || self.registry_password_file.is_some();
        if !has_any_auth {
            return Ok(());
        }

        if self.registry_username.is_none() {
            return Err(CompanyCiError::MissingSecretEnv {
                plan: plan_name.to_string(),
                name: "COMPANY_CI_IMAGE_REGISTRY_USERNAME".to_string(),
            });
        }

        if self.registry_password.is_none() && self.registry_password_file.is_none() {
            return Err(CompanyCiError::MissingEnvOrFile {
                plan: plan_name.to_string(),
                env_name: "COMPANY_CI_IMAGE_REGISTRY_PASSWORD".to_string(),
                file_env_name: "COMPANY_CI_IMAGE_REGISTRY_PASSWORD_FILE".to_string(),
            });
        }

        Ok(())
    }

    fn image_ref(&self, app: ApplicationImage, registry: &str) -> String {
        if let Some(explicit) = self.explicit_override(app) {
            return explicit.to_string();
        }

        if self.namespace.is_empty() {
            format!("{registry}/{}:{}", app.name(), self.tag)
        } else {
            format!("{registry}/{}/{}:{}", self.namespace, app.name(), self.tag)
        }
    }

    fn explicit_override(&self, app: ApplicationImage) -> Option<&str> {
        match app {
            ApplicationImage::NextWeb => self.next_web_override.as_deref(),
            ApplicationImage::SpringApi => self.spring_api_override.as_deref(),
        }
    }
}

fn normalize_env_value(value: Option<String>) -> Option<String> {
    value.and_then(|candidate| {
        if candidate.trim().is_empty() {
            None
        } else {
            Some(candidate)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn from_map(profile: ImageProfile, entries: &[(&str, &str)]) -> ImageSettings {
        let values: HashMap<String, String> = entries
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();
        ImageSettings::from_lookup(profile, |key| values.get(key).cloned())
    }

    #[test]
    fn local_defaults_match_kind_registry_flow() {
        let settings = from_map(ImageProfile::Local, &[]);
        assert_eq!(settings.push_registry(), "localhost:5001");
        assert_eq!(settings.pull_registry(), "localhost:5001");
        assert_eq!(settings.namespace(), "");
        assert_eq!(settings.tag(), "dev");
        assert_eq!(
            settings.push_ref(ApplicationImage::NextWeb),
            "localhost:5001/next-web:dev"
        );
        assert!(!settings.has_registry_auth());
    }

    #[test]
    fn openshift_local_defaults_match_local_repository_contract() {
        let settings = from_map(ImageProfile::OpenshiftLocal, &[]);
        assert_eq!(settings.push_registry(), "localhost:5002");
        assert_eq!(settings.pull_registry(), "host.crc.testing:5002");
        assert_eq!(settings.namespace(), "company-ci");
        assert_eq!(settings.tag(), "dev");
        assert_eq!(settings.registry_username(), Some("admin"));
        assert_eq!(
            settings.registry_password_file(),
            Some("testbeds/repository/.runtime/admin.password")
        );
        assert_eq!(
            settings.pull_ref(ApplicationImage::SpringApi),
            "host.crc.testing:5002/company-ci/spring-api:dev"
        );
    }

    #[test]
    fn pull_registry_follows_push_registry_when_only_push_is_overridden() {
        let settings = from_map(
            ImageProfile::OpenshiftLocal,
            &[("COMPANY_CI_IMAGE_PUSH_REGISTRY", "registry.example.test")],
        );
        assert_eq!(settings.push_registry(), "registry.example.test");
        assert_eq!(settings.pull_registry(), "registry.example.test");
    }

    #[test]
    fn explicit_image_overrides_apply_to_push_and_pull_refs() {
        let settings = from_map(
            ImageProfile::OpenshiftLocal,
            &[(
                "NEXT_WEB_IMAGE_REF",
                "registry.example.test/custom/next-web:qa",
            )],
        );
        assert_eq!(
            settings.push_ref(ApplicationImage::NextWeb),
            "registry.example.test/custom/next-web:qa"
        );
        assert_eq!(
            settings.pull_ref(ApplicationImage::NextWeb),
            "registry.example.test/custom/next-web:qa"
        );
    }

    #[test]
    fn publish_contract_is_valid_without_registry_auth() {
        let settings = from_map(ImageProfile::Local, &[]);
        assert!(settings.validate_publish_contract("image-publish").is_ok());
    }

    #[test]
    fn publish_contract_requires_password_when_username_is_present() {
        let settings = from_map(
            ImageProfile::Local,
            &[("COMPANY_CI_IMAGE_REGISTRY_USERNAME", "robot")],
        );
        let error = settings
            .validate_publish_contract("image-publish")
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "missing required secret env or env file for plan image-publish: COMPANY_CI_IMAGE_REGISTRY_PASSWORD|COMPANY_CI_IMAGE_REGISTRY_PASSWORD_FILE"
        );
    }
}
