#![allow(unused_imports)]

use api::file_system::init_new_dictionary_system;
use api::parser::*;
use api::static_translate::generate_empty_dictionaries_from_static_basic;
use api::static_translate::autotranslate_from_basic_dictionary;
use api::types::TranslatorApis;
use api::types::Word;
use api::types::ApiArgs;
use clap::Parser;
use tokio::*;

mod args;
use crate::CliSubcommands::*;
use args::cli_args::*;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args = TranslatorCli::parse();
    match args.subcommand {
        Translate(translate_type) => {
            match translate_type {
                TranslateType::Manual(arguments) => {
                    println!(
                        "Генерируются пустые словари для языков {:?}",
                        &arguments.languages
                    );
                    let generate_result = generate_empty_dictionaries_from_static_basic(
                        &arguments.dictionary_path,
                        arguments.languages,
                    );
                    match generate_result {
                    Ok(()) => {
                        println!("Пустые словари успешно сгенерированы!");
                    }
                    Err(err) => {
                        match err {api::errors::errors::StaticDictionaryErrors::BasicDictionaryNotFound=>{println!("Ошибка: Не удалось найти базовый словарь!")}
                        api::errors::errors::StaticDictionaryErrors::JSONParsingError(_)=>{println!("Ошибка: Не удалось спарсить JSON файл словаря!")},
                        api::errors::errors::StaticDictionaryErrors::APIError(_)=>{println!("Ошибка: Ошибка при обращении к API!")},
                        api::errors::errors::StaticDictionaryErrors::IOError(_)=>{println!("Ошибка: Не удалось создать файлы!")},
                        api::errors::errors::StaticDictionaryErrors::AsyncError(_) => todo!(), }
                    }
                }
                }
                //TODO: Implement auto translation for static dictionaries with LibreTranslate
                TranslateType::Auto(api) => {
                    match api {
                        ApiVariants::Libretranslate(args) => {
                            let args_clone = args.clone();
                            let result = autotranslate_from_basic_dictionary(&args.dictionaries_path, args.languages, TranslatorApis::LibreTranslate, args_clone.into_api_args()).await;
                            match result {
                                Ok(_) => println!("Словари переведены успешно"),
                                Err(err) => println!("{}", err)
                            }
                        }
                        _ => println!("Пока не реализовано!")
                    }
                }
            }
        }

        Init(args) => match init_new_dictionary_system(args.directory, args.basic_language) {
            Ok(_) => {
                println!("Новый репозиторий словарей создан успешно");
            }
            Err(error) => match error {
                api::errors::errors::StaticDictionaryErrors::BasicDictionaryNotFound => {}
                api::errors::errors::StaticDictionaryErrors::JSONParsingError(_) => {}
                api::errors::errors::StaticDictionaryErrors::APIError(_) => {}
                api::errors::errors::StaticDictionaryErrors::IOError(_) => {
                    println!("Произошла ошибка при инициализации нового репозитория словарей. Возможно, у вас уже создан репозиторий в директории, где вы пытаетесь его создать")
                },
                api::errors::errors::StaticDictionaryErrors::AsyncError(_) => todo!()
            },
        },
    }
    Ok(())
}
