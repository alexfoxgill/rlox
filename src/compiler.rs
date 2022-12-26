use crate::scanner::{Scanner, TokenType};

pub fn compile(source: String) {
    let mut scanner = Scanner::init(source);
    let mut line = 0;

    loop {
        let token = scanner.token();
        if token.line != line {
            print!("{:>4} ", token.line);
            line = token.line
        } else {
            print!("    | ")
        }

        println!("{:>2?} '{}'", token.typ, token.slice);

        if token.typ == TokenType::EOF {
            break;
        }
    }
}