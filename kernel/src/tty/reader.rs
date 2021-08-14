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
        while !reader.line.is_empty() {
            let token = reader.tokenize();
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
            "true" => MalType::Bool(true),
            "false" => MalType::Bool(false),
            string if string.starts_with('\"') => {
                MalType::String(string[1..string.len() - 1].to_string())
            }
            "'" => MalType::List(alloc::vec![MalType::quote(), self.read_form()]),
            "`" => MalType::List(alloc::vec![MalType::quasiquote(), self.read_form()]),
            "~" => MalType::List(alloc::vec![MalType::unquote(), self.read_form()]),
            "~@" => MalType::List(alloc::vec![MalType::splice_unquote(), self.read_form()]),
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

    fn tokenize(&mut self) -> String {
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

    fn tokenize_string(&mut self) -> String {
        let mut string = String::from('"');
        let mut escaped = false;
        for (i, c) in self.line.chars().enumerate().skip(1) {
            if escaped {
                match c {
                    'n' => string.push('\n'),
                    '"' => string.push('\"'),
                    '\\' => string.push('\\'),
                    c => {
                        string.push('\\');
                        string.push(c)
                    }
                }
                escaped = false;
                continue;
            }
            if c == '\\' {
                escaped = true;
                continue;
            }
            string.push(c);
            if c == '"' {
                self.line = &self.line[i + 1..];
                break;
            }
        }
        string
    }

    fn trim_whitespace(&mut self) {
        while let Some(c) = self.line.chars().next() {
            match c {
                ' ' | '\t' | '\n' | ',' => self.line = &self.line[1..],
                _ => break,
            }
        }
    }

    fn extract_token(&mut self, end_index: usize) -> String {
        let (token, remaning) = self.line.split_at(end_index);
        self.line = remaning;
        token.to_string()
    }

    fn lex_special(&mut self) -> String {
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
