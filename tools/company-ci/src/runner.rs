use crate::error::CompanyCiError;
use crate::plan::{Plan, Step};
use crate::requirements::EnvRequirement;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

pub trait CommandRunner {
    fn check_tool(&self, plan: &Plan, tool: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn check_env(
        &self,
        plan: &Plan,
        requirement: &EnvRequirement,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn run(&self, step: &Step) -> Result<(), Box<dyn std::error::Error>>;

    fn run_plan(&self, plan: &Plan, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
        if dry_run {
            for note in &plan.dry_run_notes {
                println!("[dry-run] {note}");
            }
        }

        for tool in &plan.required_tools {
            if dry_run {
                println!("[dry-run] verify required tool: {tool}");
            } else {
                self.check_tool(plan, tool)?;
            }
        }

        for requirement in &plan.required_env {
            if dry_run {
                println!("[dry-run] {}", requirement.dry_run_message());
            } else {
                self.check_env(plan, requirement)?;
            }
        }

        for step in &plan.steps {
            if dry_run {
                println!(
                    "[dry-run] {} => {}",
                    step.description,
                    step.command.join(" ")
                );
            } else {
                self.run(step)?;
            }
        }
        Ok(())
    }
}

pub struct ShellRunner;

impl CommandRunner for ShellRunner {
    fn check_tool(&self, plan: &Plan, tool: &str) -> Result<(), Box<dyn std::error::Error>> {
        if tool_on_path(tool) {
            Ok(())
        } else {
            Err(Box::new(CompanyCiError::MissingTool {
                plan: plan.name.clone(),
                tool: tool.to_string(),
            }))
        }
    }

    fn run(&self, step: &Step) -> Result<(), Box<dyn std::error::Error>> {
        let (program, args) = step
            .command
            .split_first()
            .expect("step command must not be empty");
        let status = Command::new(program).args(args).status()?;
        if status.success() {
            Ok(())
        } else {
            Err(Box::new(CompanyCiError::CommandFailed {
                command: step.command.join(" "),
                status: status.code().unwrap_or(1),
            }))
        }
    }

    fn check_env(
        &self,
        plan: &Plan,
        requirement: &EnvRequirement,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match requirement {
            EnvRequirement::Variable {
                name,
                secret: false,
            } => require_env(plan, name),
            EnvRequirement::Variable { name, secret: true } => require_secret_env(plan, name),
            EnvRequirement::VariableOrFile {
                variable_name,
                file_name,
                ..
            } => require_env_or_file(plan, variable_name, file_name),
        }
    }
}

fn tool_on_path(tool: &str) -> bool {
    if tool.contains(std::path::MAIN_SEPARATOR) {
        return is_executable(Path::new(tool));
    }

    let Some(path_var) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&path_var).any(|dir| is_executable(&dir.join(tool)))
}

fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };

    if !metadata.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    {
        true
    }
}

fn require_env(plan: &Plan, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if env_var_is_nonempty(name) {
        Ok(())
    } else {
        Err(Box::new(CompanyCiError::MissingEnv {
            plan: plan.name.clone(),
            name: name.to_string(),
        }))
    }
}

fn require_secret_env(plan: &Plan, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if env_var_is_nonempty(name) {
        Ok(())
    } else {
        Err(Box::new(CompanyCiError::MissingSecretEnv {
            plan: plan.name.clone(),
            name: name.to_string(),
        }))
    }
}

fn require_env_or_file(
    plan: &Plan,
    variable_name: &str,
    file_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if env_var_is_nonempty(variable_name) {
        return Ok(());
    }

    let file_path = env::var(file_name).unwrap_or_default();
    if file_path.trim().is_empty() {
        return Err(Box::new(CompanyCiError::MissingEnvOrFile {
            plan: plan.name.clone(),
            env_name: variable_name.to_string(),
            file_env_name: file_name.to_string(),
        }));
    }

    if Path::new(&file_path).is_file() {
        Ok(())
    } else {
        Err(Box::new(CompanyCiError::MissingEnvFile {
            plan: plan.name.clone(),
            name: file_name.to_string(),
            path: file_path,
        }))
    }
}

fn env_var_is_nonempty(name: &str) -> bool {
    env::var(name)
        .ok()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    struct RecordingRunner {
        checked_tools: RefCell<Vec<String>>,
        checked_env: RefCell<Vec<String>>,
        commands: RefCell<Vec<String>>,
    }

    impl RecordingRunner {
        fn new() -> Self {
            Self {
                checked_tools: RefCell::new(Vec::new()),
                checked_env: RefCell::new(Vec::new()),
                commands: RefCell::new(Vec::new()),
            }
        }
    }

    impl CommandRunner for RecordingRunner {
        fn check_tool(&self, _plan: &Plan, tool: &str) -> Result<(), Box<dyn std::error::Error>> {
            self.checked_tools.borrow_mut().push(tool.to_string());
            Ok(())
        }

        fn check_env(
            &self,
            _plan: &Plan,
            requirement: &EnvRequirement,
        ) -> Result<(), Box<dyn std::error::Error>> {
            self.checked_env
                .borrow_mut()
                .push(requirement.display_name());
            Ok(())
        }

        fn run(&self, step: &Step) -> Result<(), Box<dyn std::error::Error>> {
            self.commands.borrow_mut().push(step.command.join(" "));
            Ok(())
        }
    }

    #[test]
    fn run_plan_executes_all_steps_when_not_dry_run() {
        let runner = RecordingRunner::new();
        let plan = Plan::new(
            "sample",
            vec![
                Step {
                    description: "one".into(),
                    command: vec!["echo".into()],
                },
                Step {
                    description: "two".into(),
                    command: vec!["true".into()],
                },
            ],
        )
        .with_required_tools(["node", "npm"])
        .with_required_env([EnvRequirement::variable("SAMPLE_ENV")]);
        runner.run_plan(&plan, false).unwrap();
        assert_eq!(runner.checked_tools.borrow().as_slice(), ["node", "npm"]);
        assert_eq!(runner.checked_env.borrow().as_slice(), ["SAMPLE_ENV"]);
        assert_eq!(runner.commands.borrow().len(), 2);
    }

    #[test]
    fn run_plan_skips_execution_for_dry_run() {
        let runner = RecordingRunner::new();
        let plan = Plan::new(
            "sample",
            vec![Step {
                description: "one".into(),
                command: vec!["echo".into()],
            }],
        )
        .with_required_tools(["node"])
        .with_required_env([EnvRequirement::secret("SAMPLE_SECRET")]);
        runner.run_plan(&plan, true).unwrap();
        assert!(runner.checked_tools.borrow().is_empty());
        assert!(runner.checked_env.borrow().is_empty());
        assert!(runner.commands.borrow().is_empty());
    }
}
