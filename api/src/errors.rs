pub mod errors {
    use std::io;

    use thiserror::Error;

    #[derive(Error, Debug)]
    #[doc = "Кастомные типы ошибки и обертки для внешних типов ошибок, которые могут возникать в функциях модулей работы со статическими словарями"]
    pub enum StaticDictionaryErrors {
        #[error("Базовый словарь не найден и не предоставлено название языка для поиска другого словаря")]
        /// Ошибка, которая вызывается, если утилита не может найти базового словаря и если при вызове функции не был передан аргумент с названием языка
        BasicDictionaryNotFound,
        #[error("Cannot parse JSON into dictionary")]
        /// Обертка для типа serde_json::Error
        JSONParsingError(#[from] serde_json::Error),
        #[error("Error occured while translating word")]
        /// Обертка для типа reqwest::Error
        APIError(#[from] reqwest::Error),
        /// Обертка для типа io::Error
        #[error("Error occured during working with dictionary file")]
        IOError(#[from] io::Error),
    }
}
