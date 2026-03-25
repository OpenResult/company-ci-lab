use crate::cli::{Cli, Command, DeployCommand, E2eCommand, ImageCommand, PublishCommand};
use crate::context::ExecutionContext;
use crate::plan;
use crate::runner::CommandRunner;

pub fn dispatch(cli: Cli, runner: &impl CommandRunner) -> Result<(), Box<dyn std::error::Error>> {
    let context = ExecutionContext::detect()?;
    match cli.command {
        Command::Publish(command) => match command {
            PublishCommand::MavenLib(args) => {
                let plan = plan::publish_maven_lib_plan(&context.repo_layout, &args.path)?;
                runner.run_plan(&plan, args.dry_run)
            }
            PublishCommand::NpmLib(args) => {
                let plan = plan::publish_npm_lib_plan(&context.repo_layout, &args.path, &args.tag)?;
                runner.run_plan(&plan, args.dry_run)
            }
        },
        Command::Verify(args) => runner.run_plan(&plan::verify_plan(&context), args.dry_run),
        Command::Build(args) => runner.run_plan(&plan::build_plan(&context), args.dry_run),
        Command::Test(args) => runner.run_plan(&plan::test_plan(&context), args.dry_run),
        Command::Package(args) => runner.run_plan(&plan::package_plan(&context), args.dry_run),
        Command::Image(command) => match command {
            ImageCommand::Build(args) => {
                runner.run_plan(&plan::image_build_plan(&context), args.dry_run)
            }
            ImageCommand::Publish(args) => {
                let plan = plan::image_publish_plan(&context)?;
                runner.run_plan(&plan, args.dry_run)
            }
        },
        Command::Deploy(command) => match command {
            DeployCommand::Openshift(args) => {
                let plan = plan::deploy_openshift_plan(&context)?;
                runner.run_plan(&plan, args.dry_run)
            }
        },
        Command::E2e(command) => match command {
            E2eCommand::Openshift(args) => {
                let plan = plan::e2e_openshift_plan(&context)?;
                runner.run_plan(&plan, args.dry_run)
            }
        },
    }
}
