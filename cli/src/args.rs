pub mod cli_args {
    use clap::{builder::Str, Args, Parser, Subcommand};

    #[derive(Parser, Debug)]
    #[clap(version = "0.1 Experimental", about = "Утилита для управления репозиторием JSON-словарей и переводом в ручном или автоматическом режиме", long_about = None)]
    #[doc = "Парсер CLI-аргументов"]
    pub struct TranslatorCli {
        #[clap(subcommand)]
        pub subcommand: CliSubcommands,
    }

    #[derive(Debug, Subcommand)]
    pub enum CliSubcommands {
        #[clap(subcommand)]
        /// Перевести текст в статических словарях
        Translate(TranslateType),
        /// Инициализировать новый репозиторий словарей
        Init(InitializeArguments),
    }

    #[derive(Debug, Subcommand)]
    #[doc = "Варианты режима перевода"]
    pub enum TranslateType {
        /// Создать пустые словари на основе базового для ручного перевода
        Manual(ManualTranslationArgs),

        /// Сгенерировать пустые словари на основе базового, а потом перевести их с помощью одного из API
        Auto(AutoTranslationArgs),
    }

    #[derive(Debug, Args)]
    #[doc = "Аргументы для команды translate manual"]
    pub struct ManualTranslationArgs {
        /// Репозиторий со словарями
        pub dictionary_path: String,
        /// Языки для перевода
        pub languages: Vec<String>,
    }

    #[derive(Debug, Args)]
    #[doc = "Аргументы для команды translate auto"]
    pub struct AutoTranslationArgs {
        /// API, которое будет использоваться для автоперевода
        // TODO: Replace with seperate implementations of API's
        pub translator_api: String,
        /// Языки для перевода
        pub languages: Vec<String>,
    }

    #[derive(Debug, Args)]
    #[doc = "Аргументы для команды init"]
    pub struct InitializeArguments {
        /// Язык, который будет использоваться в базовом словаре
        pub basic_language: String,
        /// Директория, где будет инициализирован репозиторий
        pub directory: Option<String>,
    }
}
