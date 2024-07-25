#![allow(unused_imports)]

use api::types::Word;
use clap::Parser;
use tokio::*;

mod args;
use args::cli_args::Cli;



#[tokio::main]
async fn main() {
    let args = Cli::parse();
    println!("Utility launched successfully");
}
