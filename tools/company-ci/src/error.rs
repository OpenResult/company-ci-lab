use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum CompanyCiError {
    InvalidArgument(String),
    Usage(String),
    Runtime(String),
    MissingTool { plan: String, tool: String },
    CommandFailed { command: String, status: i32 },
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
            Self::CommandFailed { command, status } => {
                write!(f, "command failed with status {status}: {command}")
            }
        }
    }
}

impl std::error::Error for CompanyCiError {}
