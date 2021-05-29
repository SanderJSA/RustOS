use super::types::*;
use alloc::{string::*, vec::Vec};

#[derive(Debug)]
pub struct Reader<'a> {
    line: &'a str,
    tokens: Vec<String>,
    index: usize,
}

impl<'a> Reader<'a> {
    pub fn new(line: &str) -> Reader {
        let mut reader = Reader {
            line,
            tokens: Vec::new(),
            index: 0,
        };
        while reader.line.len() != 0 {
            let token = reader.tokenize().to_string();
            reader.tokens.push(token);
        }
        reader
    }

    fn peek(&self) -> &str {
        &self.tokens[self.index]
    }

    fn pop(&mut self) -> &str {
        self.index += 1;
        &self.tokens[self.index - 1]
    }

    pub fn read_form(&mut self) -> MalType {
        match self.peek() {
            "(" => self.read_list(),
            _ => self.read_atom(),
        }
    }

    fn read_list(&mut self) -> MalType {
        self.pop();
        let mut list = Vec::new();
        loop {
            match self.peek() {
                ")" => {
                    self.pop();
                    return MalType::List(list);
                }
                _ => list.push(self.read_form()),
            }
        }
    }

    fn read_atom(&mut self) -> MalType {
        match self.pop() {
            "nil" => MalType::Nil,
            "true" => MalType::True,
            "false" => MalType::False,
            token => {
                if let Ok(num) = token.parse() {
                    MalType::Number(num)
                } else {
                    MalType::Symbol(String::from(token))
                }
            }
        }
    }

    /// LEXER

    fn tokenize(&mut self) -> &str {
        self.trim_whitespace();

        if self.line.len() >= 2 && &self.line[..2] == "~@" {
            return self.extract_token(2);
        }

        match &self.line[0..1] {
            "[" | "]" | "{" | "}" | "(" | ")" | "'" | "`" | "~" | "^" | "@" => {
                self.extract_token(1)
            }
            "\"" => self.tokenize_string(),
            ";" => self.extract_token(self.line.len()),
            _ => self.lex_special(),
        }
    }

    fn tokenize_string(&mut self) -> &str {
        let mut i = 1;
        while i < self.line.len() {
            if self.line.len() >= 2 && &self.line[i..i + 2] == "\\\"" {
                i += 1;
            } else if &self.line[i..i + 1] == "\"" {
                i += 1;
                break;
            }
            i += 1;
        }
        self.extract_token(i)
    }

    fn trim_whitespace(&mut self) {
        while self.line.len() != 0 {
            let c = self.line.chars().nth(0).unwrap();
            if c != ' ' && c != '\t' && c != ',' {
                break;
            }
            self.line = &self.line[1..];
        }
    }

    fn extract_token(&mut self, end_index: usize) -> &str {
        let token = &self.line[..end_index];
        self.line = &self.line[end_index..];
        token
    }

    fn lex_special(&mut self) -> &str {
        let mut i = 0;
        while i < self.line.len() {
            match &self.line[i..i + 1] {
                " " | "\t" | "[" | "]" | "{" | "}" | "(" | ")" | "'" | "\"" | "`" | "," | ";" => {
                    break;
                }
                _ => {
                    i += 1;
                }
            }
        }
        self.extract_token(i)
    }
}
