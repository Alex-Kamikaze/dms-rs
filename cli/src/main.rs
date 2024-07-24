use api::Word;
fn main() {
    let word = Word::new(
        "Плохое слово".to_owned(),
        "offensive_word".to_owned(),
        "RUS".to_owned(),
    );
    println!("{word}");
}
