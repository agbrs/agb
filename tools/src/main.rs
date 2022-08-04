use clap::{command, Parser};

mod publish;

#[derive(Parser, Debug)]
struct PublishAllCratesArgs {}

fn main() {
    let matches = command!()
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(clap::Command::new("publish").about("Publishes agb and all subcrates"))
        .get_matches();

    match matches.subcommand() {
        Some(("publish", _)) => publish::publish(),
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}
