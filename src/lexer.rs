use crate::error::{ForthError, SourceLocation};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Number(i64),
    Float(f64),
    Word(String),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub location: SourceLocation,
}

impl Token {
    pub fn word(&self) -> Option<&str> {
        match &self.kind {
            TokenKind::Word(word) => Some(word),
            _ => None,
        }
    }

    pub fn number(&self) -> Option<i64> {
        match self.kind {
            TokenKind::Number(number) => Some(number),
            _ => None,
        }
    }

    pub fn string(&self) -> Option<&str> {
        match &self.kind {
            TokenKind::String(value) => Some(value),
            _ => None,
        }
    }
}

pub fn tokenize(source: &str, file: &str) -> Result<Vec<Token>, ForthError> {
    let mut lexer = Lexer {
        chars: source.chars().collect(),
        index: 0,
        line: 1,
        column: 1,
        file,
    };

    let mut tokens = Vec::new();

    while let Some(current) = lexer.peek() {
        if current.is_whitespace() {
            lexer.bump();
            continue;
        }

        if current == '\\' {
            lexer.skip_line_comment();
            continue;
        }

        if current == '(' {
            let start = lexer.location();
            lexer.bump();
            lexer.skip_block_comment(start)?;
            continue;
        }

        if current == '"' {
            let start = lexer.location();
            lexer.bump();
            let value = lexer.read_string_body(start.clone())?;
            tokens.push(Token {
                kind: TokenKind::String(value),
                location: start,
            });
            continue;
        }

        // Accept both `"text"` and the traditional `s" text"` spelling.
        if current == 's' && lexer.peek_next() == Some('"') {
            let start = lexer.location();
            lexer.bump(); // s
            lexer.bump(); // "
            if lexer.peek() == Some(' ') {
                lexer.bump();
            }
            let value = lexer.read_string_body(start.clone())?;
            tokens.push(Token {
                kind: TokenKind::String(value),
                location: start,
            });
            continue;
        }

        let start = lexer.location();
        let mut text = String::new();

        while let Some(ch) = lexer.peek() {
            if ch.is_whitespace() {
                break;
            }
            text.push(ch);
            lexer.bump();
        }

        let kind = match parse_number(&text) {
            Some(number) => TokenKind::Number(number),
            None => match parse_float(&text) {
                Some(float) => TokenKind::Float(float),
                None => TokenKind::Word(text),
            },
        };

        tokens.push(Token {
            kind,
            location: start,
        });
    }

    Ok(tokens)
}

fn parse_number(text: &str) -> Option<i64> {
    let (negative, body) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text),
    };

    let magnitude = if let Some(hex) = body.strip_prefix("0x").or_else(|| body.strip_prefix('$')) {
        i128::from_str_radix(hex, 16).ok()?
    } else if let Some(binary) = body.strip_prefix("0b").or_else(|| body.strip_prefix('%')) {
        i128::from_str_radix(binary, 2).ok()?
    } else {
        body.parse::<i128>().ok()?
    };

    let value = if negative { -magnitude } else { magnitude };
    i64::try_from(value).ok()
}

/// A float literal is a token containing a decimal point that parses as
/// an `f64`: `3.14`, `-2.5`, `2.`, `.5` (after the int parser has had its
/// chance). The decimal point is required so plain words like `1e3` or
/// dotted file names stay words, not numbers.
fn parse_float(text: &str) -> Option<f64> {
    if !text.contains('.') {
        return None;
    }

    if text.chars().any(|character| character == '$') {
        return None;
    }

    text.parse::<f64>().ok()
}

struct Lexer<'a> {
    chars: Vec<char>,
    index: usize,
    line: usize,
    column: usize,
    file: &'a str,
}

impl Lexer<'_> {
    fn location(&self) -> SourceLocation {
        SourceLocation::new(self.file, self.line, self.column)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.index + 1).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = *self.chars.get(self.index)?;
        self.index += 1;

        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        Some(ch)
    }

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.bump();
        }
    }

    fn skip_block_comment(&mut self, start: SourceLocation) -> Result<(), ForthError> {
        while let Some(ch) = self.bump() {
            if ch == ')' {
                return Ok(());
            }
        }

        Err(ForthError::at("unterminated comment", start))
    }

    fn read_string_body(&mut self, start: SourceLocation) -> Result<String, ForthError> {
        let mut value = String::new();

        loop {
            match self.bump() {
                None => return Err(ForthError::at("unterminated string", start)),
                Some('"') => return Ok(value),
                Some('\\') => match self.bump() {
                    None => return Err(ForthError::at("unterminated string escape", start)),
                    Some('n') => value.push('\n'),
                    Some('t') => value.push('\t'),
                    Some('r') => value.push('\r'),
                    Some('\\') => value.push('\\'),
                    Some('"') => value.push('"'),
                    Some('0') => value.push('\0'),
                    Some(other) => {
                        value.push('\\');
                        value.push(other);
                    }
                },
                Some(ch) => value.push(ch),
            }
        }
    }
}
