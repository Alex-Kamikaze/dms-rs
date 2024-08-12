pub mod cli_args {
    use api::types::ApiArgs;
    use clap::{builder::Str, Args, Parser, Subcommand};

    #[derive(Parser, Debug)]
    #[clap(version = "0.2 Experimental", about = "Утилита для управления репозиторием JSON-словарей и переводом в ручном или автоматическом режиме", long_about = None)]
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
        #[clap(subcommand)]
        /// Собрать из репозитория словарей файлы в другой формат для проекта
        Build(FrameworkType),
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

    #[derive(Debug, Subcommand)]
    #[doc = "Варианты фреймворков для сборки словарей"]
    pub enum FrameworkType {
        /// Сборка в словари, совместимые с фреймворком i18next
        I18next(BuildArgs),
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
        Libretranslate(LibreTranslateArgs),

        /// Перевод с использованиеим DeepL API
        Deepl(AutoTranslationArgs),
    }

    #[derive(Debug, Args, Clone)]
    #[doc = "Аргументы для команды translate auto"]
    #[deprecated(
        since = "0.2.0",
        note = "Используйте разные реализации структур аргументов (LibreTranslateArgs)"
    )]
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

    #[derive(Debug, Args, Clone)]
    #[doc = "Аргументы, передаваемые в LibreTranslate API"]
    pub struct LibreTranslateArgs {
        /// Директория с репозиторием словарей
        pub dictionaries_path: String,
        /// Хостинг LibreTranslate
        pub host: String,
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

    impl Into<ApiArgs> for LibreTranslateArgs {
        fn into(self) -> ApiArgs {
            ApiArgs::new(None, self.host)
        }
    }

    #[derive(Debug, Clone, Args)]
    #[doc = "Аргументы, которые передаются в функции сборки итоговых словарей для конкретных фреймворков"]
    pub struct BuildArgs {
        /// Директория с репозиторием словарей
        pub dictionary_path: String,
        /// Директория с итоговыми словарями
        pub output_directory: String,
        /// По умолчанию, утилита будет собирать все словари, если нужно обновить какой-то конкретный, то можно указать их список при сборке
        pub languages: Option<Vec<String>>,
    }
}
