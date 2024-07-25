#![allow(unused_imports)]

use api::types::Word;
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
            TranslateType::Manual(arguments) => {
                todo!()
            }
            TranslateType::Auto(arguments) => {
                todo!()
            }
        },
    }
    Ok(())
}
