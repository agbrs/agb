use clap::{Arg, ArgAction};

pub fn command() -> clap::Command {
    clap::Command::new("deploy").arg(
        Arg::new("Dry run")
            .long("dry-run")
            .help("Don't actually deploy")
            .action(ArgAction::SetTrue),
    )
}
