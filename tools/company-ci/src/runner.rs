use crate::error::CompanyCiError;
use crate::plan::{Plan, Step};
use std::process::Command;

pub trait CommandRunner {
    fn run(&self, step: &Step) -> Result<(), Box<dyn std::error::Error>>;
    fn run_plan(&self, plan: &Plan, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
        for step in &plan.steps {
            if dry_run {
                println!("[dry-run] {} => {}", step.description, step.command.join(" "));
            } else {
                self.run(step)?;
            }
        }
        Ok(())
    }
}

pub struct ShellRunner;

impl CommandRunner for ShellRunner {
    fn run(&self, step: &Step) -> Result<(), Box<dyn std::error::Error>> {
        let (program, args) = step.command.split_first().expect("step command must not be empty");
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    struct RecordingRunner { commands: RefCell<Vec<String>> }
    impl RecordingRunner { fn new() -> Self { Self { commands: RefCell::new(Vec::new()) } } }
    impl CommandRunner for RecordingRunner {
        fn run(&self, step: &Step) -> Result<(), Box<dyn std::error::Error>> {
            self.commands.borrow_mut().push(step.command.join(" "));
            Ok(())
        }
    }
    #[test]
    fn run_plan_executes_all_steps_when_not_dry_run() {
        let runner = RecordingRunner::new();
        let plan = Plan::new("sample", vec![Step { description: "one".into(), command: vec!["echo".into()] }, Step { description: "two".into(), command: vec!["true".into()] }]);
        runner.run_plan(&plan, false).unwrap();
        assert_eq!(runner.commands.borrow().len(), 2);
    }
    #[test]
    fn run_plan_skips_execution_for_dry_run() {
        let runner = RecordingRunner::new();
        let plan = Plan::new("sample", vec![Step { description: "one".into(), command: vec!["echo".into()] }]);
        runner.run_plan(&plan, true).unwrap();
        assert!(runner.commands.borrow().is_empty());
    }
}
