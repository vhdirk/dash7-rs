use clap::{Parser, Subcommand};

mod parse;
use clap_verbosity_flag::Verbosity;
use parse::{parse, ParseArgs};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    #[command(flatten)]
    verbose: Verbosity,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Parse a hex string
    Parse(ParseArgs),
}

pub fn main() {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    match cli.command {
        Commands::Parse(args) => parse(args),
    }
}
