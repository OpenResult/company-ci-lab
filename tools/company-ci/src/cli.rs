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
    Publish(PublishCommand),
    Image(ImageCommand),
    Deploy(DeployCommand),
    E2e(E2eCommand),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionArgs {
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishCommand {
    MavenLib(PublishArgs),
    NpmLib(NpmPublishArgs),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishArgs {
    pub path: String,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpmPublishArgs {
    pub path: String,
    pub tag: String,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub enum ImageCommand {
    Build(ExecutionArgs),
    Publish(ExecutionArgs),
}

#[derive(Debug, Clone)]
pub enum DeployCommand {
    Openshift(ExecutionArgs),
}

#[derive(Debug, Clone)]
pub enum E2eCommand {
    Openshift(ExecutionArgs),
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
        "publish" => parse_publish_command(&args[1..]),
        "image" => parse_image_command(&args[1..]),
        "deploy" => parse_deploy_command(&args[1..]),
        "e2e" => parse_e2e_command(&args[1..]),
        "help" | "--help" | "-h" => Err(CompanyCiError::Usage(usage())),
        other => Err(CompanyCiError::InvalidArgument(format!(
            "unknown command: {other}"
        ))),
    }
}

fn parse_publish_command(args: &[String]) -> Result<Command, CompanyCiError> {
    let Some(contract) = args.first().map(|value| value.as_str()) else {
        return Err(CompanyCiError::Usage(usage()));
    };

    match contract {
        "maven-lib" => {
            let parsed = parse_publish_args(&args[1..], false)?;
            Ok(Command::Publish(PublishCommand::MavenLib(PublishArgs {
                path: parsed.path,
                dry_run: parsed.dry_run,
            })))
        }
        "npm-lib" => {
            let parsed = parse_publish_args(&args[1..], true)?;
            Ok(Command::Publish(PublishCommand::NpmLib(NpmPublishArgs {
                path: parsed.path,
                tag: parsed.tag.unwrap_or_else(|| "ci".to_string()),
                dry_run: parsed.dry_run,
            })))
        }
        other => Err(CompanyCiError::InvalidArgument(format!(
            "unknown publish contract: {other}"
        ))),
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
        other => Err(CompanyCiError::InvalidArgument(format!(
            "unknown image command: {other}"
        ))),
    }
}

fn parse_deploy_command(args: &[String]) -> Result<Command, CompanyCiError> {
    let Some(subcommand) = args.first().map(|value| value.as_str()) else {
        return Err(CompanyCiError::Usage(usage()));
    };
    let parsed = parse_execution_args(&args[1..])?;
    match subcommand {
        "openshift" => Ok(Command::Deploy(DeployCommand::Openshift(parsed))),
        other => Err(CompanyCiError::InvalidArgument(format!(
            "unknown deploy command: {other}"
        ))),
    }
}

fn parse_e2e_command(args: &[String]) -> Result<Command, CompanyCiError> {
    let Some(subcommand) = args.first().map(|value| value.as_str()) else {
        return Err(CompanyCiError::Usage(usage()));
    };
    let parsed = parse_execution_args(&args[1..])?;
    match subcommand {
        "openshift" => Ok(Command::E2e(E2eCommand::Openshift(parsed))),
        other => Err(CompanyCiError::InvalidArgument(format!(
            "unknown e2e command: {other}"
        ))),
    }
}

struct ParsedPublishArgs {
    path: String,
    dry_run: bool,
    tag: Option<String>,
}

fn parse_publish_args(
    args: &[String],
    allow_tag: bool,
) -> Result<ParsedPublishArgs, CompanyCiError> {
    let mut path = None;
    let mut dry_run = false;
    let mut tag = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--dry-run" => dry_run = true,
            "--tag" if allow_tag => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(CompanyCiError::InvalidArgument(
                        "missing value for --tag".to_string(),
                    ));
                };
                if value.trim().is_empty() {
                    return Err(CompanyCiError::InvalidArgument(
                        "npm publish tag must not be empty".to_string(),
                    ));
                }
                tag = Some(value.clone());
            }
            "--tag" => {
                return Err(CompanyCiError::InvalidArgument(
                    "--tag is only supported for publish npm-lib".to_string(),
                ))
            }
            other if other.starts_with('-') => {
                return Err(CompanyCiError::InvalidArgument(format!(
                    "unknown argument: {other}"
                )))
            }
            other => {
                if path.is_some() {
                    return Err(CompanyCiError::InvalidArgument(format!(
                        "unexpected argument: {other}"
                    )));
                }
                path = Some(other.to_string());
            }
        }
        index += 1;
    }

    let Some(path) = path else {
        return Err(CompanyCiError::Usage(usage()));
    };

    Ok(ParsedPublishArgs { path, dry_run, tag })
}

