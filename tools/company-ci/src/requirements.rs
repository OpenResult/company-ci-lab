#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvRequirement {
    Variable {
        name: String,
        secret: bool,
    },
    VariableOrFile {
        variable_name: String,
        file_name: String,
        secret: bool,
    },
}

impl EnvRequirement {
    pub fn variable(name: impl Into<String>) -> Self {
        Self::Variable {
            name: name.into(),
            secret: false,
        }
    }

    pub fn secret(name: impl Into<String>) -> Self {
        Self::Variable {
            name: name.into(),
            secret: true,
        }
    }

    pub fn secret_or_file(variable_name: impl Into<String>, file_name: impl Into<String>) -> Self {
        Self::VariableOrFile {
            variable_name: variable_name.into(),
            file_name: file_name.into(),
            secret: true,
        }
    }

    #[cfg(test)]
    pub fn display_name(&self) -> String {
        match self {
            Self::Variable { name, .. } => name.clone(),
            Self::VariableOrFile {
                variable_name,
                file_name,
                ..
            } => format!("{variable_name}|{file_name}"),
        }
    }

    pub fn dry_run_message(&self) -> String {
        match self {
            Self::Variable {
                name,
                secret: false,
            } => {
                format!("verify required env: {name}")
            }
            Self::Variable { name, secret: true } => {
                format!("verify required secret env: {name}")
            }
            Self::VariableOrFile {
                variable_name,
                file_name,
                secret: false,
            } => format!("verify required env or file: {variable_name}|{file_name}"),
            Self::VariableOrFile {
                variable_name,
                file_name,
                secret: true,
            } => format!("verify required secret env or file: {variable_name}|{file_name}"),
        }
    }
}
