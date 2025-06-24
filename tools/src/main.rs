#![deny(clippy::all)]
use clap::Command;

mod configure;
mod deploy;
mod publish;
mod release;
mod utils;

#[derive(Debug)]
pub enum Error {
    PublishError(publish::Error),
    ReleaseError(release::Error),
    DeployError(deploy::Error),
    ConfigureError(configure::Error),
}

fn cli() -> Command {
    Command::new("Agb tools")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(publish::command())
        .subcommand(release::command())
        .subcommand(deploy::command())
        .subcommand(configure::command())
}

fn main() {
    let matches = cli().get_matches();

    let result = match matches.subcommand() {
        Some(("publish", arg_matches)) => {
            publish::publish(arg_matches).map_err(Error::PublishError)
        }

        Some(("release", arg_matches)) => {
            release::release(arg_matches).map_err(Error::ReleaseError)
        }

        Some(("deploy", arg_matches)) => deploy::deploy(arg_matches).map_err(Error::DeployError),

        Some(("configure", arg_matches)) => {
            configure::configure(arg_matches).map_err(Error::ConfigureError)
        }

        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    };

    if let Err(e) = result {
        eprintln!("Error: {e:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        cli().debug_assert();
    }
}
