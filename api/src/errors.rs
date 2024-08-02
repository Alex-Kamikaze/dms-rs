pub mod errors {
    use std::io;

    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum StaticDictionaryErrors {
        #[error("Basic dictionary not found and language is not provided")]
        BasicDictionaryNotFound,
        #[error("Cannot parse JSON into dictionary")]
        JSONParsingError(#[from] serde_json::Error),
        #[error("Error occured while translating word")]
        APIError(#[from] reqwest::Error),
        #[error("Error occured during working with dictionary file")]
        IOError(#[from] io::Error),
    }
}
