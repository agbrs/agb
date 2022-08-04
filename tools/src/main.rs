use clap::Command;

mod publish;

fn main() {
    let matches = Command::new("Agb tools")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(clap::Command::new("publish").about("Publishes agb and all subcrates"))
        .get_matches();

    let result = match matches.subcommand() {
        Some(("publish", _)) => publish::publish(),
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    };

    if let Err(e) = result {
        eprintln!("Error: {:?}", e);
    }
}
