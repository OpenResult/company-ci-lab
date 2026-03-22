use crate::error::CompanyCiError;
use crate::plan::{Plan, Step};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

pub trait CommandRunner {
    fn check_tool(&self, plan: &Plan, tool: &str) -> Result<(), Box<dyn std::error::Error>>;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    struct RecordingRunner {
        checked_tools: RefCell<Vec<String>>,
        commands: RefCell<Vec<String>>,
    }

    impl RecordingRunner {
        fn new() -> Self {
            Self {
                checked_tools: RefCell::new(Vec::new()),
                commands: RefCell::new(Vec::new()),
            }
        }
    }

    impl CommandRunner for RecordingRunner {
        fn check_tool(&self, _plan: &Plan, tool: &str) -> Result<(), Box<dyn std::error::Error>> {
            self.checked_tools.borrow_mut().push(tool.to_string());
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
        .with_required_tools(["node", "npm"]);
        runner.run_plan(&plan, false).unwrap();
        assert_eq!(runner.checked_tools.borrow().as_slice(), ["node", "npm"]);
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
        .with_required_tools(["node"]);
        runner.run_plan(&plan, true).unwrap();
        assert!(runner.checked_tools.borrow().is_empty());
        assert!(runner.commands.borrow().is_empty());
    }
}
