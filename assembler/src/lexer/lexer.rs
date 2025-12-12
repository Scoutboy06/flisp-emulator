#![allow(unused)]

use std::{borrow::Cow, collections::VecDeque, str::Bytes};

use crate::lexer::{
    directive::parse_directive,
    instruction::parse_instruction,
    named_literal::parse_named_literal,
    symbol::Symbol,
    token::{Token, TokenKind, TokenValue},
};

pub struct Lexer<'a> {
    source: &'a str,
    bytes: Bytes<'a>,
    pos: usize,
    curr: Option<u8>,
    byte_queue: VecDeque<u8>,
    token_queue: VecDeque<Token>,
}

type LexResult<T> = Result<T, ()>;

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut bytes = source.bytes();
        let curr = bytes.next();

        Self {
            source,
            bytes,
            pos: 0,
            curr,
            byte_queue: Default::default(),
            token_queue: Default::default(),
        }
    }

    pub fn next_token(&mut self) -> Token {
        if let Some(token) = self.token_queue.pop_front() {
            token
        } else {
            self.lex_next_token()
        }
    }

    pub fn peek_token(&mut self) -> &Token {
        if self.token_queue.is_empty() {
            let token = self.lex_next_token();
            self.token_queue.push_back(token);
        }
        self.token_queue.front().unwrap()
    }

    pub fn peek_token_at(&mut self, offset: usize) -> &Token {
        while self.token_queue.len() <= offset {
            let token = self.lex_next_token();
            self.token_queue.push_back(token);
        }
        self.token_queue.get(offset).unwrap()
    }

    fn peek(&mut self, offset: usize) -> Option<u8> {
        while self.byte_queue.len() <= offset {
            if let Some(b) = self.bytes.next() {
                self.byte_queue.push_back(b);
            } else {
                return None;
            }
        }
        self.byte_queue.get(offset).cloned()
    }

    fn advance(&mut self) {
        if self.curr.is_some() {
            self.pos += 1;
        }
        if self.byte_queue.len() > 0 {
            self.curr = self.byte_queue.pop_front();
        } else {
            self.curr = self.bytes.next();
        }
    }

    fn skip_whitespace(&mut self) {
        while self.curr.is_some_and(|b| b.is_ascii_whitespace()) {
            self.advance();
        }
    }

    fn lex_next_token(&mut self) -> Token {
        use TokenKind as TK;
        use TokenValue as TV;
        self.skip_whitespace();

        if self.curr.is_none() {
            return Token::eof(self.pos);
        }

        let start = self.pos;

        let (token_kind, token_value) = match self.curr.unwrap() {
            b'#' => {
                self.advance();
                (TK::ImmediatePrefix, TV::Empty)
            }
            b'A'..=b'Z' | b'a'..=b'z' => {
                let id = self.collect_identifier();
                if let Some(instr) = parse_instruction(&id) {
                    (TK::Instruction, TV::Instruction(instr))
                } else if let Some(lit) = parse_named_literal(&id) {
                    (TK::NamedLiteral, TV::NamedLiteral(lit))
                } else if let Some(dir) = parse_directive(&id) {
                    (TK::Directive, TV::Directive(dir))
                } else {
                    (TK::Sym, TV::Sym(Symbol(id)))
                }
            }
            b'0'..=b'9' | b'$' | b'%' => {
                (TK::NumberLiteral, TV::NumberLiteral(self.parse_number()))
            }
            b';' => {
                while self.curr != Some(b'\n') {
                    self.advance();
                }
                self.advance(); // Skip \n
                (TK::Comment, TV::Empty)
            }
            b':' => {
                self.advance();
                (TK::Colon, TV::Empty)
            }
            _ => todo!(),
        };

        if token_kind == TK::Comment {
            return self.lex_next_token();
        }

        Token {
            kind: token_kind,
            value: token_value,
            start,
            end: self.pos,
        }
    }

    fn parse_number(&mut self) -> u8 {
        let mult: u8 = match self.curr.unwrap() {
            b'%' => {
                self.advance();
                2
            }
            b'$' => {
                self.advance();
                16
            }
            b'0'..=b'9' => 10,
            _ => unreachable!(),
        };
        let mut sum: u8 = 0;

        loop {
            let nxt = match self.curr {
                Some(b'0' | b'1') => self.curr.unwrap() - b'0',
                Some(b'0'..=b'9') if mult >= 10 => self.curr.unwrap() - b'0',
                Some(b'a'..=b'f') if mult == 16 => self.curr.unwrap() - b'a' + 0x10,
                Some(b'A'..=b'F') if mult == 16 => self.curr.unwrap() - b'A' + 0x10,
                _ => break,
            };

            if u8::MAX / mult - nxt < sum {
                break;
            }
            sum = sum * mult + nxt;

            self.advance();
        }

        sum
    }

    fn collect_identifier(&mut self) -> String {
        let mut id = String::new();

        while let Some(b) = self.curr {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'_' => id.push(self.curr.unwrap() as char),
                _ => break,
            }

            self.advance();
        }

        id
    }
}
