pub mod errors {
    use std::io;

    use thiserror::Error;

    #[derive(Error, Debug)]
    #[doc = "Кастомные типы ошибки и обертки для внешних типов ошибок, которые могут возникать в функциях модулей работы со статическими словарями"]
    pub enum StaticDictionaryErrors {
        #[error("Базовый словарь не найден и не предоставлено название языка для поиска другого словаря")]
        /// Ошибка, которая вызывается, если утилита не может найти базового словаря и если при вызове функции не был передан аргумент с названием языка
        BasicDictionaryNotFound,
        #[error("Не удалось спарсить JSON файл словаря")]
        /// Обертка для типа serde_json::Error
        JSONParsingError(#[from] serde_json::Error),
        #[error("Произошла ошибка при переводе слова в API")]
        /// Обертка для типа reqwest::Error
        APIError(#[from] reqwest::Error),
        /// Обертка для типа io::Error
        #[error("Произошла ошибка при работе с файлом словаря")]
        IOError(#[from] io::Error),
        /// Обертка для ошибок при асинхронных операциях
        #[error("Произошла ошибка при выполнении асинхронной операции")]
        AsyncError(#[from] tokio::task::JoinError),
        /// Обертка для ошибок при работе с регулярными выражениями
        #[error("Ошибка при работе с регулярными выражениями")]
        RegexError(#[from] regex::Error),
    }

    #[derive(Error, Debug)]
    #[doc = "Кастомные типы ошибок и обертки для внешних типов ошибок, которые могут возникать при работе с системой сборки итоговых словарей"]
    pub enum BuildSystemErrors {
        #[error("Произошла ошибка при работе с файловой системой")]
        IOError(#[from] io::Error),
        #[error("Произошла ошибка при компиляции регулярного выражения")]
        RegexError(#[from] regex::Error),
        #[error("Произошла ошибка в системе статических словарей")]
        StaticDictionaryError(#[from] StaticDictionaryErrors),
        #[error("Произошла ошибка при работе с JSON")]
        JSONError(#[from] serde_json::Error),
    }
}
