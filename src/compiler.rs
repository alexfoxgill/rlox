use std::rc::Rc;

use crate::{
    chunk::{Chunk, OpCode},
    debug::disassemble_chunk,
    scanner::{Scanner, Token, TokenType},
    string_intern::StringInterner,
    value::{Object, Value}, vm::VM,
};

pub fn compile(source: Rc<str>) -> Option<VM> {
    let scanner = Scanner::init(source);
    Parser::new(scanner).compile()
}

struct Parser {
    scanner: Scanner,
    chunk: Chunk,
    strings: StringInterner,
    locals: Vec<Local>,
    scope_depth: usize,
    current: Option<Token>,
    previous: Option<Token>,
    had_error: bool,
    panic_mode: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Local {
    name: Token,
    depth: LocalDepth
}
impl Local {
    fn initialize(&mut self, depth: usize) {
        self.depth = LocalDepth::Initialized(depth);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
enum LocalDepth {
    Uninitialized,
    Initialized(usize)
}

impl Parser {
    fn new(scanner: Scanner) -> Parser {
        Parser {
            scanner,
            chunk: Chunk::new(),
            strings: StringInterner::with_capacity(16),
            locals: Vec::new(),
            scope_depth: 0,
            current: None,
            previous: None,
            had_error: false,
            panic_mode: false,
        }
    }

    fn compile(mut self) -> Option<VM> {
        self.advance();

        while !self.match_token(TokenType::EOF) {
            self.declaration();
        }

        self.end_compiler();

        if self.had_error {
            None
        } else {
            Some(VM::new(self.chunk, self.strings))
        }
    }

    fn end_compiler(&mut self) {
        self.emit_return();

        if !self.had_error {
            disassemble_chunk(&self.chunk, "code", &self.strings)
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
                let msg = token.into_string();
                self.current = Some(token);
                self.error_at_current(&msg);
            }
        }
    }

    fn check(&self, typ: TokenType) -> bool {
        self.current().typ == typ
    }

    fn match_token(&mut self, typ: TokenType) -> bool {
        let matched = self.check(typ);
        if matched {
            self.advance();
        }
        matched
    }

    fn declaration(&mut self) {
        if self.match_token(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn var_declaration(&mut self) {
        let addr = self.parse_variable("Expect variable name");

        if self.match_token(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_op_code(OpCode::Nil);
        }

        self.consume(
            TokenType::SemiColon,
            "Expect ';' after variable declaration",
        );

        self.define_variable(addr);
    }

    fn statement(&mut self) {
        if self.match_token(TokenType::Print) {
            self.print_statement();
        } else if self.match_token(TokenType::If) {
            self.if_statement();
        } else if self.match_token(TokenType::While) {
            self.while_statement();
        } else if self.match_token(TokenType::For) {
            self.for_statement();
        } else if self.match_token(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition");

        let then_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.emit_op_code(OpCode::Pop);

        self.statement();
        let else_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(then_jump);

        self.emit_op_code(OpCode::Pop);

        if self.match_token(TokenType::Else) {
            self.statement();
        }

        self.patch_jump(else_jump);
    }

    fn while_statement(&mut self) {
        let loop_start = self.chunk.code.len();
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition");
        
        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_op_code(OpCode::Pop);
        self.statement();

        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_op_code(OpCode::Pop);
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'");

        if !self.match_token(TokenType::SemiColon) {
            if self.match_token(TokenType::Var) {
                self.var_declaration();
            } else {
                self.expression_statement();
            }
        } 

        let mut loop_start = self.chunk.code.len();
        let mut exit_jump = None;
        if !self.match_token(TokenType::SemiColon) {
            self.expression();
            self.consume(TokenType::SemiColon, "Expect ';' after loop");

            exit_jump = Some(self.emit_jump(OpCode::JumpIfFalse));
            self.emit_op_code(OpCode::Pop); // pop the condition
        }

        if !self.match_token(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump);
            let increment_start = self.chunk.code.len();

            self.expression();
            self.emit_op_code(OpCode::Pop); // pop the increment expression
            self.consume(TokenType::RightParen, "Expect ')' after for clauses");

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement();
        self.emit_loop(loop_start);

        if let Some(exit_jump) = exit_jump {
            self.patch_jump(exit_jump);
            self.emit_op_code(OpCode::Pop); // pop the condition again
        }

        self.end_scope();
    }

    fn define_variable(&mut self, addr: u8) {
        if self.scope_depth > 0 {
            if let Some(x) = self.locals.last_mut() {
                x.initialize(self.scope_depth)
            }
        } else {
            self.emit_bytes(OpCode::DefineGlobal as u8, addr)
        }

    }

    fn parse_variable(&mut self, error: &str) -> u8 {
        self.consume(TokenType::Identifier, error);

        self.declare_variable();
        if self.scope_depth > 0 {
            return 0;
        }

        self.identifier_constant(self.previous())
    }

    fn declare_variable(&mut self) {
        if self.scope_depth == 0 {
            return;
        }

        let name = self.previous();

        let existing = self.locals.iter().rev()
            .take_while(|local| match local.depth {
                LocalDepth::Uninitialized => false,
                LocalDepth::Initialized(d) => d >= self.scope_depth,
            })
            .any(|local| local.name.string_eq(&name));

        if existing {
            self.error("A variable with this name already exists in this scope");
        }

        self.add_local(name);
    }

    fn add_local(&mut self, name: Token) {
        if self.locals.len() == u8::MAX as usize {
            self.error("Too many local variables in function");
            return;
        }
        self.locals.push(Local {
            name,
            depth: LocalDepth::Uninitialized
        })
    }

    fn identifier_constant(&mut self, token: Token) -> u8 {
        let value = self.make_string_id(token.into_string());
        self.make_constant(value)
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Expect ';' after value");
        self.emit_op_code(OpCode::Print);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Expect ';' after expression");
        self.emit_op_code(OpCode::Pop);
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::EOF) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block");
    }

    fn end_scope(&mut self) {
        let to_pop = self.locals.iter().rev()
            .take_while(|local| match local.depth {
                LocalDepth::Uninitialized => true,
                LocalDepth::Initialized(d) => d >= self.scope_depth,
            })
            .count();

        self.scope_depth -= 1;

        for _ in 0..to_pop {
            self.emit_op_code(OpCode::Pop);
            self.locals.pop();
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self) {
        let value: f64 = self.previous().slice.parse().unwrap();

        self.emit_constant(Value::Number(value));
    }

    fn unary(&mut self) {
        let op_type = self.previous().typ;

        self.parse_precedence(Precedence::Unary);

        match op_type {
            TokenType::Minus => self.emit_op_code(OpCode::Negate),
            TokenType::Bang => self.emit_op_code(OpCode::Not),
            _ => (),
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
            _ => (),
        }
    }

    fn get_rule(&self, op_type: TokenType) -> ParseRule {
        use Precedence::*;
        use TokenType::*;
        match op_type {
            LeftParen => ParseRule::new().prefix(|p, _| p.grouping()),
            RightParen => ParseRule::new(),
            LeftBrace => ParseRule::new(),
            RightBrace => ParseRule::new(),
            Comma => ParseRule::new(),
            Dot => ParseRule::new(),
            Minus => ParseRule::prec(Term)
                .prefix(|p, _| p.unary())
                .infix(|p| p.binary()),
            Plus => ParseRule::prec(Term).infix(|p| p.binary()),
            SemiColon => ParseRule::new(),
            Slash => ParseRule::prec(Factor).infix(|p| p.binary()),
            Star => ParseRule::prec(Factor).infix(|p| p.binary()),
            Bang => ParseRule::new().prefix(|p, _| p.unary()),
            BangEqual => ParseRule::prec(Equality).infix(|p| p.binary()),
            Equal => ParseRule::new(),
            EqualEqual => ParseRule::prec(Equality).infix(|p| p.binary()),
            Greater => ParseRule::prec(Comparison).infix(|p| p.binary()),
            GreaterEqual => ParseRule::prec(Comparison).infix(|p| p.binary()),
            Less => ParseRule::prec(Comparison).infix(|p| p.binary()),
            LessEqual => ParseRule::prec(Comparison).infix(|p| p.binary()),
            Identifier => ParseRule::new().prefix(|p, can_assign| p.variable(can_assign)),
            String => ParseRule::new().prefix(|p, _| p.string()),
            Number => ParseRule::new().prefix(|p, _| p.number()),
            TokenType::And => ParseRule::prec(Precedence::And).infix(|p| p.and()),
            Class => ParseRule::new(),
            Else => ParseRule::new(),
            False => ParseRule::new().prefix(|p, _| p.literal()),
            For => ParseRule::new(),
            Fun => ParseRule::new(),
            If => ParseRule::new(),
            Nil => ParseRule::new().prefix(|p, _| p.literal()),
            TokenType::Or => ParseRule::prec(Precedence::Or).infix(|p| p.or()),
            Print => ParseRule::new(),
            Return => ParseRule::new(),
            Super => ParseRule::new(),
            This => ParseRule::new(),
            True => ParseRule::new().prefix(|p, _| p.literal()),
            Var => ParseRule::new(),
            While => ParseRule::new(),
            Error => ParseRule::new(),
            EOF => ParseRule::new(),
        }
    }

    fn string(&mut self) {
        let str = String::from(self.previous().slice.trim_matches('\"'));
        let obj = self.make_string(str);
        self.emit_constant(obj)
    }

    fn make_string(&mut self, str: String) -> Value {
        let (_, str) = self.strings.intern(&str);
        let obj = Rc::new(Object::String(str));
        Value::Object(obj)
    }

    fn make_string_id(&mut self, str: String) -> Value {
        let (id, _) = self.strings.intern(&str);
        let obj = Rc::new(Object::StringId(id));
        Value::Object(obj)
    }

    fn literal(&mut self) {
        match self.previous().typ {
            TokenType::False => self.emit_op_code(OpCode::False),
            TokenType::Nil => self.emit_op_code(OpCode::Nil),
            TokenType::True => self.emit_op_code(OpCode::True),
            _ => (),
        }
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.previous(), can_assign)
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let (arg, get, set) = self.resolve_local(&name)
            .map(|arg| (arg, OpCode::GetLocal, OpCode::SetLocal))
            .unwrap_or_else(|| {
                let arg = self.identifier_constant(name);
                (arg, OpCode::GetGlobal, OpCode::SetGlobal)
            });

        if can_assign && self.match_token(TokenType::Equal) {
            self.expression();
            self.emit_bytes(set as u8, arg)
        } else {
            self.emit_bytes(get as u8, arg)
        }
    }

    fn resolve_local(&mut self, name: &Token) -> Option<u8> {
        let (i, depth) = self.locals.iter().enumerate().rev()
            .find_map(|(i, local)| {
                if local.name.string_eq(name) {
                    Some((i as u8, local.depth))
                } else {
                    None
                }
            })?;

        if depth == LocalDepth::Uninitialized {
            self.error("Can't read local variable in its own initializer")
        }

        Some(i)
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression")
    }

    pub fn and(&mut self) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.emit_op_code(OpCode::Pop);

        self.parse_precedence(Precedence::And);

        self.patch_jump(end_jump);
    }

    pub fn or(&mut self) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse);
        let end_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(else_jump);
        self.emit_op_code(OpCode::Pop);

        self.parse_precedence(Precedence::Or);

        self.patch_jump(end_jump);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        let rule = self.get_rule(self.previous().typ);

        if let Some(prefix) = rule.prefix {
            let can_assign = precedence <= Precedence::Assignment;
            prefix(self, can_assign);

            while self.get_rule(self.current().typ).precedence >= precedence {
                self.advance();
                let infix = self.get_rule(self.previous().typ).infix.unwrap();
                infix(self);
            }

            if can_assign && self.match_token(TokenType::Equal) {
                self.error("Invalid assignment target")
            }
        } else {
            self.error("Expect expression");
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

    fn emit_jump(&mut self, instruction: OpCode) -> usize {
        self.emit_op_code(instruction);
        self.emit_byte(0xFF);
        self.emit_byte(0xFF);
        self.chunk.code.len() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.code.len() - offset - 2;

        if jump > u16::MAX as usize {
            self.error("Too much code to jump over")
        }

        self.chunk.code[offset] = ((jump >> 8) & 0xFF) as u8;
        self.chunk.code[offset + 1] = (jump & 0xFF) as u8;
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

    fn emit_loop(&mut self, start: usize) {
        self.emit_op_code(OpCode::Loop);

        let offset = self.chunk.code.len() - start + 2;
        if offset > (u16::MAX as usize) {
            self.error("Loop body too large");
        }

        self.emit_short(offset as u16);
    }

    fn emit_return(&mut self) {
        self.emit_op_code(OpCode::Return)
    }

    fn emit_op_code(&mut self, op_code: OpCode) {
        self.emit_byte(op_code as u8)
    }

    fn emit_short(&mut self, short: u16) {
        let b = ((short >> 8) & 0xFF) as u8;
        self.emit_byte(b);
        let b = (short & 0xFF) as u8;
        self.emit_byte(b);
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

    fn current(&self) -> Token {
        self.current.as_ref().unwrap().clone()
    }

    fn previous(&self) -> Token {
        self.previous.as_ref().unwrap().clone()
    }

    fn synchronize(&mut self) {
        use TokenType::*;
        self.panic_mode = false;

        while self.current().typ != EOF {
            if self.previous().typ == SemiColon {
                return;
            }

            match self.current().typ {
                Class | Fun | Var | For | If | While | Print | Return => {
                    return;
                }
                _ => (),
            }
        }

        self.advance();
    }
}

fn print_error(token: Token, message: &str) {
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
    Primary,
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
    prefix: Option<Box<dyn Fn(&mut Parser, bool) -> ()>>,
    infix: Option<Box<dyn Fn(&mut Parser) -> ()>>,
    precedence: Precedence,
}
impl ParseRule {
    fn new() -> ParseRule {
        ParseRule::prec(Precedence::None)
    }

    fn prec(precedence: Precedence) -> ParseRule {
        ParseRule {
            prefix: None,
            infix: None,
            precedence,
        }
    }

    fn prefix(self, prefix: impl Fn(&mut Parser, bool) -> () + 'static) -> ParseRule {
        ParseRule {
            prefix: Some(Box::new(prefix)),
            infix: self.infix,
            precedence: self.precedence,
        }
    }

    fn infix(self, infix: impl Fn(&mut Parser) -> () + 'static) -> ParseRule {
        ParseRule {
            prefix: self.prefix,
            infix: Some(Box::new(infix)),
            precedence: self.precedence,
        }
    }
}
