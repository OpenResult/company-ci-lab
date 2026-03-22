use crate::cli::{
    Cli, Command, DeployCommand, E2eCommand, EnvCommand, EnvironmentTarget, ImageCommand,
    PublishCommand,
};
use crate::context::ExecutionContext;
use crate::plan;
use crate::runner::CommandRunner;

pub fn dispatch(cli: Cli, runner: &impl CommandRunner) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Command::Publish(command) => match command {
            PublishCommand::MavenLib(args) => {
                let plan = plan::publish_maven_lib_plan(&args.path)?;
                runner.run_plan(&plan, args.dry_run)
            }
            PublishCommand::NpmLib(args) => {
                let plan = plan::publish_npm_lib_plan(&args.path, &args.tag)?;
                runner.run_plan(&plan, args.dry_run)
            }
        },
        command => {
            let context = ExecutionContext::detect()?;
            match command {
                Command::Verify(args) => {
                    runner.run_plan(&plan::verify_plan(&context), args.dry_run)
                }
                Command::Build(args) => runner.run_plan(&plan::build_plan(&context), args.dry_run),
                Command::Test(args) => runner.run_plan(&plan::test_plan(&context), args.dry_run),
                Command::Package(args) => {
                    runner.run_plan(&plan::package_plan(&context), args.dry_run)
                }
                Command::Image(command) => match command {
                    ImageCommand::Build(args) => {
                        runner.run_plan(&plan::image_build_plan(&context), args.dry_run)
                    }
                    ImageCommand::Publish(args) => {
                        runner.run_plan(&plan::image_publish_plan(&context), args.dry_run)
                    }
                },
                Command::Deploy(command) => match command {
                    DeployCommand::Kubernetes(args) => {
                        runner.run_plan(&plan::deploy_kubernetes_plan(&context), args.dry_run)
                    }
                    DeployCommand::Openshift(args) => {
                        runner.run_plan(&plan::deploy_openshift_plan(&context), args.dry_run)
                    }
                },
                Command::Env(command) => match command {
                    EnvCommand::Up(platform, args) => match platform {
                        EnvironmentTarget::Kind => {
                            runner.run_plan(&plan::env_up_kind_plan(&context), args.dry_run)
                        }
                        EnvironmentTarget::Nexus => {
                            runner.run_plan(&plan::env_up_nexus_plan(&context), args.dry_run)
                        }
                    },
                    EnvCommand::Down(platform, args) => match platform {
                        EnvironmentTarget::Kind => {
                            runner.run_plan(&plan::env_down_kind_plan(&context), args.dry_run)
                        }
                        EnvironmentTarget::Nexus => {
                            runner.run_plan(&plan::env_down_nexus_plan(&context), args.dry_run)
                        }
                    },
                },
                Command::E2e(command) => match command {
                    E2eCommand::Emulated(args) => {
                        runner.run_plan(&plan::e2e_emulated_plan(&context), args.dry_run)
                    }
                    E2eCommand::OpenshiftLocal(args) => {
                        runner.run_plan(&plan::e2e_openshift_local_plan(&context), args.dry_run)
                    }
                },
                Command::Publish(_) => unreachable!("publish is handled before context detection"),
            }
        }
    }
}
