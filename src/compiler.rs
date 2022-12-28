use crate::{scanner::{Scanner, TokenType, Token}, chunk::{Chunk, OpCode}, value::{Value, Object}, debug::disassemble_chunk, string_intern::StringInterner};

pub fn compile(source: &str, chunk: &mut Chunk, strings: &mut StringInterner) -> bool {
    let scanner = Scanner::init(source);

    let mut parser = Parser::new(scanner, chunk, strings);

    parser.advance();
    parser.expression();
    parser.consume(TokenType::EOF, "Expect end of expression.");
    parser.end_compiler();

    !parser.had_error
}

struct Parser<'c, 's> {
    scanner: Scanner<'s>,
    chunk: &'c mut Chunk,
    strings: &'c mut StringInterner,
    current: Option<Token<'s>>,
    previous: Option<Token<'s>>,
    had_error: bool,
    panic_mode: bool
}

impl<'c, 's> Parser<'c, 's> {
    fn new(scanner: Scanner<'s>, chunk: &'c mut Chunk, strings: &'c mut StringInterner) -> Parser<'c, 's> {
        Parser {
            scanner,
            chunk,
            strings,
            current: None,
            previous: None,
            had_error: false,
            panic_mode: false
        }
    }

    fn end_compiler(&mut self) {
        self.emit_return();

        if !self.had_error {
            disassemble_chunk(self.chunk, "code")
        }
    }

