use clap::{Args, ValueEnum};
use dash7::{
    app::command::Command,
    file::{File, FileId},
    link::{BackgroundFrame, ForegroundFrame}, transport::serial::SerialFrame,
};
use deku::DekuError;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ParseType {
    #[clap(alias = "fg")]
    Foreground,
    #[clap(alias = "bg")]
    Background,
    #[clap(alias = "a")]
    Alp,
    #[clap(alias = "e")]
    Serial,
    #[clap(alias = "s")]
    Systemfile,
}

#[derive(Debug, Args)]
pub struct ParseArgs {
    #[arg(value_enum, short = 't', long = "type")]
    parse_type: Option<ParseType>,

    #[arg(short = 'f')]
    file_id: Option<u8>,

    #[arg()]
    hex: String,
}

fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

fn parse_foreground_frame(input: &[u8]) -> Result<(), DekuError> {
    let frame = ForegroundFrame::try_from(input)?;
    println!("{:?}", frame);
    Ok(())
}

fn parse_background_frame(input: &[u8]) -> Result<(), DekuError> {
    let frame = BackgroundFrame::try_from(input)?;
    println!("{:?}", frame);
    Ok(())
}

fn parse_alp_command(input: &[u8]) -> Result<(), DekuError> {
    let command = Command::try_from(input)?;
    println!("{}", command);
    Ok(())
}

fn parse_serial(input: &[u8]) -> Result<(), DekuError> {
    let frame = SerialFrame::try_from(input)?;
    println!("{:?}", frame);
    Ok(())
}

fn parse_file(input: &[u8], file_id: FileId) -> Result<(), DekuError> {
    let file = File::from_bytes((input, 0), file_id, 0)?;
    println!("{:?}", file);
    Ok(())
}

fn parse_any_file(input: &[u8]) -> Result<(), DekuError> {
    for file_id_raw in 0..=0x2Eu8 {
        let file_id: FileId = file_id_raw.try_into()?;
        if parse_file(input, file_id).is_ok() {
            return Ok(());
        }
    }
    Ok(())
}

fn parse_any(input: &[u8]) -> Result<(), DekuError> {
    if parse_foreground_frame(input).is_ok() {
        return Ok(());
    }
    if parse_background_frame(input).is_ok() {
        return Ok(());
    }
    if parse_alp_command(input).is_ok() {
        return Ok(());
    }
    if parse_serial(input).is_ok() {
        return Ok(());
    }
    if parse_any_file(input).is_ok() {
        return Ok(());
    }

    Err(DekuError::Parse("Could not parse input".to_string()))
}

pub fn main(args: ParseArgs) {
    let input_vec = hex::decode(remove_whitespace(&args.hex)).expect("Could not parse input hex");
    let input = input_vec.as_slice();

    match args.parse_type {
        Some(ParseType::Foreground) => {
            parse_foreground_frame(input).expect("Could not foreground frame")
        }
        Some(ParseType::Background) => {
            parse_background_frame(input).expect("Could not background frame")
        }
        Some(ParseType::Alp) => parse_alp_command(input).expect("Could not parse command"),
        Some(ParseType::Serial) => parse_serial(input).expect("Could not parse serial"),
        Some(ParseType::Systemfile) => {
            if let Some(file_id_raw) = args.file_id {
                let file_id: FileId = file_id_raw.try_into().expect("File id invalid");
                parse_file(input, file_id).expect("Could not parse file")
            } else {
                parse_any_file(input).expect("Could not parse file")
            }
        }
        None => parse_any(input).expect("Could not parse input"),
    }
}
