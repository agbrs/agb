use clap::{Arg, ArgAction, ArgMatches};

pub fn command() -> clap::Command {
    clap::Command::new("deploy").arg(
        Arg::new("Dry run")
            .long("dry-run")
            .help("Don't actually deploy")
            .action(ArgAction::SetTrue),
    )
}

pub fn deploy(matches: &ArgMatches) -> Result<(), Error> {
    Ok(())
}

#[derive(Debug)]
pub enum Error {}
