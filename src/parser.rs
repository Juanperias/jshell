use std::ffi::CString;

use crate::env::var;

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

pub fn parse(cmd: &str) -> Cmd {
    let raw_parsed = command_parser::program(cmd).unwrap();

    let (_, parsed) = raw_parsed.0.iter().fold(
        (
            false,
            Cmd {
                command: String::new(),
                args: vec![],
                env: vec![],
            },
        ),
        |mut acc, x| {
            match x.clone() {
                Item::Command(s) => {
                    if acc.0 == false {
                        acc.1.command = s;
                        acc.0 = true;
                    }
                }
                Item::VarAssign(id, val) => {
                    if acc.0 == false {
                        if &id == "?" {
                            // todo: remove this panic
                            panic!("invalid assing");
                        }

                        acc.1.env.push(CString::new(format!("{id}={val}")).unwrap());
                    }
                }
                Item::Str(s) => {
                    if acc.0 == false {
                        acc.1.command = s;
                        acc.0 = true;
                        return acc;
                    }

                    acc.1.args.push(CString::new(s).unwrap());
                }
                Item::Iden(s) => {
                    if acc.0 == false {
                        acc.1.command = s;
                        acc.0 = true;
                        return acc;
                    }

                    acc.1.args.push(CString::new(s).unwrap());
                }
                Item::Var(s) => {
                    let val = var(&s).unwrap_or_default();

                    if !acc.0 {
                        acc.1.command = val;
                    } else {
                        acc.1.args.push(CString::new(val).unwrap());
                    }

                    acc.0 = false;
                }
            }

            acc
        },
    );

    parsed
}

#[derive(Debug)]
pub enum CmdExpr {
    Command(String),
    Env(EnvInfo),
    Arg(String),
}
