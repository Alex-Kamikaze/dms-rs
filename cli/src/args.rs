pub mod cli_args {
    use clap::{builder::Str, Args, Parser, Subcommand};

    #[derive(Parser, Debug)]
    #[clap(version = "0.1 Experimental", about = "Utility for auto-generating JSON dictionaries with wide range of translating APIs", long_about = None)]
    #[doc = "Parser for CLI arguments"]
    pub struct TranslatorCli {
        #[clap(subcommand)]
        pub subcommand: CliSubcommands,
    }

    #[derive(Debug, Subcommand)]
    pub enum CliSubcommands {
        #[clap(subcommand)]
        /// Translate text from dictionaries
        Translate(TranslateType),
        /// Initialize new dictionaries system
        Init(InitializeArguments),
    }

    #[derive(Debug, Subcommand)]
    #[doc = "Subcommand for providing type of translation"]
    pub enum TranslateType {
        /// Create an empty dictionaries for manual translation
        Manual(ManualTranslationArgs),

        /// Generate empty dictionaries and fill them with translated words from basic dictionaries by translation API's
        Auto(AutoTranslationArgs),
    }

    #[derive(Debug, Args)]
    #[doc = "Arguments for manual translation feature"]
    pub struct ManualTranslationArgs {
        /// Directory where dictionaries are stored
        pub dictionary_path: String,
        /// Target languages
        pub languages: Vec<String>,
    }

    #[derive(Debug, Args)]
    #[doc = "Arguments for automatic translation feature"]
    pub struct AutoTranslationArgs {
        /// API that is used to translate words
        // TODO: Replace with seperate implementations of API's
        pub translator_api: String,
        /// Target languages
        pub languages: Vec<String>,
    }

    #[derive(Debug, Args)]
    pub struct InitializeArguments {
        pub basic_language: String,
        pub directory: Option<String>,
    }
}
