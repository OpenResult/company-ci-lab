use crate::error::CompanyCiError;

#[derive(Debug, Clone)]
pub struct Cli {
    pub command: Command,
}

#[derive(Debug, Clone)]
pub enum Command {
    Verify(ExecutionArgs),
    Build(ExecutionArgs),
    Test(ExecutionArgs),
    Package(ExecutionArgs),
    Publish(ExecutionArgs),
    Image(ImageCommand),
    Deploy(DeployCommand),
    Env(EnvCommand),
    E2e(E2eCommand),
}

#[derive(Debug, Clone, Copy)]
pub struct ExecutionArgs {
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub enum ImageCommand {
    Build(ExecutionArgs),
    Publish(ExecutionArgs),
}

#[derive(Debug, Clone)]
pub enum DeployCommand {
    Kubernetes(ExecutionArgs),
    Openshift(ExecutionArgs),
}

#[derive(Debug, Clone)]
pub enum EnvCommand {
    Up(EnvironmentTarget, ExecutionArgs),
    Down(EnvironmentTarget, ExecutionArgs),
}

#[derive(Debug, Clone)]
pub enum EnvironmentTarget {
    Kind,
    Nexus,
}

#[derive(Debug, Clone)]
pub enum E2eCommand {
    Emulated(ExecutionArgs),
    OpenshiftLocal(ExecutionArgs),
}

impl Cli {
    pub fn parse<I>(args: I) -> Result<Self, CompanyCiError>
    where
        I: IntoIterator<Item = String>,
    {
        let args: Vec<String> = args.into_iter().collect();
        let command = parse_command(&args)?;
        Ok(Self { command })
    }
}

fn parse_command(args: &[String]) -> Result<Command, CompanyCiError> {
    let Some(first) = args.first().map(|value| value.as_str()) else {
        return Err(CompanyCiError::Usage(usage()));
    };

    match first {
        "verify" => Ok(Command::Verify(parse_execution_args(&args[1..])?)),
        "build" => Ok(Command::Build(parse_execution_args(&args[1..])?)),
        "test" => Ok(Command::Test(parse_execution_args(&args[1..])?)),
        "package" => Ok(Command::Package(parse_execution_args(&args[1..])?)),
        "publish" => Ok(Command::Publish(parse_execution_args(&args[1..])?)),
        "image" => parse_image_command(&args[1..]),
        "deploy" => parse_deploy_command(&args[1..]),
        "env" => parse_env_command(&args[1..]),
        "e2e" => parse_e2e_command(&args[1..]),
        "help" | "--help" | "-h" => Err(CompanyCiError::Usage(usage())),
        other => Err(CompanyCiError::InvalidArgument(format!("unknown command: {other}"))),
    }
}

fn parse_image_command(args: &[String]) -> Result<Command, CompanyCiError> {
    let Some(subcommand) = args.first().map(|value| value.as_str()) else {
        return Err(CompanyCiError::Usage(usage()));
    };
    let parsed = parse_execution_args(&args[1..])?;
    match subcommand {
        "build" => Ok(Command::Image(ImageCommand::Build(parsed))),
        "publish" => Ok(Command::Image(ImageCommand::Publish(parsed))),
        other => Err(CompanyCiError::InvalidArgument(format!("unknown image command: {other}"))),
    }
}

fn parse_deploy_command(args: &[String]) -> Result<Command, CompanyCiError> {
    let Some(subcommand) = args.first().map(|value| value.as_str()) else {
        return Err(CompanyCiError::Usage(usage()));
    };
    let parsed = parse_execution_args(&args[1..])?;
    match subcommand {
        "kubernetes" => Ok(Command::Deploy(DeployCommand::Kubernetes(parsed))),
        "openshift" => Ok(Command::Deploy(DeployCommand::Openshift(parsed))),
        other => Err(CompanyCiError::InvalidArgument(format!("unknown deploy command: {other}"))),
    }
}

fn parse_env_command(args: &[String]) -> Result<Command, CompanyCiError> {
    if args.len() < 2 {
        return Err(CompanyCiError::Usage(usage()));
    }
    let action = args[0].as_str();
    let target = match args[1].as_str() {
        "kind" => EnvironmentTarget::Kind,
        "nexus" => EnvironmentTarget::Nexus,
        other => return Err(CompanyCiError::InvalidArgument(format!("unknown env target: {other}"))),
    };
    let parsed = parse_execution_args(&args[2..])?;
    match action {
        "up" => Ok(Command::Env(EnvCommand::Up(target, parsed))),
        "down" => Ok(Command::Env(EnvCommand::Down(target, parsed))),
        other => Err(CompanyCiError::InvalidArgument(format!("unknown env action: {other}"))),
    }
}

fn parse_e2e_command(args: &[String]) -> Result<Command, CompanyCiError> {
    let Some(subcommand) = args.first().map(|value| value.as_str()) else {
        return Err(CompanyCiError::Usage(usage()));
    };
    let parsed = parse_execution_args(&args[1..])?;
    match subcommand {
        "emulated" => Ok(Command::E2e(E2eCommand::Emulated(parsed))),
        "openshift-local" => Ok(Command::E2e(E2eCommand::OpenshiftLocal(parsed))),
        other => Err(CompanyCiError::InvalidArgument(format!("unknown e2e command: {other}"))),
    }
}

fn parse_execution_args(args: &[String]) -> Result<ExecutionArgs, CompanyCiError> {
    let mut dry_run = false;
    for arg in args {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            other => return Err(CompanyCiError::InvalidArgument(format!("unknown argument: {other}"))),
        }
    }
    Ok(ExecutionArgs { dry_run })
}

fn usage() -> String {
    [
        "Usage: company-ci <command> [--dry-run]",
        "Commands:",
        "  verify",
        "  build",
        "  test",
        "  package",
        "  publish",
        "  image build|publish",
        "  deploy kubernetes|openshift",
        "  env up|down kind|nexus",
        "  e2e emulated|openshift-local",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_command_with_dry_run() {
        let cli = Cli::parse(["image".into(), "build".into(), "--dry-run".into()]).unwrap();
        match cli.command {
            Command::Image(ImageCommand::Build(args)) => assert!(args.dry_run),
            _ => panic!("expected image build"),
        }
    }
}
