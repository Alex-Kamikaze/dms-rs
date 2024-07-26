#![allow(dead_code)]
#![allow(async_fn_in_trait)]

#[doc = "Types that is used across whole API"]
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

#[doc = "Functionality, that is related to Translator API's in web"]
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

#[doc = "Module with functionality that is related to parsing JSON Dictionary files"]
pub mod parser {
    use std::fs;

    use serde_json::Value;

    use crate::types::Word;

    #[doc = "Reads JSON from given file"]
    pub fn read_json_dictionary(file_name: &str) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&fs::read_to_string(file_name).unwrap())
    }

    #[doc = "Returns list of tags, that is used in dictionary"]
    //TODO: Replace with correct error type
    pub fn get_tags_from_dictionary(dictionary: &Value) -> Result<Vec<String>, ()> {
        if let Value::Object(dict) = dictionary {
            Ok(dict.keys().cloned().collect())
        } else {
            Err(())
        }
    }

    #[doc = "Returns filepath of dictionary based on the language input"]
    pub fn get_dictionary_by_lang(dictionary_path: &str, lang: &str) -> Option<String> {
        let dictionary_list_dir = fs::read_dir(dictionary_path).expect("Error occurred during reading dictionary dir");
    
        for file in dictionary_list_dir {
            if let Ok(entry) = file {
                let filename = entry.file_name().into_string().unwrap();
                if filename.contains(lang) {
                    return Some(filename);
                }
            }
        }

        None
    }

    #[doc = "Parses json file into Vec<Word>"]
    //TODO: Replace with correct error type
    pub fn parse_json_into_words(file_name: &str) -> Result<Vec<Word>, ()> {
        let json = read_json_dictionary(file_name);
        let language = "ru".to_owned(); //TODO: Replace with getting language from name of dictionary (dictionary-ru.json -> ru)
        match json {
            Ok(data) => {
                let keys = get_tags_from_dictionary(&data);
                match keys {
                    Ok(tags) => {
                        return Ok(tags
                            .into_iter()
                            .map(|tag| {
                                Word::new(
                                    data.get("word").unwrap().to_string(),
                                    tag,
                                    language.clone(),
                                )
                            })
                            .collect::<Vec<Word>>());
                    }
                    Err(_) => return Err(()),
                }
            }
            Err(_) => return Err(()),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::types::*;
    use crate::parser::get_tags_from_dictionary;
    use crate::parser::read_json_dictionary;
    use crate::web_api::LibreTranslateApi;
    use crate::parser::get_dictionary_by_lang;

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

    #[test]
    fn test_dictionary_file_reading() {
        let file_path = "C:/Users/Timur/Desktop/auto-translator/cli/src/test.json";
        let read_result = read_json_dictionary(&file_path);
        match read_result {
            Ok(json_object) => {
                assert_eq!(json_object.get("greeting").is_some(), true);
                assert_eq!(json_object.get("farewell").is_some(), true);
                assert_eq!(json_object["greeting"]["ru"], "Привет");
                assert_eq!(json_object["greeting"]["en"], "Hello");
                assert_eq!(json_object["greeting"]["de"], "Hallo");
            }
            Err(_) => panic!("Error occured while reading the file"),
        }
    }

    #[test]
    fn test_tags_parsed_correctly() {
        let file_path = "C:/Users/Timur/Desktop/auto-translator/cli/src/test.json";
        let read_result = read_json_dictionary(&file_path);
        match read_result {
            Ok(json) => {
                let keys = get_tags_from_dictionary(&json);
                match keys {
                    Ok(tags) => {
                        assert_eq!(tags.contains(&"farewell".to_owned()), true);
                        assert_eq!(tags.contains(&"greeting".to_owned()), true);
                    }
                    Err(_) => panic!("Tag parser function returned an Err type"),
                }
            }
            Err(_) => panic!("File-reader returned an Err type"),
        }
    }

    #[test]
    fn test_utility_finds_correct_path_to_dictionary() {
        let dictionaries_dir = "C:/Users/Timur/Desktop/auto-translator/api/src/dictionaries";
        let language = "ru";
        let result = get_dictionary_by_lang(&dictionaries_dir, &language);
        match result {
            Some(filename) => { println!("{}", filename); }
            None => { panic!("Error: dictionary is not found!"); }
        }
    }
}
