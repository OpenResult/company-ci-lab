use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum CompanyCiError {
    InvalidArgument(String),
    Usage(String),
    Runtime(String),
    MissingTool {
        plan: String,
        tool: String,
    },
    MissingEnv {
        plan: String,
        name: String,
    },
    MissingSecretEnv {
        plan: String,
        name: String,
    },
    MissingEnvFile {
        plan: String,
        name: String,
        path: String,
    },
    MissingEnvOrFile {
        plan: String,
        env_name: String,
        file_env_name: String,
    },
    InvalidEnvValue {
        plan: String,
        name: String,
        message: String,
    },
    CommandFailed {
        command: String,
        status: i32,
    },
}

impl Display for CompanyCiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArgument(message) => write!(f, "{message}"),
            Self::Usage(message) => write!(f, "{message}"),
            Self::Runtime(message) => write!(f, "{message}"),
            Self::MissingTool { plan, tool } => {
                write!(f, "required tool not found on PATH for plan {plan}: {tool}")
            }
            Self::MissingEnv { plan, name } => {
                write!(f, "missing required env for plan {plan}: {name}")
            }
            Self::MissingSecretEnv { plan, name } => {
                write!(f, "missing required secret env for plan {plan}: {name}")
            }
            Self::MissingEnvFile { plan, name, path } => {
                write!(
                    f,
                    "required env file not found for plan {plan}: {name} -> {path}"
                )
            }
            Self::MissingEnvOrFile {
                plan,
                env_name,
                file_env_name,
            } => write!(
                f,
                "missing required secret env or env file for plan {plan}: {env_name}|{file_env_name}"
            ),
            Self::InvalidEnvValue {
                plan,
                name,
                message,
            } => write!(f, "invalid env value for plan {plan}: {name} ({message})"),
            Self::CommandFailed { command, status } => {
                write!(f, "command failed with status {status}: {command}")
            }
        }
    }
}

impl std::error::Error for CompanyCiError {}
