#![allow(dead_code)]


pub mod types {
    use std::fmt::Display;
    use serde::{Serialize, Deserialize};
    use serde_json::*;

    pub trait TranslatorApi {
        fn translate_word_with_tag(word: String, tag: String) -> Word;
    }
    
    #[derive(Serialize, Deserialize)]
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
    
        pub fn into_json(&self) -> Result<String> {
            serde_json::to_string(&self)
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
