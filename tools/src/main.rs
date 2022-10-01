#![deny(clippy::all)]
use clap::Command;

mod publish;

fn cli() -> Command {
    Command::new("Agb tools")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(publish::command())
}

fn main() {
    let matches = cli().get_matches();

    let result = match matches.subcommand() {
        Some(("publish", arg_matches)) => publish::publish(arg_matches),
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