    fn advance(&mut self) {
        self.previous = std::mem::take(&mut self.current);

        loop {
            let token = self.scanner.token();

            if token.typ != TokenType::Error {
                self.current = Some(token);
                break;
            } else {
                let msg = token.slice;
                self.current = Some(token);
                self.error_at_current(msg);
            }
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self) {
        let value: f64 = self.previous().slice.parse().unwrap();

        self.emit_constant(Value::Number(value))
    }

    fn unary(&mut self) {
        let op_type = self.previous().typ;

        self.parse_precedence(Precedence::Unary);

        match op_type {
            TokenType::Minus => self.emit_op_code(OpCode::Negate),
            TokenType::Bang => self.emit_op_code(OpCode::Not),
            _ => ()
        }
    }

    fn binary(&mut self) {
        let op_type = self.previous().typ;
        let rule = self.get_rule(op_type);

        self.parse_precedence(rule.precedence.next());

        match op_type {
            TokenType::BangEqual => self.emit_bytes(OpCode::Equal as u8, OpCode::Not as u8),
            TokenType::EqualEqual => self.emit_op_code(OpCode::Equal),
            TokenType::Greater => self.emit_op_code(OpCode::Greater),
            TokenType::GreaterEqual => self.emit_bytes(OpCode::Less as u8, OpCode::Not as u8),
            TokenType::Less => self.emit_op_code(OpCode::Less),
            TokenType::LessEqual => self.emit_bytes(OpCode::Greater as u8, OpCode::Not as u8),
            TokenType::Plus => self.emit_op_code(OpCode::Add),
            TokenType::Minus => self.emit_op_code(OpCode::Subtract),
            TokenType::Star => self.emit_op_code(OpCode::Multiply),
            TokenType::Slash => self.emit_op_code(OpCode::Divide),
            _ => ()
        }
    }

    fn get_rule(&self, op_type: TokenType) -> ParseRule {
        use Precedence::*;
        use TokenType::*;
        match op_type {
            LeftParen => ParseRule::new().prefix(|p| p.grouping()),
            RightParen => ParseRule::new(),
            LeftBrace => ParseRule::new(),
            RightBrace => ParseRule::new(),
            Comma => ParseRule::new(),
            Dot => ParseRule::new(),
            Minus => ParseRule::prec(Term).prefix(|p| p.unary()).infix(|p| p.binary()),
            Plus => ParseRule::prec(Term).infix(|p| p.binary()),
            SemiColon => ParseRule::new(),
            Slash => ParseRule::prec(Factor).infix(|p| p.binary()),
            Star => ParseRule::prec(Factor).infix(|p| p.binary()),
            Bang => ParseRule::new().prefix(|p| p.unary()),
            BangEqual => ParseRule::prec(Equality).infix(|p| p.binary()),
            Equal => ParseRule::new(),
            EqualEqual => ParseRule::prec(Equality).infix(|p| p.binary()),
            Greater => ParseRule::prec(Comparison).infix(|p| p.binary()),
            GreaterEqual => ParseRule::prec(Comparison).infix(|p| p.binary()),
            Less => ParseRule::prec(Comparison).infix(|p| p.binary()),
            LessEqual => ParseRule::prec(Comparison).infix(|p| p.binary()),
            Identifier => ParseRule::new(),
            String => ParseRule::new().prefix(|p| p.string()),
            Number => ParseRule::new().prefix(|p| p.number()),
            TokenType::And => ParseRule::new(),
            Class => ParseRule::new(),
            Else => ParseRule::new(),
            False => ParseRule::new().prefix(|p| p.literal()),
            For => ParseRule::new(),
            Fun => ParseRule::new(),
            If => ParseRule::new(),
            Nil => ParseRule::new().prefix(|p| p.literal()),
            TokenType::Or => ParseRule::new(),
            Print => ParseRule::new(),
            Return => ParseRule::new(),
            Super => ParseRule::new(),
            This => ParseRule::new(),
            True => ParseRule::new().prefix(|p| p.literal()),
            Var => ParseRule::new(),
            While => ParseRule::new(),
            Error => ParseRule::new(),
            EOF => ParseRule::new(),
        }
    }

    fn string(&mut self) {
        let str = String::from(self.previous().slice.trim_matches('\"'));
        let (_, str) = self.strings.intern(&str);
        let obj = Box::new(Object::String(str));
        self.emit_constant(Value::Object(obj))
    }

    fn literal(&mut self) {
        match self.previous().typ {
            TokenType::False => self.emit_op_code(OpCode::False),
            TokenType::Nil => self.emit_op_code(OpCode::Nil),
            TokenType::True => self.emit_op_code(OpCode::True),
            _ => ()
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression")
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        let rule = self.get_rule(self.previous().typ);

        if let Some(prefix) = rule.prefix {
            prefix(self);

            while precedence <= self.get_rule(self.current().typ).precedence {
                self.advance();
                let infix = self.get_rule(self.previous().typ).infix.unwrap();
                infix(self);
            }

        } else {
            self.error("Expect expression");
            return;
        }
    }

    fn consume(&mut self, typ: TokenType, message: &str) {
        if let Some(current) = self.current.as_ref() {
            if current.typ == typ {
                self.advance();
                return;
            }
        }

        self.error_at_current(message)
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(OpCode::Constant as u8, constant);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let c = self.chunk.add_constant(value);
        c.try_into().unwrap_or_else(|_| {
            self.error("Too many constants in one chunk");
            0
        })
    }

    fn emit_return(&mut self) {
        self.emit_op_code(OpCode::Return)
    }

    fn emit_op_code(&mut self, op_code: OpCode) {
        self.emit_byte(op_code as u8)
    }

    fn emit_byte(&mut self, byte: u8) {
        let line = self.previous().line;
        self.chunk.write(byte, line)
    }

    fn emit_bytes(&mut self, a: u8, b: u8) {
        let line = self.previous().line;
        self.chunk.write(a, line);
        self.chunk.write(b, line);
    }

    fn error_at_current(&mut self, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        print_error(self.current(), message);
        self.had_error = true;
    }

    fn error(&mut self, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        print_error(self.previous(), message);
        self.had_error = true;
    }

    fn current(&self) -> &Token {
        self.current.as_ref().unwrap()
    }

    fn previous(&self) -> &Token {
        self.previous.as_ref().unwrap()
    }
}

fn print_error(token: &Token, message: &str) {
    eprint!("[line {}] Error", token.line);

    if token.typ == TokenType::EOF {
        eprint!(" at end");
    } else if token.typ == TokenType::Error {
        // ...
    } else {
        eprint!(" at '{}'", token.slice);
    }

    eprintln!(": {message}");
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary
}

impl Precedence {
    fn next(&self) -> Precedence {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
        }
    }
}

struct ParseRule {
    prefix: Option<Box<dyn Fn(&mut Parser) -> ()>>,
    infix: Option<Box<dyn Fn(&mut Parser) -> ()>>,
    precedence: Precedence
}
impl ParseRule {
    fn new() -> ParseRule {
        ParseRule::prec(Precedence::None)
    }

    fn prec(precedence: Precedence) -> ParseRule {
        ParseRule {
            prefix: None,
            infix: None,
            precedence
        }
        
    }

    fn prefix(self, prefix: impl Fn(&mut Parser) -> () + 'static) -> ParseRule {
        ParseRule {
            prefix: Some(Box::new(prefix)),
            infix: self.infix,
            precedence: self.precedence
        }
    }

    fn infix(self, infix: impl Fn(&mut Parser) -> () + 'static) -> ParseRule {
        ParseRule {
            prefix: self.prefix,
            infix: Some(Box::new(infix)),
            precedence: self.precedence
        }
    }
}

