use clap::{Args, Parser, Subcommand};
use lib::app::command::Command;

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
    systemfile: bool,
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
    let input =  hex::decode(remove_whitespace(&args.hex)).expect("Could not parse input jex");

    if args.parse_type.dll_foreground {
    } else if args.parse_type.dll_background {
    } else if args.parse_type.alp {
        let command = Command::try_from(input.as_slice()).expect("Could not parse command");
        println!("{:?}", command);
        return;
    } else if args.parse_type.serial {
        unimplemented!();
    } else if args.parse_type.systemfile {
        unimplemented!();
    }
}

pub fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse(args) => parse(args),
    }
}

// from d7a.alp.parser import Parser as AlpParser
// from d7a.dll.parser import Parser as DllFrameParser, FrameType
// from d7a.serial_modem_interface.parser import Parser as SerialParser
// from d7a.system_files.system_file_ids import SystemFileIds
// from d7a.system_files.system_files import SystemFiles

// parser_types = ["fg", "bg", "alp", "serial", "systemfile"]
// argparser = argparse.ArgumentParser()
// argparser.add_argument("-t", "--type", choices=parser_types, required=True)
// argparser.add_argument("-f", "--file-id", help="the ID of the system file to parse", type=int)
// argparser.add_argument('data', help="The data to be parsed, input as an hexstring")
// args = argparser.parse_args()

// hexstring = args.data.strip().replace(' ', '')
// data = bytearray(hexstring.decode("hex"))
// if args.type == "alp":
//   print AlpParser().parse(ConstBitStream(data), len(data))
//   exit(0)
// if args.type == "serial":
//   parser = SerialParser()
// if args.type == "fg":
//   parser = DllFrameParser(frame_type=FrameType.FOREGROUND)
// if args.type == "bg":
//   parser = DllFrameParser(frame_type=FrameType.BACKGROUND)
// if args.type == "systemfile":
//   file = SystemFileIds(args.file_id)
//   file_type = SystemFiles().files[file]
//   print(file_type.parse(ConstBitStream(data)))
//   exit(0)

// msgtype, cmds, info = parser.parse(data)
// for cmd in cmds:
//   print cmd

// print info
