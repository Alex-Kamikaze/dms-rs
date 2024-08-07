pub mod cli_args {
    use api::types::ApiArgs;
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
        #[clap(subcommand)]
        Auto(ApiVariants),
    }

    #[derive(Debug, Args)]
    #[doc = "Аргументы для команды translate manual"]
    pub struct ManualTranslationArgs {
        /// Репозиторий со словарями
        pub dictionary_path: String,
        /// Языки для перевода
        pub languages: Vec<String>,
    }

    #[derive(Subcommand, Debug)]
    #[doc = "Варианты API для автоперевода"]
    pub enum ApiVariants {
        /// Перевод с использованием LibreTranslate API
        Libretranslate(AutoTranslationArgs),

        /// Перевод с использованиеим DeepL API
        Deepl(AutoTranslationArgs),
    }

    #[derive(Debug, Args, Clone)]
    #[doc = "Аргументы для команды translate auto"]
    // TODO: Заменить на разные аргументы для разных реализаций API
    pub struct AutoTranslationArgs {
        /// Директория репозитория словарей
        pub dictionaries_path: String,
        /// Языки для перевода
        pub languages: Vec<String>,
        /// API ключ для тех, кому он требуется
        #[arg(last(true))]
        pub api_key: Option<String>,
    }

    #[derive(Debug, Args)]
    #[doc = "Аргументы для команды init"]
    pub struct InitializeArguments {
        /// Язык, который будет использоваться в базовом словаре
        pub basic_language: String,
        /// Директория, где будет инициализирован репозиторий
        pub directory: Option<String>,
    }

    impl AutoTranslationArgs {
        pub fn into_api_args(self) -> api::types::ApiArgs {
            ApiArgs::new(self.api_key, "http://127.0.0.1:5000".to_owned())
        }
    }
}
