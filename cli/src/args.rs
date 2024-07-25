pub mod cli_args {
    use clap::{builder::Str, Args, Parser, Subcommand};
    
    #[derive(Parser, Debug)]
    #[clap(version = "1", about = "Utility for auto-generating JSON dictionaries with wide range of translating APIs", long_about = None)]
    #[doc = "Parser for CLI arguments"]
    pub struct Cli {
        #[clap(subcommand)]
        pub subcommand: CliSubcommands
    }

    #[derive(Debug, Subcommand)]
    pub enum CliSubcommands {
        #[clap(subcommand)]
        #[doc = "Translate feature with two variants: Auto or Manual"]
        /// Translate text from dictionaries
        Translate(TranslateType)
    }


    #[derive(Debug, Subcommand)]
    #[doc = "Subcommand for providing type of translation"]
    pub enum TranslateType {
        #[doc = "Manual translation variant"]
        /// Create an empty dictionaries for manual translation
        Manual(ManualTranslationArgs),

        #[doc = "Auto translation variant"]
        /// Generate empty dictionaries and fill them with translated words from basic dictionaries by translation API's
        Auto(AutoTranslationArgs)
    }

    #[derive(Debug, Args)]
    #[doc = "Arguments for manual translation feature"]
    pub struct ManualTranslationArgs {
        /// Target languages
        pub languages: Vec<String>
    }

    #[derive(Debug, Args)]
    #[doc = "Arguments for automatic translation feature"]
    pub struct AutoTranslationArgs {
        /// API that is used to translate words
        pub translator_api: String,
        /// Target languages
        pub languages: Vec<String>
    }
}