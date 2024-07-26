#![allow(unused_imports)]

use api::types::Word;
use api::parser::read_json_dictionary;
use clap::Parser;
use tokio::*;

mod args;
use crate::CliSubcommands::Translate;
use args::cli_args::*;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args = Cli::parse();
    match args.subcommand {
        Translate(translate_type) => match translate_type {
            TranslateType::Manual(arguments) => {}
            TranslateType::Auto(arguments) => {}
        },
        CliSubcommands::TestRead(args) => {
            println!("{:?}", read_json_dictionary(&args.file_name))
        },
    }
    Ok(())
}
