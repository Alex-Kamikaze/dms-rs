#![allow(dead_code)]
#![allow(async_fn_in_trait)]

pub mod errors;

#[doc = "Типы данных, которые используются во всех частях API"]
pub mod types {
    use serde::{Deserialize, Serialize};
    use std::fmt::Display;

    use crate::errors::errors::StaticDictionaryErrors;

    #[doc = "Треит, который должны реализовывать все структуры, используемые для обращения к API переводчиков"]
    pub trait TranslatorApi {
        async fn translate_word_with_tag(
            &self,
            word: Word,
            target_language: String,
        ) -> Result<Word, StaticDictionaryErrors>;
    }

    #[derive(Serialize, Deserialize, Default, Clone)]
    #[doc = "Промежуточная модель между JSON-словарями и API"]
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
        #[inline]
        #[doc = "Сериализует модель в JSON"]
        pub fn into_json(&self) -> Result<String, serde_json::Error> {
            serde_json::to_string(self)
        }
        #[inline]
        #[doc = "Инициализирует модель из JSON"]
        pub fn from_json(json_data: String) -> Result<Word, serde_json::Error> {
            serde_json::from_str::<Word>(&json_data)
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

#[doc = "Компоненты для работы с API переводчиками"]
pub mod web_api {
    use std::collections::HashMap;

    use crate::errors::errors::StaticDictionaryErrors;
    use crate::types::TranslatorApi;
    use crate::types::Word;

    use serde::Deserialize;
    use serde::Serialize;
    use serde_json::Value;

    #[derive(Debug, Clone)]
    #[doc = "Структура для работы с API LibreTranslate"]
    pub struct LibreTranslateApi {
        pub host: String,
    }

    #[derive(Serialize, Deserialize)]
    #[doc = "Модель запроса к LibreTranslate"]
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
        ) -> Result<Word, StaticDictionaryErrors> {
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
            let translated_word: HashMap<String, Value> = serde_json::from_str(&result)?;
            Ok(Word::new(
                translated_word["translatedText"].to_string(),
                word.tag,
                target_language,
            ))
        }
    }
}

#[doc = "Парсер для JSON словарей (А также некоторые фичи для preprocess)"]
//TODO: Вынести функции, используемые только в preprocess в отдельный модуль
pub mod parser {
    use std::{
        fs,
        io::{self, Write},
        sync::Arc,
        thread,
    };

    use regex::Regex;

    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
    use serde::de::Error;
    use serde_json::json;

    use crate::{errors::errors::StaticDictionaryErrors, types::Word};

