#![allow(unused_imports)]

use api::types::Word;
use clap::Parser;
use tokio::*;

mod args;
use args::cli_args::*;
use crate::CliSubcommands::Translate;



#[tokio::main]
async fn main() -> Result<(), reqwest::Error>{
    let args = Cli::parse();
    match args.subcommand {
        Translate(translate_type) => {
            match translate_type {
                TranslateType::Manual(arguments) => { println!("Generating empty dictionaries for languages {:?}", arguments.languages)},
                TranslateType::Auto(arguments) => { println!("Using auto translation for languages {:?} with {} API", arguments.languages, arguments.translator_api)},
            }
        }
    }
    Ok(())
}
