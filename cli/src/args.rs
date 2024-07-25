pub mod cli_args {
    use clap::{Parser, Subcommand, Args};
    
    #[derive(Parser, Debug)]
    #[clap(version = "1", about = "Utility for auto-generating JSON dictionaries with wide range of translating APIs", long_about = None)]
    pub struct Cli {
        #[clap(subcommand)]
        pub translate: TranslateType
    }


    #[derive(Debug, Subcommand)]
    pub enum TranslateType {
        /// Create an empty dictionaries for manual translation
        Manual(ManualTranslationArgs),

        /// Generate empty dictionaries and fill them with translated words from basic dictionaries
        Auto(AutoTranslationArgs)
    }

    #[derive(Debug, Args)]
    pub struct ManualTranslationArgs {}

    #[derive(Debug, Args)]
    pub struct AutoTranslationArgs {}
}