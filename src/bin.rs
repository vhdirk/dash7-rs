use clap::{Args, Parser, Subcommand};
use lib::{
    app::command::Command,
    file::{File, FileId},
    link::{BackgroundFrame, ForegroundFrame},
};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Parse(ParseArgs),
}

#[derive(Debug, Args, Clone)]
#[group(required = true, multiple = false)]
struct ParseTypeArgs {
    #[clap(long = "fg", short = 'f', action)]
    dll_foreground: bool,

    #[clap(long = "bg", short = 'b', action)]
    dll_background: bool,

    #[clap(long = "alp", short = 'a', action)]
    alp: bool,

    #[clap(long = "serial", short = 's', action)]
    serial: bool,

    #[clap(long = "systemfile", short = 'g', action)]
    systemfile: Option<u8>,
}

#[derive(Debug, Args)]
struct ParseArgs {
    #[command(flatten)]
    parse_type: ParseTypeArgs,

    #[arg()]
    hex: String,
}

fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

fn parse(args: ParseArgs) {
    let input = hex::decode(remove_whitespace(&args.hex)).expect("Could not parse input jex");

    if args.parse_type.dll_foreground {
        let frame =
            ForegroundFrame::try_from(input.as_slice()).expect("Could not foreground frame");
        println!("{:?}", frame);
        return;
    } else if args.parse_type.dll_background {
        let frame =
            BackgroundFrame::try_from(input.as_slice()).expect("Could not background frame");
        println!("{:?}", frame);
        return;
    } else if args.parse_type.alp {
        let command = Command::try_from(input.as_slice()).expect("Could not parse command");
        println!("{}", command);
        return;
    } else if args.parse_type.serial {
        unimplemented!();
    } else if let Some(file_id_raw) = args.parse_type.systemfile {
        let file_id: FileId = file_id_raw.try_into().expect("File id invalid");
        let file =
            File::from_bytes((input.as_slice(), 0), file_id, 0).expect("Could not parse file");
        println!("{:?}", file);
        return;
    }
}

pub fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse(args) => parse(args),
    }
}