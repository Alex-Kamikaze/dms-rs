#![allow(unused_imports)]

use api::build_system::i18next_integration::build_for_i18next;
use api::file_system::init_new_dictionary_system;
use api::parser::*;
use api::static_translate::autotranslate_from_basic_dictionary;
use api::static_translate::generate_empty_dictionaries_from_static_basic;
use api::types::ApiArgs;
use api::types::TranslatorApis;
use api::types::Word;
use clap::Parser;
use tokio::*;

mod args;
use crate::CliSubcommands::*;
use args::cli_args::FrameworkType;
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
                        api::errors::errors::StaticDictionaryErrors::AsyncError(_) => todo!(),
                        api::errors::errors::StaticDictionaryErrors::RegexError(_) => todo!() 
                    }
                    }
                }
                }

                TranslateType::Auto(api) => {
                    match api {
                        ApiVariants::Libretranslate(args) => {
                            let args_clone = args.clone();
                            println!("{}", args.host);
                            println!("{}", args.dictionaries_path);
                            let result = autotranslate_from_basic_dictionary(
                                &args.dictionaries_path,
                                args.languages,
                                TranslatorApis::LibreTranslate,
                                args_clone.into(),
                            )
                            .await;
                            match result {
                                Ok(_) => println!("Словари переведены успешно"),
                                // TODO: Заменить на корректную обработку ошибки
                                Err(err) => {
                                    println!("{:?}", err)
                                }
                            }
                        }
                        _ => println!("Пока не реализовано!"),
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
                }
                api::errors::errors::StaticDictionaryErrors::AsyncError(_) => todo!(),
                api::errors::errors::StaticDictionaryErrors::RegexError(_) => todo!()
            },
        },

        Build(framework) => match framework {
            FrameworkType::I18next(args) => {
                let result = build_for_i18next(
                    &args.dictionary_path,
                    &args.output_directory,
                    args.languages,
                );
                match result {
                    Ok(()) => {
                        println!("Сборка завершена успешно!")
                    }
                    Err(error) => {
                        println!("{:?}", error)
                    }
                }
            }
        },
    }
    Ok(())
}
