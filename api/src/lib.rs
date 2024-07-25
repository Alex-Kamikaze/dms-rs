#![allow(dead_code)]
#![allow(async_fn_in_trait)]

pub mod types {
    use serde::{Deserialize, Serialize};
    use std::fmt::Display;

    #[doc = "Trait, that should be implemented in all objects, that is used for translating words with API's. This enables the State pattern"]
    pub trait TranslatorApi {
        async fn translate_word_with_tag(
            &self,
            word: Word,
            target_language: String,
        ) -> Result<Word, reqwest::Error>;
    }

    #[derive(Serialize, Deserialize, Default, Clone)]
    #[doc = "Middleware representation between json and api model"]
    pub struct Word {
        pub word: String,
        pub tag: String,
        pub language: String,
    }

    impl Word {
        pub fn new(word: String, tag: String, lang: String) -> Word {
            return Word {
                word,
                tag,
                language: lang,
            };
        }

        #[doc = "Serializing structure into JSON for build step"]
        pub fn into_json(&self) -> Result<String, String> {
            if let Ok(json) = serde_json::to_string(&self) {
                Ok(json)
            } else {
                Err("Error occured while serializing data".to_owned())
            }
        }

        #[doc = "Deserializing JSON into Structure for internal functionality"]
        pub fn from_json(json_data: String) -> Result<Word, String> {
            if let Ok(word) = serde_json::from_str(&json_data) {
                Ok(word)
            } else {
                Err("Error happened while deserializing".to_owned())
            }
        }
    }

    impl Display for Word {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "Word: {}, tag: {}, lang: {}",
                self.word, self.tag, self.language
            )
        }
    }
}

pub mod web_api {
    use std::collections::HashMap;

    use crate::types::TranslatorApi;
    use crate::types::Word;

    use serde::Deserialize;
    use serde::Serialize;
    use serde_json::Value;

    #[derive(Debug, Clone)]
    #[doc = "Struct that represents the caller for LibreTranslate API"]
    pub struct LibreTranslateApi {
        pub host: String,
    }

    #[derive(Serialize, Deserialize)]
    #[doc = "Struct that represents JSON that is sent to the LibreTranslator API"]
    struct LibreTranslateJsonRequest {
        #[serde(rename = "q")]
        pub word: String,
        pub source: String,
        pub target: String,
        pub format: String,
    }

    impl LibreTranslateJsonRequest {
        pub fn new(
            word: String,
            source: String,
            target: String,
            format: String,
        ) -> LibreTranslateJsonRequest {
            LibreTranslateJsonRequest {
                word,
                source,
                target,
                format,
            }
        }
    }

    impl LibreTranslateApi {
        pub fn new(host: String) -> LibreTranslateApi {
            LibreTranslateApi { host }
        }
    }

    impl TranslatorApi for LibreTranslateApi {
        async fn translate_word_with_tag(
            &self,
            word: Word,
            target_language: String,
        ) -> Result<Word, reqwest::Error> {
            let client = reqwest::Client::new();
            let json_data = LibreTranslateJsonRequest::new(
                word.word,
                word.language,
                target_language.clone(),
                "text".to_owned(),
            );
            let result = client
                .post(format!("{}/translate", self.host))
                .json(&json_data)
                .send()
                .await?
                .text()
                .await?;
            let translated_word: HashMap<String, Value> =
                serde_json::from_str(&result).expect("Error occured while parsing");
            return Ok(Word::new(
                translated_word["translatedText"].to_string(),
                word.tag,
                target_language,
            ));
        }
    }
}

mod manual_translation {
    use std::fs::read_to_string;

    #[doc = "Reads JSON from given file"]
    fn parse_json_dictionary(file_name: &str) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&read_to_string(file_name).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::types::*;
    use crate::web_api::LibreTranslateApi;

    #[test]
    fn test_word_constructor_works() {
        let word = Word::new(
            "Плохое слово".to_owned(),
            "offensive_word".to_owned(),
            "RUS".to_owned(),
        );
        assert_eq!(word.language, "RUS");
        assert_eq!(word.tag, "offensive_word");
    }

    #[tokio::test]
    async fn test_libre_translator_on_localhost_works() {
        let api = LibreTranslateApi::new("http://127.0.0.1:5000".to_owned());
        let test_word = Word::new("Привет".to_owned(), "greeting".to_owned(), "ru".to_owned());
        let test_word_clone = test_word.clone();
        let result = api
            .translate_word_with_tag(test_word, "en".to_owned())
            .await;
        match result {
            Ok(word) => {
                assert_eq!(word.word.trim().replace("\"", ""), "Hey");
                assert_eq!(word.language, "en");
                assert_eq!(word.tag, test_word_clone.tag)
            }
            Err(err) => {
                println!("{}", err)
            }
        }
    }
}