fn parse_execution_args(args: &[String]) -> Result<ExecutionArgs, CompanyCiError> {
    let mut dry_run = false;
    for arg in args {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            other => {
                return Err(CompanyCiError::InvalidArgument(format!(
                    "unknown argument: {other}"
                )))
            }
        }
    }
    Ok(ExecutionArgs { dry_run })
}

fn usage() -> String {
    [
        "Usage:",
        "  company-ci verify [--dry-run]",
        "  company-ci build [--dry-run]",
        "  company-ci test [--dry-run]",
        "  company-ci package [--dry-run]",
        "  company-ci publish maven-lib <path> [--dry-run]",
        "  company-ci publish npm-lib <path> [--dry-run] [--tag <dist-tag>]",
        "  company-ci image build [--dry-run]",
        "  company-ci image publish [--dry-run]",
        "  company-ci deploy openshift [--dry-run]",
        "  company-ci e2e openshift [--dry-run]",
        "Commands:",
        "  verify",
        "  build",
        "  test",
        "  package",
        "  publish maven-lib <path>",
        "  publish npm-lib <path> [--tag <dist-tag>]",
        "  image build|publish",
        "  deploy openshift",
        "  e2e openshift",
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

    #[test]
    fn parses_maven_publish_target_with_dry_run() {
        let cli = Cli::parse([
            "publish".into(),
            "maven-lib".into(),
            "libs/java-lib".into(),
            "--dry-run".into(),
        ])
        .unwrap();
        match cli.command {
            Command::Publish(PublishCommand::MavenLib(args)) => {
                assert_eq!(args.path, "libs/java-lib");
                assert!(args.dry_run);
            }
            _ => panic!("expected maven-lib publish"),
        }
    }

    #[test]
    fn parses_npm_publish_target_with_tag() {
        let cli = Cli::parse([
            "publish".into(),
            "npm-lib".into(),
            "libs/node-lib".into(),
            "--tag".into(),
            "beta".into(),
            "--dry-run".into(),
        ])
        .unwrap();
        match cli.command {
            Command::Publish(PublishCommand::NpmLib(args)) => {
                assert_eq!(args.path, "libs/node-lib");
                assert_eq!(args.tag, "beta");
                assert!(args.dry_run);
            }
            _ => panic!("expected npm-lib publish"),
        }
    }

    #[test]
    fn rejects_tag_for_maven_publish() {
        let error = Cli::parse([
            "publish".into(),
            "maven-lib".into(),
            "libs/java-lib".into(),
            "--tag".into(),
            "ci".into(),
        ])
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "--tag is only supported for publish npm-lib"
        );
    }

    #[test]
    fn publish_requires_contract() {
        let error = Cli::parse(["publish".into()]).unwrap_err();
        assert!(matches!(error, CompanyCiError::Usage(_)));
    }

    #[test]
    fn publish_requires_path() {
        let error = Cli::parse(["publish".into(), "maven-lib".into()]).unwrap_err();
        assert!(matches!(error, CompanyCiError::Usage(_)));
    }

    #[test]
    fn rejects_kubernetes_deploy_command() {
        let error = Cli::parse(["deploy".into(), "kubernetes".into()]).unwrap_err();
        assert_eq!(error.to_string(), "unknown deploy command: kubernetes");
    }

    #[test]
    fn rejects_legacy_env_command() {
        let error = Cli::parse(["env".into(), "up".into(), "repository".into()]).unwrap_err();
        assert_eq!(error.to_string(), "unknown command: env");
    }

    #[test]
    fn rejects_legacy_openshift_local_e2e_command() {
        let error = Cli::parse(["e2e".into(), "openshift-local".into()]).unwrap_err();
        assert_eq!(error.to_string(), "unknown e2e command: openshift-local");
    }

    #[test]
    fn rejects_unknown_publish_contract() {
        let error =
            Cli::parse(["publish".into(), "python-lib".into(), "libs/example".into()]).unwrap_err();
        assert_eq!(error.to_string(), "unknown publish contract: python-lib");
    }
}
