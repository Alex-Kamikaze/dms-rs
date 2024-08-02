pub mod errors {
    use std::io;

    use thiserror::Error;

    #[derive(Error, Debug)]
    #[doc = "Custom errors and wrapper types for external error types, that can occur in static dictionaries"]
    pub enum StaticDictionaryErrors {
        #[error("Basic dictionary not found and language is not provided")]
        /// Error, that is raised when utility cannot find basic dictionary (dictionary-*.base.json)
        BasicDictionaryNotFound,
        #[error("Cannot parse JSON into dictionary")]
        /// Wrapper around serde_json::Error type
        JSONParsingError(#[from] serde_json::Error),
        #[error("Error occured while translating word")]
        /// Wrapper around reqwest::Error type
        APIError(#[from] reqwest::Error),
        /// Wrapper around io::Error type
        #[error("Error occured during working with dictionary file")]
        IOError(#[from] io::Error),
    }
}