    #[doc = "Считывает JSON из словаря"]
    pub fn read_json_dictionary(file_name: &str) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&fs::read_to_string(file_name).unwrap())
    }

    #[doc = "Парсит список тегов из JSON словаря"]
    pub fn get_tags_from_dictionary(
        dictionary: serde_json::Value,
    ) -> Result<Vec<String>, StaticDictionaryErrors> {
        match dictionary.as_object() {
            Some(dict) => Ok(dict.keys().cloned().collect()),
            None => Err(StaticDictionaryErrors::JSONParsingError(
                serde_json::Error::custom("Tags not found in dictionary"),
            )),
        }
    }

    #[doc = "Возвращает путь к словарю на определенном языке"]
    pub fn get_dictionary_by_lang(
        dictionary_path: &str,
        lang: &str,
    ) -> Result<String, StaticDictionaryErrors> {
        let dictionary_list_dir = fs::read_dir(dictionary_path)?;

        for file in dictionary_list_dir {
            if let Ok(entry) = file {
                let filename = entry.file_name().into_string().unwrap();
                if filename.contains(&("dictionary-".to_owned() + lang)) {
                    return Ok(filename);
                }
            }
        }

        Err(StaticDictionaryErrors::IOError(io::Error::new(
            io::ErrorKind::NotFound,
            "Файл словаря не найден",
        )))
    }

    
    #[doc = "Возвращает путь к базовому словарю"]
    pub fn get_basic_dictionary(dictionary_dir: &str) -> Result<String, StaticDictionaryErrors> {
        let dictionary_list_dir = fs::read_dir(dictionary_dir)?;

        for file in dictionary_list_dir {
            if let Ok(entry) = file {
                let filename = entry.file_name().into_string().unwrap();
                if filename.contains(".base") {
                    return Ok(filename);
                }
            }
        }

        Err(StaticDictionaryErrors::BasicDictionaryNotFound)
    }

    #[doc = "(Только для preprocess) Создает пустые словари на основе базового словаря"]
    pub fn generate_empty_dictionaries(
        dictionary_path: String,
        languages: Vec<String>,
    ) -> Result<(), StaticDictionaryErrors> {
        let basic_dictionary_path = get_basic_dictionary(&dictionary_path)?;
        let mut handlers = vec![];
        let dictionary_path_arc = Arc::new(dictionary_path);
        for language in languages {
            let dictionary_path_clone = Arc::clone(&dictionary_path_arc);
            let path_clone = basic_dictionary_path.clone();
            let handler = thread::spawn(move || {
                let tags = get_tags_from_dictionary(read_json_dictionary(
                    &(format!("{}/", *dictionary_path_clone.to_owned()) + &path_clone),
                )?)
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

    #[doc = "Возвращает язык файла словаря"]
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

    #[doc = "Парсит JSON файл в Vec<Word>"]
    pub fn parse_json_into_words(
        dictionary_dir: &str,
        language: &str,
    ) -> Result<Vec<Word>, StaticDictionaryErrors> {
        let filename = get_dictionary_by_lang(dictionary_dir, language)?;
        let path = format!("{}/", dictionary_dir.to_owned()) + &filename;
        let json = read_json_dictionary(&path)?;
        let json_clone = json.clone();
        let keys = get_tags_from_dictionary(json)?;
        Ok(keys
            .par_iter()
            .map(|tag| {
                let tag_data = json_clone.get(tag).unwrap();
                Word::new(
                    tag_data.get("word").unwrap().to_string(),
                    tag.to_owned(),
                    language.to_owned(),
                )
            })
            .collect::<Vec<Word>>())
    }
}

#[doc = "Функционал для генерации и парсинга static-словарей"]
pub mod static_translate {
    use std::fs;
    use std::sync::{Arc, Mutex};

    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
    use serde_json::Value;

    use crate::errors::errors::StaticDictionaryErrors;
    use crate::file_system::check_dictionary_exists;
    use crate::parser::get_basic_dictionary;
    use crate::parser::get_dictionary_language;
    use crate::types::Word;

    #[doc = "Парсит список слов в Vec<Word>. Если не передан аргумент с названием языка, то будут получены слова из базового словаря"]
    //TODO: Переписать так, чтобы парсился ТОЛЬКО БАЗОВЫЙ СЛОВАРЬ
    pub fn parse_static_dictionary(
        dictionary_dir: &str,
        lang: Option<&str>,
    ) -> Result<Vec<String>, StaticDictionaryErrors> {
        if lang.is_some() {
            let file_content = fs::read_to_string(format!(
                "{}/dictionary-{}.json",
                dictionary_dir,
                lang.unwrap()
            ))?;
            let json_obj: Value = serde_json::from_str(&file_content)?;
            Ok(json_obj
                .as_array()
                .unwrap()
                .par_iter()
                .map(|v| v.as_str().unwrap().to_owned())
                .collect::<Vec<String>>())
        } else {
            let basic_dictionary = get_basic_dictionary(dictionary_dir)?;
            let file_content =
                fs::read_to_string(format!("{}/{}", dictionary_dir, basic_dictionary))?;
            let json_object: Value = serde_json::from_str(&file_content)?;
            Ok(json_object
                .as_array()
                .unwrap()
                .par_iter()
                .map(|v| v.as_str().unwrap().to_owned())
                .collect::<Vec<String>>())
        }
    }

    #[doc = "Генерирует пустые статические словари из базового статического словаря"]
    //TODO: Переписать с кастомной имплементацией параллелизма с thread scope
    pub fn generate_empty_dictionaries_from_static_basic(
        dictionary_dir: &str,
        languages: Vec<String>,
    ) -> Result<(), StaticDictionaryErrors> {
        let mut basic_dictionary = parse_static_dictionary(dictionary_dir, None)?;
        basic_dictionary.dedup();
        let words = Arc::new(
            basic_dictionary
                .par_iter()
                .map(|word| {
                    Word::new(
                        word.to_owned(),
                        word.to_owned(),
                        get_dictionary_language(&get_basic_dictionary(dictionary_dir).unwrap())
                            .unwrap(),
                    )
                    .to_owned()
                })
                .collect::<Vec<Word>>(),
        );

        languages.par_iter().for_each(|language| {
            if check_dictionary_exists(dictionary_dir, language) {
                fs::remove_file(format!("{}/dictionary-{}.json", dictionary_dir, language))
                    .unwrap();
            }
            let file =
                fs::File::create_new(format!("{}/dictionary-{}.json", dictionary_dir, language))
                    .expect(&format!(
                        "Произошла ошибка при попытке создать файл словаря dictionary-{}.json",
                        language
                    ));
            let json_object = Arc::new(Mutex::new(serde_json::json!({})));
            let words = Arc::clone(&words);
            words.par_iter().for_each(|word| {
                let mut json_object = json_object.lock().unwrap();
                json_object[word.clone().word] = "".into();
            });
            serde_json::to_writer_pretty(&file, &*json_object.lock().unwrap()).unwrap();
        });
        Ok(())
    }
}

#[doc = "Модуль с функциями для работы с репозиториями словарей"]
pub mod file_system {
    use std::{
        fs::{self, File},
        path::Path,
    };

    use crate::errors::errors::StaticDictionaryErrors;

    #[doc = "Инициализирует новый репозиторий словарей"]
    pub fn init_new_dictionary_system(
        parent: Option<String>,
        basic_language: String,
    ) -> Result<(), StaticDictionaryErrors> {
        match parent {
            Some(path) => {
                fs::create_dir_all(format!("{}/dictionaries", path))?;
                let file = File::create_new(format!(
                    "{}/dictionaries/dictionary-{}.base.json",
                    path, basic_language
                ))?;
                let json_object = serde_json::json!([]);
                serde_json::to_writer_pretty(&file, &json_object)?;
            }
            None => {
                let path = std::env::current_dir()?.to_str().unwrap().to_owned();
                fs::create_dir_all(format!("{}/dictionaries", path))?;
                let file = File::create_new(format!(
                    "{}/dictionaries/dictionary-{}.base.json",
                    &path, basic_language
                ))?;
                let json_object = serde_json::json!([]);
                serde_json::to_writer_pretty(&file, &json_object)?;
            }
        }
        Ok(())
    }

    #[doc = "Проверяет наличие словаря определенного языка в репозитории"]
    pub fn check_dictionary_exists(dictionary_path: &str, language: &str) -> bool {
        Path::new(&format!("{}/dictionary-{}.json", dictionary_path, language)).exists()
    }
}

#[cfg(test)]
mod tests {

    use super::types::*;
    use crate::file_system::check_dictionary_exists;
    use crate::parser::get_basic_dictionary;
    use crate::parser::get_dictionary_by_lang;
    use crate::parser::get_tags_from_dictionary;
    use crate::parser::read_json_dictionary;
    use crate::static_translate::generate_empty_dictionaries_from_static_basic;
    use crate::static_translate::parse_static_dictionary;
    use crate::web_api::LibreTranslateApi;

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
                let keys = get_tags_from_dictionary(json);
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
            Ok(filename) => {
                println!("{}", filename);
            }
            Err(_) => {
                panic!("Error: dictionary is not found!");
            }
        }
    }

    #[test]
    fn test_utility_finds_correct_path_to_basic_dictionary() {
        let dictionaries_dir = "C:/Users/Timur/Desktop/auto-translator/api/src/dictionaries";
        let result = get_basic_dictionary(&dictionaries_dir);
        match result {
            Ok(path) => {
                assert_eq!("dictionary-ru.base.json", path)
            }
            Err(_) => {
                println!("Basic dictionary is not found")
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
    //FIXME: Figure out why code works correct, but test fails
    fn test_generation_of_static_dictionaries() {
        let dictionary_path = "C:/Users/Timur/Desktop/auto-translator/api/src/dictionaries";
        let result = generate_empty_dictionaries_from_static_basic(
            &dictionary_path,
            vec!["en".to_owned(), "de".to_owned()],
        );
        match result {
            Ok(_) => {}
            Err(_err) => {
                panic!("Error occured while generating empty dictionaries");
            }
        }
    }

    #[test]
    fn test_check_path_works_correctly() {
        let dictionaries_path = "C:/Users/Timur/Desktop/auto-translator/dictionaries";
        assert_eq!(check_dictionary_exists(dictionaries_path, "de"), true);
        assert_eq!(check_dictionary_exists(dictionaries_path, "en"), true);
    }
}
