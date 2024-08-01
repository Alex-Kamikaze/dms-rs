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
                println!(
                    "Generating empty dictionaries for languages {:?}",
                    &arguments.languages
                );
                let generate_result = generate_empty_dictionaries(arguments.dictionary_path, arguments.languages);
                match generate_result {
                    Ok(()) => { println!("Finished generating empty dictionaries");}
                    Err(_) => { println!("Error: Problem occured while generating empty dictionaries")}
                }
                
            }
            //TODO: Implement auto translation for static dictionaries with LibreTranslate
            TranslateType::Auto(arguments) => {}
        },
        TestRead(args) => {
            println!("{:?}", read_json_dictionary(&args.file_name))
        }
    }
    Ok(())
}
