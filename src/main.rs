use price_backfill::{operations::{historical, last, schedule, delete, stats}};
use env_logger;

fn main() -> () {
    env_logger::init();
    let matches = clap::Command::new("prices")
        .bin_name("prices")
        .subcommand_required(true)
        .subcommand(clap::command!("last")
            .about("Retrieve last prices for all symbols"))
        .subcommand(clap::command!("historical")
            .about("Retrieve all historical for all symbols"))
        .subcommand(clap::command!("schedule")
            .about("Schedule all hard-coded schedules"))
        .subcommand(clap::command!("delete")
            .about("Delete all prices from database"))
        .subcommand(clap::command!("stats")
            .about("List how many prices are stored"))
        .get_matches();
    match matches.subcommand() {
        Some(("last", _sub_matches)) => last(),
        Some(("historical", _sub_matches)) => historical(),
        Some(("schedule", _sub_matches)) => schedule(),
        Some(("stats", _sub_matches)) => stats(),
        Some(("delete", _sub_matches)) => delete(),
        _ => unreachable!(),
    }
}
