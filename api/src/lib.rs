#![allow(dead_code)]
#![allow(async_fn_in_trait)]

pub mod errors;
use errors::*;

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
            Word {
                word,
                tag,
                language: lang,
            }
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
            Ok(Word::new(
                translated_word["translatedText"].to_string(),
                word.tag,
                target_language,
            ))
        }
    }
}

#[doc = "Module with functionality that is related to parsing JSON Dictionary files"]
pub mod parser {
    use std::{
        fs,
        io::{self, Write},
        sync::Arc,
        thread,
    };

    use regex::Regex;

    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
    use serde_json::{json, Value};

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
        let dictionary_list_dir =
            fs::read_dir(dictionary_path).expect("Error occurred during reading dictionary dir");

        for file in dictionary_list_dir {
            if let Ok(entry) = file {
                let filename = entry.file_name().into_string().unwrap();
                if filename.contains(&("dictionary-".to_owned() + lang)) {
                    return Some(filename);
                }
            }
        }

        None
    }

    #[doc = "Parses json file into Vec<Word>"]
    //TODO: Replace with correct error type
    pub fn parse_json_into_words(dictionary_dir: &str, language: &str) -> Result<Vec<Word>, ()> {
        let filename = get_dictionary_by_lang(dictionary_dir, language);
        let json = if filename.is_some() {
            let path = format!("{}/", dictionary_dir.to_owned()) + &filename.unwrap();
            read_json_dictionary(&path)
        } else {
            return Err(());
        };

        match json {
            Ok(data) => {
                let keys = get_tags_from_dictionary(&data);
                match keys {
                    Ok(tags) => {
                        return Ok(tags
                            .par_iter()
                            .map(|tag| {
                                let tag_data = data.get(tag).unwrap();
                                Word::new(
                                    tag_data.get("word").unwrap().to_string(),
                                    tag.to_owned(),
                                    language.to_owned(),
                                )
                            })
                            .collect::<Vec<Word>>());
                    }
                    Err(_) => Err(()),
                }
            }
            Err(_) => Err(()),
        }
    }

    #[doc = "Returns path to the basic dictionary"]
    pub fn get_basic_dictionary(dictionary_dir: &str) -> Option<String> {
        let dictionary_list_dir =
            fs::read_dir(dictionary_dir).expect("Error occurred during reading dictionary dir");

        for file in dictionary_list_dir {
            if let Ok(entry) = file {
                let filename = entry.file_name().into_string().unwrap();
                if filename.contains(".base") {
                    return Some(filename);
                }
            }
        }

        None
    }

    #[doc = "Generates empty dictionaries, based on basic dictionaries, for manual translation"]
    pub fn generate_empty_dictionaries(
        dictionary_path: String,
        languages: Vec<String>,
    ) -> std::io::Result<()> {
        let basic_dictionary_path = get_basic_dictionary(&dictionary_path);
        let mut handlers = vec![];
        match basic_dictionary_path {
            Some(path) => {
                let dictionary_path_arc = Arc::new(dictionary_path);
                for language in languages {
                    let dictionary_path_clone = Arc::clone(&dictionary_path_arc);
                    let path_clone = path.clone();
                    let handler = thread::spawn(move || {
                        let tags = get_tags_from_dictionary(
                            &read_json_dictionary(
                                &(format!("{}/", *dictionary_path_clone.to_owned()) + &path_clone),
                            )
                            .unwrap(),
                        )
                        .unwrap();
                        let mut json_object = serde_json::json!({});
                        for tag in tags {
                            json_object[&tag] = json!({
                                "word": ""
                            });
                        }
                        let mut new_dictionary = fs::File::create_new(format!(
                            "{}/dictionary-{}.json",
                            *dictionary_path_clone.to_owned(),
                            language
                        ))
                        .unwrap();
                        new_dictionary.write_all(
                            serde_json::to_string_pretty(&json_object)
                                .unwrap()
                                .as_bytes(),
                        )
                    });
                    handlers.push(handler);
                }
                for handle in handlers {
                    let _ = handle.join().unwrap();
                }
                Ok(())
            }
            None => Err(std::io::Error::new(
                io::ErrorKind::Other,
                "Couldn't find a basic dictionary",
            )),
        }
    }

    #[doc = "Returns a language of dictionary"]
    pub fn get_dictionary_language(dictionary_name: &str) -> Result<String, ()> {
        let pattern = Regex::new(r"^dictionary-(.+?)(?:\.base)?\.json$").unwrap();
        if let Some(captures) = pattern.captures(dictionary_name) {
            if let Some(language) = captures.get(1) {
                return Ok(language.as_str().to_owned());
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

#[doc = "Module with items related to generating and parsing static dictionaries"]
mod static_translate {
    use std::fs;
    use std::sync::{Arc, Mutex};

    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
    use serde_json::Value;

    use crate::errors::errors::StaticDictionaryErrors;
    use crate::parser::get_basic_dictionary;
    use crate::parser::get_dictionary_language;
    use crate::types::Word;

    #[doc = "Parses list of words into Vec<String>. If language is not provided, it will parse basic dictionary"]
    pub fn parse_static_dictionary(
        dictionary_dir: &str,
        lang: Option<&str>,
    ) -> Result<Vec<String>, StaticDictionaryErrors> {
        if lang.is_some() {
            let file_content = fs::read_to_string(format!(
                "{}/dictionary-{}.json",
                dictionary_dir,
                lang.unwrap()
            ))
            .unwrap();
            let json_obj: Value = serde_json::from_str(&file_content).unwrap();
            Ok(json_obj
                .as_array()
                .unwrap()
                .par_iter()
                .map(|v| v.as_str().unwrap().to_owned())
                .collect::<Vec<String>>())
        } else {
            let basic_dictionary = get_basic_dictionary(dictionary_dir);
            match basic_dictionary {
                Some(path) => {
                    let file_content =
                        fs::read_to_string(format!("{}/{}", dictionary_dir, path)).unwrap();
                    let json_object: Value = serde_json::from_str(&file_content).unwrap();
                    Ok(json_object
                        .as_array()
                        .unwrap()
                        .par_iter()
                        .map(|v| v.as_str().unwrap().to_owned())
                        .collect::<Vec<String>>())
                }
                None => {
                    return Err(
                        StaticDictionaryErrors::LanguageNotProvidedAndBasicDictionaryNotFound,
                    )
                }
            }
        }
    }

    #[doc = "Generates empty dictionaries from basic static dictionary"]
    pub fn generate_empty_dictionaries_from_static_basic(
        dictionary_dir: &str,
        languages: Vec<&str>,
    ) -> Result<(), StaticDictionaryErrors> {
        let basic_dictionary = parse_static_dictionary(dictionary_dir, None)?;
        let words = Arc::new(basic_dictionary.par_iter().map(|word| {
            Word::new(
                word.to_owned(),
                word.to_owned(),
                get_dictionary_language(&get_basic_dictionary(dictionary_dir).unwrap()).unwrap(),
            )
            .to_owned()
        })
        .collect::<Vec<Word>>());

        languages.par_iter()
            .for_each(|language| {
                let file = fs::File::create_new(format!("{}/dictionary-{}.json", dictionary_dir, language)).unwrap();
                let json_object = Arc::new(Mutex::new(serde_json::json!({})));
                let words = Arc::clone(&words);
                words.par_iter()
                    .for_each(|word| {
                        let mut json_object = json_object.lock().unwrap();
                        json_object[word.clone().word] = "".into();
                    });
                serde_json::to_writer_pretty(&file, &*json_object.lock().unwrap()).unwrap();
            });
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::types::*;
    use crate::parser::generate_empty_dictionaries;
    use crate::parser::get_basic_dictionary;
    use crate::parser::get_dictionary_by_lang;
    use crate::parser::get_tags_from_dictionary;
    use crate::parser::parse_json_into_words;
    use crate::parser::read_json_dictionary;
    use crate::static_translate::parse_static_dictionary;
    use crate::web_api::LibreTranslateApi;
    use crate::static_translate::generate_empty_dictionaries_from_static_basic;

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
            Some(filename) => {
                println!("{}", filename);
            }
            None => {
                panic!("Error: dictionary is not found!");
            }
        }
    }

    #[test]
    fn test_words_parsing_correctly() {
        let dictionaries_dir = "C:/Users/Timur/Desktop/auto-translator/api/src/dictionaries";
        let language = "ru";
        let words = parse_json_into_words(&dictionaries_dir, language);
        match words {
            Ok(words) => {
                assert_eq!(words.get(0).unwrap().language, "ru");
                assert_eq!(words.get(0).unwrap().tag, "greeting");
                assert_eq!(words.get(0).unwrap().word.replace("\"", ""), "Привет");
            }
            Err(_) => {
                panic!("Error occured while parsing words");
            }
        }
    }

    #[test]
    fn test_utility_finds_correct_path_to_basic_dictionary() {
        let dictionaries_dir = "C:/Users/Timur/Desktop/auto-translator/api/src/dictionaries";
        let result = get_basic_dictionary(&dictionaries_dir);
        match result {
            Some(path) => {
                assert_eq!("dictionary-en.base.json", path)
            }
            None => {
                println!("Basic dictionary is not found")
            }
        }
    }

    #[test]
    fn test_generate_empty_dictionaries() {
        let languages = vec!["de".to_owned(), "en".to_owned()];
        let dictionaries_dir = "C:/Users/Timur/Desktop/auto-translator/api/src/dictionaries";
        let result = generate_empty_dictionaries(dictionaries_dir.to_owned(), languages);
        match result {
            Ok(()) => {}
            Err(_) => {
                panic!("Error occured while creating dictionaries");
            }
        }
    }

    #[test]
    fn test_static_dictionary_parses_correctly() {
        let dictionary_path = "C:/Users/Timur/Desktop/auto-translator/api/src/dictionaries";
        let result = parse_static_dictionary(dictionary_path, None);
        match result {
            Ok(words) => {
                assert_eq!(
                    words.contains(&"Добро пожаловать на наш сайт".to_owned()),
                    true
                );
                assert_eq!(words.contains(&"Здесь вам не рады".to_owned()), true);
            }
            Err(_) => {
                panic!("Error occured: Coudn't find basic dictionary");
            }
        }
    }

    #[test]
    fn test_generation_of_static_dictionaries() {
        let dictionary_path = "C:/Users/Timur/Desktop/auto-translator/api/src/dictionaries";
        let result = generate_empty_dictionaries_from_static_basic(&dictionary_path, vec!["en", "de"]);
        match result {
            Ok(_) => {}
            Err(err) => {
                panic!("Error occured while generating empty dictionaries");
            }
        }
    }
}
