pub mod cli_args {
    use clap::{builder::Str, Args, Parser, Subcommand};
    
    #[derive(Parser, Debug)]
    #[clap(version = "1", about = "Utility for auto-generating JSON dictionaries with wide range of translating APIs", long_about = None)]
    pub struct Cli {
        #[clap(subcommand)]
        pub subcommand: CliSubcommands
    }

    #[derive(Debug, Subcommand)]
    pub enum CliSubcommands {
        #[clap(subcommand)]
        /// Translate text from dictionaries
        Translate(TranslateType)
    }


    #[derive(Debug, Subcommand)]
    pub enum TranslateType {
        /// Create an empty dictionaries for manual translation
        Manual(ManualTranslationArgs),

        /// Generate empty dictionaries and fill them with translated words from basic dictionaries by translation API's
        Auto(AutoTranslationArgs)
    }

    #[derive(Debug, Args)]
    pub struct ManualTranslationArgs {
        /// Target languages
        pub languages: Vec<String>
    }

    #[derive(Debug, Args)]
    pub struct AutoTranslationArgs {
        /// API that is used to translate words
        pub translator_api: String,
        /// Target languages
        pub languages: Vec<String>
    }
}