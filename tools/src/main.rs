#![deny(clippy::all)]
use clap::Command;

mod publish;
mod release;
mod utils;

#[derive(Debug)]
pub enum Error {
    PublishError(publish::Error),
    ReleaseError(release::Error),
}

fn cli() -> Command {
    Command::new("Agb tools")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(publish::command())
        .subcommand(release::command())
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

        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    };

    if let Err(e) = result {
        eprintln!("Error: {:?}", e);
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
