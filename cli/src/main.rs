#![allow(unused_imports)]

use api::file_system::init_new_dictionary_system;
use api::parser::*;
use api::static_translate::generate_empty_dictionaries_from_static_basic;
use api::types::Word;
use clap::Parser;
use tokio::*;

mod args;
use crate::CliSubcommands::*;
use args::cli_args::*;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args = TranslatorCli::parse();
    match args.subcommand {
        Translate(translate_type) => match translate_type {
            TranslateType::Manual(arguments) => {
                println!(
                    "Generating empty dictionaries for languages {:?}",
                    &arguments.languages
                );
                let generate_result = generate_empty_dictionaries_from_static_basic(
                    &arguments.dictionary_path,
                    arguments.languages,
                );
                match generate_result {
                    Ok(()) => {
                        println!("Finished generating empty dictionaries");
                    }
                    Err(_) => {
                        println!("Error: Problem occured while generating empty dictionaries")
                    }
                }
            }
            //TODO: Implement auto translation for static dictionaries with LibreTranslate
            TranslateType::Auto(arguments) => {}
        },

        Init(args) => match init_new_dictionary_system(args.directory, args.basic_language) {
            Ok(()) => {
                println!("New dictionary system is initialized successfully")
            }
            Err(err) => {
                println!("{}", err)
            }
        },
    }
    Ok(())
}
