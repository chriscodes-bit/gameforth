pub enum Token {
    Number(i64),
    Word(String),
}

pub fn tokenize(source: &str) -> Vec<Token> {
    source
        .split_whitespace()
        .map(|word| match word.parse::<i64>() {
            Ok(number) => Token::Number(number),
            Err(_) => Token::Word(word.to_string()),
        })
        .collect()
}
