#![allow(unused_imports)]

use api::parser::*;
use api::types::Word;
use clap::Parser;
use tokio::*;

mod args;
use crate::CliSubcommands::*;
use args::cli_args::*;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args = Cli::parse();
    match args.subcommand {
        Translate(translate_type) => match translate_type {
            TranslateType::Manual(arguments) => {
                println!("Generating empty dictionaries for languages {:?}", &arguments.languages);
                generate_empty_dictionaries(arguments.dictionary_path, arguments.languages);
                println!("Finished generating empty dictionaries");
            }
            TranslateType::Auto(arguments) => {}
        },
        TestRead(args) => {
            println!("{:?}", read_json_dictionary(&args.file_name))
        }
    }
    Ok(())
}
