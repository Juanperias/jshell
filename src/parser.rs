use std::ffi::{CString, NulError};

use peg::{error::ParseError, str::LineCol};
use thiserror::Error;

use crate::{cmd::CmdError, env::var};

peg::parser! {
    grammar command_parser() for str {
        rule _() =  quiet!{___+}
        rule __() = quiet!{ ____()+ }
        rule ___() = [' ' | '\n' | '\t']
        rule ____() = [' ' | '\t']

        pub rule program() -> RawProgram = _? items:item()** _ { RawProgram(items) }

        rule item() -> Item = command() / var() / var_assing() / string() / identifier();

        rule command() -> Item  = "./" command: identifier() {
            match command {
                Item::Iden(s) => Item::Command(format!("./{s}")),
                _ => unreachable!()
            }
        }

        rule var() -> Item = "$" var: identifier_raw() {
            Item::Var(var)
        }

        rule var_assing() -> Item = id: identifier_raw() "="  val: (string_raw() / identifier_raw()) {
            Item::VarAssign(id, val)
        }

        rule identifier() -> Item = n:identifier_raw() { Item::Iden(n) }
        rule identifier_raw() -> String = n:$(['!' | '#'..='<' | '>'..='~']+) { n.to_string() }

        rule string() -> Item
            = n:string_raw()  {
            Item::Str(n)
        }


        rule string_raw() -> String
            = q:$("\"" (['!' | '#'..='~' | ' '])* "\"") {
            let inner = &q[1..q.len()-1];
            inner.to_string()
        }

    }
}

#[derive(Debug)]
pub struct RawProgram(Vec<Item>);

#[derive(Debug, Clone)]
pub enum Item {
    Iden(String),
    Str(String),
    Command(String),
    Var(String),
    VarAssign(String, String),
}

#[derive(Debug, Clone)]
pub struct EnvInfo {
    pub var: String,
    pub val: String,
}

#[derive(Debug, Clone)]
pub struct Cmd {
    pub command: String,
    pub args: Vec<CString>,
    pub env: Vec<CString>,
}

pub fn parse(cmd: &str) -> Result<Cmd, ParserError> {
    let raw_parsed = command_parser::program(cmd)?;

    let parsed: Result<Cmd, ParserError> = {
        let mut flag = false;
        let mut parsed = Cmd {
            command: String::new(),
            args: vec![],
            env: vec![],
        };

        for x in raw_parsed.0.iter() {
            match x.clone() {
                Item::Command(s) => {
                    if !flag {
                        parsed.command = s;
                        flag = true;
                    }
                }
                Item::VarAssign(id, val) => {
                    if flag == false {
                        if &id == "?" {
                            return Err(ParserError::NoMatches(id));
                        }

                        parsed
                            .env
                            .push(CString::new(format!("{id}={val}"))?);
                    }
                }
                Item::Str(s) => {
                    if flag == false {
                        parsed.command = s;
                        flag = true;
                        continue;
                    }

                    parsed.args.push(CString::new(s)?);
                }
                Item::Iden(s) => {
                    if flag == false {
                        parsed.command = s;
                        flag = true;
                        continue;
                    }

                    parsed.args.push(CString::new(s)?);
                }
                Item::Var(s) => {
                    let val = var(&s).unwrap_or_default();

                    if !flag {
                        parsed.command = val;
                    } else {
                        parsed.args.push(CString::new(val)?);
                    }

                    flag = false;
                }
            }
        }

        Ok(parsed)
    };

    parsed
}

#[derive(Debug)]
pub enum CmdExpr {
    Command(String),
    Env(EnvInfo),
    Arg(String),
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("parse Error {0}")]
    ParseError(#[from] ParseError<LineCol>),

    #[error("no matches found {0}")]
    NoMatches(String),

    #[error("cstring error, nul error: {0}")]
    NulError(#[from] NulError),
}
