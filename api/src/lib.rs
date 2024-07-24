#![allow(dead_code)]


pub mod types {
    use std::fmt::Display;
    use serde::{Serialize, Deserialize};
    use serde_json::*;

    #[doc = "Trait, that should be implemented in all objects, that is used for translating words with API's. This enables the State pattern"]
    pub trait TranslatorApi {
        fn translate_word_with_tag(word: Word) -> Word;
    }
    
    #[derive(Serialize, Deserialize, Default)]
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
        pub fn into_json(&self) -> Result<String> {
            serde_json::to_string(&self)
        }

        #[doc = "Deserializing JSON into Structure for internal functionality"]
        pub fn from_json(json_data: String) -> Result<Word> {
            return serde_json::from_str(&json_data)
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



#[cfg(test)]
mod tests {
    use super::types::*;

    #[test]
    fn word_constructor_works() {
        let word = Word::new(
            "Плохое слово".to_owned(),
            "offensive_word".to_owned(),
            "RUS".to_owned(),
        );
        assert_eq!(word.language, "RUS");
        assert_eq!(word.tag, "offensive_word");
    }
}
