use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;

mod parse;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Command to run
    #[clap(subcommand)]
    command: Commands,

    /// Control command verbosity
    #[command(flatten)]
    verbose: Verbosity,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Parse a hex string
    Parse(parse::ParseArgs),
}

#[quit::main]
pub fn main() {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    match cli.command {
        Commands::Parse(args) => parse::main(args),
    }
}
