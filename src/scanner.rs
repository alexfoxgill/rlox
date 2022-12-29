use std::rc::Rc;

use crate::rc_slice::RcSlice;

fn is_alpha(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

pub struct Scanner {
    pub source: Rc<str>,
    pub start: usize,
    pub current: usize,
    pub line: usize,
}

impl Scanner {
    pub fn init(source: Rc<str>) -> Scanner {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::EOF);
        }

        match self.advance() {
            c if c.is_ascii_digit() => self.number(),
            c if is_alpha(c) => self.identifier(),
            '(' => self.make_token(TokenType::LeftParen),
            ')' => self.make_token(TokenType::RightParen),
            '{' => self.make_token(TokenType::LeftBrace),
            '}' => self.make_token(TokenType::RightBrace),
            ';' => self.make_token(TokenType::SemiColon),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => self.make_token(TokenType::Minus),
            '+' => self.make_token(TokenType::Plus),
            '/' => self.make_token(TokenType::Slash),
            '*' => self.make_token(TokenType::Star),
            '!' => self.token_if_match('=', TokenType::BangEqual, TokenType::Bang),
            '=' => self.token_if_match('=', TokenType::EqualEqual, TokenType::Equal),
            '<' => self.token_if_match('=', TokenType::LessEqual, TokenType::Less),
            '>' => self.token_if_match('=', TokenType::GreaterEqual, TokenType::Greater),
            '"' => self.string(),
            _ => self.error_token("Unexpected character"),
        }
    }

    fn identifier(&mut self) -> Token {
        while is_alpha(self.peek()) || self.peek().is_ascii_digit() {
            self.advance();
        }

        self.make_token(self.identifier_type())
    }

    fn identifier_type(&self) -> TokenType {
        match self.get_char(self.start) {
            'a' => self.check_keyword(1, "nd", TokenType::And),
            'c' => self.check_keyword(1, "lass", TokenType::Class),
            'e' => self.check_keyword(1, "lse", TokenType::Else),
            'f' => {
                if self.current - self.start > 1 {
                    match self.get_char(self.start + 1) {
                        'a' => self.check_keyword(2, "lse", TokenType::False),
                        'o' => self.check_keyword(2, "r", TokenType::For),
                        'u' => self.check_keyword(2, "n", TokenType::Fun),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'i' => self.check_keyword(1, "f", TokenType::If),
            'n' => self.check_keyword(1, "il", TokenType::Nil),
            'o' => self.check_keyword(1, "r", TokenType::Or),
            'p' => self.check_keyword(1, "rint", TokenType::Print),
            'r' => self.check_keyword(1, "eturn", TokenType::Return),
            's' => self.check_keyword(1, "uper", TokenType::Super),
            't' => {
                if self.current - self.start > 1 {
                    match self.get_char(self.start + 1) {
                        'h' => self.check_keyword(2, "is", TokenType::This),
                        'r' => self.check_keyword(2, "ue", TokenType::True),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'v' => self.check_keyword(1, "ar", TokenType::Var),
            'w' => self.check_keyword(1, "hile", TokenType::While),
            _ => TokenType::Identifier,
        }
    }

    fn check_keyword(&self, start: usize, rest: &str, typ: TokenType) -> TokenType {
        let s = self.start + start;
        let end = self.source.len().min(s + rest.len());
        let slice = &self.source[s..end];
        if slice == rest {
            typ
        } else {
            TokenType::Identifier
        }
    }

    fn number(&mut self) -> Token {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn string(&mut self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string");
        }

        self.advance();
        self.make_token(TokenType::String)
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() != '/' {
                        return;
                    }

                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                }
                _ => {
                    return;
                }
            }
        }
    }

    fn peek_next(&self) -> char {
        self.get_char(self.current + 1)
    }

    fn peek(&self) -> char {
        self.get_char(self.current)
    }

    fn token_if_match(
        &mut self,
        expected: char,
        if_present: TokenType,
        if_absent: TokenType,
    ) -> Token {
        if self.match_char(expected) {
            self.make_token(if_present)
        } else {
            self.make_token(if_absent)
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.get_char(self.current) != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn get_char(&self, i: usize) -> char {
        if i >= self.source.len() {
            '\0'
        } else {
            self.source.as_bytes()[i] as char
        }
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.get_char(self.current - 1)
    }

    fn is_at_end(&self) -> bool {
        self.current == self.source.len()
    }

    fn make_token(&self, typ: TokenType) -> Token {
        Token {
            typ,
            line: self.line,
            slice: RcSlice::new(self.source.clone(), self.start..self.current),
        }
    }

    fn error_token(&self, error: &'static str) -> Token {
        Token {
            typ: TokenType::Error,
            line: self.line,
            slice: RcSlice::from_string(error),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct Token {
    pub typ: TokenType,
    pub line: usize,
    pub slice: RcSlice,
}

impl Token {
    pub fn into_string(&self) -> String {
        (&self.slice).into()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    SemiColon,
    Slash,
    Star,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier,
    String,
    Number,

    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error,
    EOF,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_single_token() {
        for (s, t) in [
            ("1", TokenType::Number),
            ("1.2", TokenType::Number),
            ("\"abc\"", TokenType::String),
            ("tru", TokenType::Identifier),
            ("tr", TokenType::Identifier),
            ("t", TokenType::Identifier),
            ("(", TokenType::LeftParen),
            (")", TokenType::RightParen),
            ("{", TokenType::LeftBrace),
            ("}", TokenType::RightBrace),
            (",", TokenType::Comma),
            (".", TokenType::Dot),
            ("-", TokenType::Minus),
            ("+", TokenType::Plus),
            (";", TokenType::SemiColon),
            ("/", TokenType::Slash),
            ("*", TokenType::Star),
            ("!", TokenType::Bang),
            ("!=", TokenType::BangEqual),
            ("=", TokenType::Equal),
            ("==", TokenType::EqualEqual),
            (">", TokenType::Greater),
            (">=", TokenType::GreaterEqual),
            ("<", TokenType::Less),
            ("<=", TokenType::LessEqual),
            ("and", TokenType::And),
            ("class", TokenType::Class),
            ("else", TokenType::Else),
            ("false", TokenType::False),
            ("for", TokenType::For),
            ("fun", TokenType::Fun),
            ("if", TokenType::If),
            ("nil", TokenType::Nil),
            ("or", TokenType::Or),
            ("print", TokenType::Print),
            ("return", TokenType::Return),
            ("super", TokenType::Super),
            ("this", TokenType::This),
            ("true", TokenType::True),
            ("var", TokenType::Var),
            ("while", TokenType::While),
        ] {
            let mut scanner = Scanner::init(s.into());
            let token = scanner.token();

            assert_eq!(s, token.slice.as_str());
            assert_eq!(token.typ, t)
        }
    }
}
