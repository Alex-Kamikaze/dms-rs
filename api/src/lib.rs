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

    #[derive(Serialize, Deserialize, Default, Clone, Debug)]
    #[doc = "Промежуточная модель между JSON-словарями и API"]
    pub struct Word {
        pub word: String,
        pub tag: String,
        pub language: String,
    }

    #[doc = "Варианты API переводчиков для передачи в функции автоматических переводчиков"]
    #[derive(Debug, Clone, Default)]
    pub enum TranslatorApis {
        #[default]
        LibreTranslate,
        DeepL,
        Yandex,
    }

    #[doc = "Аргументы для API автоперевода"]
    #[derive(Debug, Clone, PartialEq)]
    pub struct ApiArgs {
        pub api_key: Option<String>,
        pub host: String,
    }

    impl ApiArgs {
        pub fn new(api_key: Option<String>, host: String) -> ApiArgs {
            ApiArgs { api_key, host }
        }
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
        collections::HashMap,
        env, fs,
        io::{self, BufRead},
        sync::{Arc, Mutex},
    };

    use regex::Regex;

    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
    use serde::de::Error;
    use types::ConfigFileParameters;

    use crate::{
        errors::errors::StaticDictionaryErrors, file_system::{get_file_extension, parse_config},
        static_translate::update_basic_dictionary, types::Word,
    };

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

    #[doc = "Составляет регулярное выражение для получения всех фраз из файла для базовго словаря"]
    #[inline]
    pub fn generate_regex(
        regex_start: Vec<String>,
        regex_end: Vec<String>,
    ) -> Result<Regex, StaticDictionaryErrors> {
        let start_pattern = regex_start.join("|");
        let end_pattern = regex_end.join("|");
        let pattern = format!(
            r#"({})"(.*?)"({})"#,
            regex::escape(&start_pattern),
            regex::escape(&end_pattern)
        );
        Ok(Regex::new(&pattern)?)
    }

    #[doc = "Сканирует файлы на наличие строк для добавления в базовый словарь"]
    pub fn scan_files_for_phrases(
        config_path: Option<String>,
    ) -> Result<(), StaticDictionaryErrors> {
        let config = parse_config(config_path)?;
        println!("{:?}", config.exclude_files);
        let exclude_files_patterns: Vec<Regex> = config
            .exclude_files
            .par_iter()
            .map(|exclude| {
                Regex::new(*&exclude).expect(&format!("Ошибка: неправильный паттерн {}", exclude))
            })
            .collect();
        println!("{:?}", exclude_files_patterns);
        let include_files_patterns: Arc<Mutex<HashMap<String, Regex>>> =
            Arc::new(Mutex::new(HashMap::new()));
        config.languages_configurations.par_iter().for_each(|conf| {
            let local_patterns = Arc::clone(&include_files_patterns);
            for (_, configurations) in conf {
                let pattern_start = configurations.string_start.clone();
                let pattern_end = configurations.string_end.clone();
                let pattern =
                    generate_regex(pattern_start, pattern_end).expect("Не удалось создать паттерн");
                configurations
                    .file_extensions
                    .par_iter()
                    .for_each(|extension| {
                        let mut patterns = local_patterns.lock().unwrap();
                        patterns.insert(extension.to_owned(), pattern.clone());
                    })
            }
        });
        let base_directory_containments = fs::read_dir(config.base_directory.clone())?;
        for file in base_directory_containments {
            match file {
                Ok(file_entry) => {
                    let exclude_patterns = exclude_files_patterns.clone();
                    let filename = file_entry.file_name().into_string().unwrap();
                    let include_patterns = Arc::clone(&include_files_patterns);
                    if exclude_patterns.len() == 0 {
                        if !filename.starts_with(".") {
                            let file_extension = get_file_extension(&filename).expect(&format!(
                                "Произошла ошибка при прочтении файла {}",
                                filename
                            ));
                            println!("Working with {}", filename);
                            if include_patterns
                                .lock()
                                .unwrap()
                                .contains_key(&format!(".{}", file_extension))
                            {
                                let phrases = get_phrases_from_file(
                                    &format!("{}/{}", config.base_directory.clone(), filename),
                                    include_patterns
                                        .lock()
                                        .unwrap()
                                        .get(&format!(".{}", file_extension))
                                        .unwrap()
                                        .clone(),
                                )?;
                                update_basic_dictionary(&config.dictionary_repo, phrases)?;
                            }
                        }
                    } else {
                        for pattern in exclude_patterns {
                            if !pattern.is_match(&filename) && !filename.starts_with(".") {
                                let file_extension = get_file_extension(&filename).expect(
                                    &format!("Произошла ошибка при прочтении файла {}", filename),
                                );
                                println!("Working with {}", filename);
                                if include_patterns
                                    .lock()
                                    .unwrap()
                                    .contains_key(&format!(".{}", file_extension))
                                {
                                    let phrases = get_phrases_from_file(
                                        &format!("{}/{}", config.base_directory.clone(), filename),
                                        include_patterns
                                            .lock()
                                            .unwrap()
                                            .get(&format!(".{}", file_extension))
                                            .unwrap()
                                            .clone(),
                                    )?;
                                    update_basic_dictionary(&config.dictionary_repo, phrases)?;
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    println!("{}", err);
                    return Err(StaticDictionaryErrors::IOError(err));
                }
            }
        }
        Ok(())
    }

    #[doc = "Ищет в файле фразы для добавления в базовый словарь"]
    pub fn get_phrases_from_file(
        filepath: &str,
        pattern: Regex,
    ) -> Result<Vec<String>, StaticDictionaryErrors> {
        let file = fs::File::open(filepath)?;
        let reader = io::BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            let line = line?;
            for cap in pattern.captures_iter(&line) {
                if let Some(matched) = cap.get(2) {
                    results.push(matched.as_str().to_string());
                }
            }
        }
        Ok(results)
    }

    #[doc = "Типы данных в парсере"]
    pub mod types {
        use std::collections::HashMap;

        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[doc = "Конфиг для настройки параметров парсера"]
        pub struct ConfigFileParameters {
            /// Директория проекта, в котором нужно сканировать файлы
            #[serde(rename = "base")]
            pub base_directory: String,
            /// Список путей, которые нужно игнорировать
            #[serde(rename = "exclude")]
            pub exclude_files: Vec<String>,
            /// Репозиторий словарей
            #[serde(rename = "dictionary_repo")]
            pub dictionary_repo: String,
            /// Директория, куда будут собираться итоговые словари
            #[serde(rename = "output_dir")]
            pub output_dir: String,
            /// Конфигурации для языков
            #[serde(rename = "include")]
            pub languages_configurations: Vec<HashMap<String, LanguageConfiguration>>,
            /// Фразы, которые не должны переводиться автоматически, только в ручную
            #[serde(rename = "manual_translate")]
            pub manual_translate_words: Vec<String>
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[doc = "Настройки парсинга: настройки для каждого конкретного языка, файлы которого будут парсится"]
        pub struct LanguageConfiguration {
            /// Расширения файлов, которые нужно проверять для конкретного языка
            #[serde(rename = "ext")]
            pub file_extensions: Vec<String>,
            /// Начало строки
            #[serde(rename = "regexp-start")]
            pub string_start: Vec<String>,
            /// Конец строки
            #[serde(rename = "regexp-end")]
            pub string_end: Vec<String>,
        }

        impl ConfigFileParameters {

            #[doc = "Парсинг конфиг-файла в структуру"]
            pub fn from_json(
                json_content: &str,
            ) -> Result<ConfigFileParameters, serde_json::Error> {
                serde_json::from_str(json_content)
            }

            #[doc = "Превращает структуру в JSON"]
            pub fn into_json(&self) -> Result<String, serde_json::Error> {
                serde_json::to_string(&self)
            }
        }
    }
}

#[doc = "Функционал для генерации и парсинга static-словарей"]
pub mod static_translate {
    use std::collections::HashMap;
    use std::fs;
    use std::{
        fs::OpenOptions,
        sync::{Arc, Mutex},
    };

    use futures::future::join_all;
    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
    use serde_json::Value;

    use crate::errors::errors::StaticDictionaryErrors;
    use crate::file_system::check_dictionary_exists;
    use crate::parser::get_basic_dictionary;
    use crate::parser::get_dictionary_language;
    use crate::types::ApiArgs;
    use crate::types::{TranslatorApi, TranslatorApis, Word};
    use crate::web_api::LibreTranslateApi;

    #[doc = "Парсит список слов из базового словаря в Vec<Word>"]
    pub fn parse_static_basic_dictionary(
        dictionary_dir: &str,
    ) -> Result<Vec<String>, StaticDictionaryErrors> {
        let basic_dictionary = get_basic_dictionary(dictionary_dir)?;
        let file_content = fs::read_to_string(format!("{}/{}", dictionary_dir, basic_dictionary))?;
        let json_object: Value = serde_json::from_str(&file_content)?;
        Ok(json_object
            .as_array()
            .unwrap()
            .par_iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect::<Vec<String>>())
    }

    #[doc = "Парсит дочерний словарь и возвращает вектор с структурами типа Word"]
    pub fn parse_translated_dictionary(
        dictionary_dir: &str,
        language: &str,
    ) -> Result<Vec<Word>, StaticDictionaryErrors> {
        let file_content =
            fs::read_to_string(format!("{}/dictionary-{}.json", dictionary_dir, language))?;
        let json_object: Value = serde_json::from_str(&file_content)?;
        let dictionary = json_object.as_object().unwrap();
        let mut result: Vec<Word> = vec![];
        for (tag, word) in dictionary {
            result.push(Word::new(
                word.to_string(),
                tag.to_owned(),
                language.to_owned(),
            ));
        }

        Ok(result)
    }

    #[doc = "Генерирует пустые статические словари из базового статического словаря"]
    pub fn generate_empty_dictionaries_from_static_basic(
        dictionary_dir: &str,
        languages: Vec<String>,
    ) -> Result<(), StaticDictionaryErrors> {
        let mut basic_dictionary = parse_static_basic_dictionary(dictionary_dir)?;
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
                    .expect(&format!("Произошла ошибка при попытке удаления существующего словаря dictionary-{}.json", language));
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

    #[doc = "Генериует статические словари на основе базового, а потом автоматически их переводит с помощью выбранного автопереводчика"]
    // Когда я писал это, только двое знали что тут вообще творится - это я и Бог. Сейчас только Бог знает, что здесь происходит....
    // А не, кажись я допер че я тут понаписал
    pub async fn autotranslate_from_basic_dictionary(
        dictionary_dir: &str,
        target_languages: Vec<String>,
        translator_api: TranslatorApis,
        api_args: ApiArgs,
    ) -> Result<(), StaticDictionaryErrors> {
        let mut basic_dictionary = parse_static_basic_dictionary(dictionary_dir)?;
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

        let translator = Arc::new(match translator_api {
            TranslatorApis::LibreTranslate => LibreTranslateApi::new(api_args.host),
            TranslatorApis::DeepL => todo!(),
            TranslatorApis::Yandex => todo!(),
        });

        let mut tasks = vec![];

        for target_language in target_languages.clone() {
            let words = Arc::clone(&words);
            let translator = Arc::clone(&translator);

            for word in &*words.clone() {
                let word = word.clone();
                let translator = Arc::clone(&translator);
                let target_language = target_language.to_string();

                let task = tokio::spawn(async move {
                    translator
                        .translate_word_with_tag(word, target_language)
                        .await
                });
                tasks.push(task);
            }
        }

        let results = join_all(tasks).await;
        let mut words_with_languages_hashmap: HashMap<String, Vec<Word>> = HashMap::new();
        target_languages.clone().iter().for_each(|language| {
            words_with_languages_hashmap.insert(language.to_owned(), vec![]);
        });
        for join_result in results {
            match join_result {
                Ok(request_result) => {
                    let word = request_result?;
                    words_with_languages_hashmap
                        .get_mut(&word.language)
                        .expect(&format!("Не найден ключ {}", word.tag))
                        .push(word.clone());
                }
                Err(err) => return Err(StaticDictionaryErrors::AsyncError(err)),
            }
        }

        for (language, words) in &words_with_languages_hashmap {
            if check_dictionary_exists(dictionary_dir, language) {
                fs::remove_file(format!("{}/dictionary-{}.json", dictionary_dir, language))?;
            }
            let file =
                fs::File::create_new(format!("{}/dictionary-{}.json", dictionary_dir, language))
                    .expect(&format!(
                        "Произошла ошибка при попытке создать файл словаря dictionary-{}.json",
                        language
                    ));
            let json_object = Arc::new(Mutex::new(serde_json::json!({})));
            let words = Arc::new(words);
            words.par_iter().for_each(|word| {
                let mut json_object = json_object.lock().unwrap();
                json_object[word.clone().tag] = word.word.replace("\"", "").clone().into();
            });
            serde_json::to_writer_pretty(&file, &*json_object.lock().unwrap())?;
        }

        Ok(())
    }

    #[doc = "Добавляет новые фразы в базовый словарь"]
    pub fn update_basic_dictionary(
        dictionary_dir: &str,
        words: Vec<String>,
    ) -> Result<(), StaticDictionaryErrors> {
        let basic_dictionary = get_basic_dictionary(dictionary_dir)?;
        let mut basic_dictionary_content = parse_static_basic_dictionary(dictionary_dir)?;

        for word in words {
            if !basic_dictionary_content.contains(&word) {
                basic_dictionary_content.push(word);
            }
        }
        let json_object: Value = serde_json::json!(basic_dictionary_content);
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(format!("{}/{}", dictionary_dir, basic_dictionary))?;
        serde_json::to_writer_pretty(&file, &json_object)?;
        Ok(())
    }

    #[doc = "Управляет синхронизацией фраз из конфига в базовый словарь"]
    pub fn sync_manual_phrases(manual_phrases: Vec<String>, dictionary_dir: &str) -> Result<(), StaticDictionaryErrors> {
        let basic_dictionary_content: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(parse_static_basic_dictionary(dictionary_dir)?));
        manual_phrases
            .par_iter()
            .for_each(|phrase| {
                let dictionary = Arc::clone(&basic_dictionary_content);
                let mut mut_dictionary = dictionary.lock().expect("Произошла ошибка при синхронизации словарей");
                if !mut_dictionary.contains(phrase) {
                    mut_dictionary.push(phrase.to_owned());
                }
            });
        Ok(())
    }
}

#[doc = "Модуль с функциями для работы с репозиториями словарей"]
pub mod file_system {
    use std::{
        ffi::OsStr,
        fs::{self, File},
        path::Path,
        env
    };

    use regex;

    use crate::{
        errors::errors::{BuildSystemErrors, StaticDictionaryErrors},
        parser::types::ConfigFileParameters,
    };

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

    #[doc = "Возвращает список всех словарей в репозитории"]
    // TODO: Заменить на другой тип ошибки
    pub fn find_all_dictionaries_in_repository(
        dictionary_path: &str,
    ) -> Result<Vec<String>, BuildSystemErrors> {
        let paths = fs::read_dir(dictionary_path)?;
        let pattern = regex::Regex::new(r"^dictionary-(.+?)(?:\.base)?\.json$")?;
        let mut result: Vec<String> = vec![];
        for file in paths {
            match file {
                Ok(path) => {
                    let filename = path.file_name().into_string().unwrap();
                    if pattern.is_match(&filename) {
                        result.push(filename);
                    }
                    return Ok(result);
                }
                Err(error) => return Err(BuildSystemErrors::IOError(error)),
            }
        }
        Ok(result)
    }

    #[doc = "Находит все переведнные словари в репозитории, игнорируя базовый словарь"]
    pub fn find_all_translated_dictionaries(
        dictionary_path: &str,
    ) -> Result<Vec<String>, StaticDictionaryErrors> {
        let paths = fs::read_dir(dictionary_path)?;
        let pattern = regex::Regex::new(r"^dictionary-[a-z]{2}\.json$")?;
        let mut result = vec![];
        for file in paths {
            match file {
                Ok(path) => {
                    let filename = path.file_name().into_string().unwrap();
                    if pattern.is_match(&filename) {
                        result.push(filename);
                    }
                }
                Err(error) => return Err(StaticDictionaryErrors::IOError(error)),
            }
        }
        return Ok(result);
    }

    #[doc = "Считывает и парсит конфиг. Если путь до конфига не передан - пытается найти его в cwd"]
    #[inline]
    pub fn parse_config_file(
        config_path: &str,
    ) -> Result<ConfigFileParameters, StaticDictionaryErrors> {
        let file_content = fs::read_to_string(config_path)?;
        let config_parsed = ConfigFileParameters::from_json(&file_content);
        match config_parsed {
            Ok(conf) => return Ok(conf),
            Err(err) => {
                println!("{:?}", err);
                return Err(StaticDictionaryErrors::JSONParsingError(err));
            }
        }
    }

    #[doc = "Идиоматически верно возвращает расширение файла"]
    #[inline]
    pub fn get_file_extension(filename: &str) -> Option<&str> {
        Path::new(filename).extension().and_then(OsStr::to_str)
    }

    #[doc = "Парсинг конфига"]
    pub fn parse_config(config_path: Option<String>) -> Result<ConfigFileParameters, StaticDictionaryErrors> {
        let config_dir = match config_path {
            Some(path) => path,
            None => format!(
                "{}/config.dms.json",
                env::current_dir()?.to_str().unwrap().to_owned()
            ),
        };
        let config_data = fs::read_to_string(config_dir)?;
        let config = ConfigFileParameters::from_json(&config_data)?;
        Ok(config)
    }
}

#[doc = "Модули и утилиты для сборки итоговых словарей"]
pub mod build_system {

    #[doc = "Интеграция с фреймворком i18next"]
    pub mod i18next_integration {
        use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

        use crate::errors::errors::BuildSystemErrors;
        use crate::file_system::find_all_translated_dictionaries;
        use crate::parser::get_dictionary_language;
        use crate::static_translate::parse_translated_dictionary;
        use std::fs;
        use std::sync::{Arc, Mutex};

        #[doc = "Функция для сборки словарей из репозитория в итоговые словари для i18next"]
        pub fn build_for_i18next(
            dictionary_dir: &str,
            output_directory: &str,
            languages: Option<Vec<String>>,
        ) -> Result<(), BuildSystemErrors> {
            let languages = match languages {
                Some(langs) => langs,
                None => {
                    let dictionaries = find_all_translated_dictionaries(dictionary_dir)?;
                    dictionaries
                        .par_iter()
                        .map(|dictionary| get_dictionary_language(&dictionary).unwrap())
                        .collect()
                }
            };
            languages
                .par_iter()
                .try_for_each(|language| -> Result<(), BuildSystemErrors> {
                    let dictionary_content = parse_translated_dictionary(dictionary_dir, language)?;
                    fs::create_dir_all(format!("{}/{}", output_directory, language))?;
                    let build_dictionary = fs::File::create_new(format!(
                        "{}/{}/translation.json",
                        output_directory, language
                    ))?;
                    let json_content = Arc::new(Mutex::new(serde_json::json!({})));

                    dictionary_content.par_iter().try_for_each(
                        |word| -> Result<(), BuildSystemErrors> {
                            let mut json_object = json_content.lock().unwrap();
                            json_object[&word.tag] = word.word.replace("\"", "").clone().into();
                            Ok(())
                        },
                    )?;

                    serde_json::to_writer_pretty(
                        &build_dictionary,
                        &*json_content.lock().unwrap(),
                    )?;
                    Ok(())
                })?;
            Ok(())
        }
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
    use crate::static_translate::parse_static_basic_dictionary;
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
        let result = parse_static_basic_dictionary(dictionary_path);
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
    fn test_check_path_works_correctly() {
        let dictionaries_path = "C:/Users/Timur/Desktop/auto-translator/dictionaries";
        assert_eq!(check_dictionary_exists(dictionaries_path, "de"), true);
        assert_eq!(check_dictionary_exists(dictionaries_path, "en"), true);
    }
}
