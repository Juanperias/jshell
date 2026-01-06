use std::ffi::CString;

peg::parser! {
    grammar command_parser() for str {
        rule _() =  quiet!{___+}
        rule __() = quiet!{ ____()+ }
        rule ___() = [' ' | '\n' | '\t'] 
        rule ____() = [' ' | '\t']
        
        pub rule program() -> RawProgram = _? items:item()** _ { RawProgram(items) }

        rule item() -> Item = command() / string() / identifier();
    
        rule command() -> Item  = "./" command: identifier() {
            match command {
                Item::Iden(s) => Item::Command(format!("./{s}")),
                _ => unreachable!()
            }
        }

        rule identifier() -> Item = n:$(['!' | '#'..='~']+) {
            Item::Iden(n.to_string())
        }

        rule string() -> Item
            = q:$("\"" (['!' | '#'..='~' | ' '])* "\"") {
            let inner = &q[1..q.len()-1];
            Item::Str(inner.to_string())
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
}

#[derive(Debug, Clone)]
pub struct EnvInfo {
    pub var: String,
    pub val: String
}

#[derive(Debug, Clone)]
pub struct Cmd {
    pub command: String,
    pub args: Vec<CString>,
    pub env: Vec<CString>
}


pub fn parse(cmd: &str) -> Cmd {
    let raw_parsed = command_parser::program(cmd).unwrap();

    let (_, parsed) = raw_parsed.0.iter().fold((true, Cmd {
        command: String::new(),
        args: vec![],
        env: vec![],
    }), |mut acc, x| {
        match x.clone() {
            Item::Command(s) => {
                if acc.0 {
                  acc.1.command = s;
                } else {
                    acc.1.args.push(CString::new(s).unwrap());
                }
                acc.0 = false;
            },
            Item::Str(s) => {
                if s.contains("=") && acc.0 {
                    // REMOVE THE UNWRAP
                    acc.1.env.push(CString::new(s).unwrap());
                } else  {
                    if acc.0 == true {
                        acc.1.command = s;
                    } else {
                        acc.1.args.push(CString::new(s).unwrap());
                    }
                    
                    acc.0 = false;
                }
            },
            Item::Iden(s) => {
                if s.contains("=") && acc.0 {
                    acc.1.env.push(CString::new(s).unwrap());
                }  else  {
                    if acc.0 == true {
                        acc.1.command = s;
                    } else {
                        acc.1.args.push(CString::new(s).unwrap());
                    }
                    
                    acc.0 = false;
                }
               
            }, 
        }        

        acc
    });

    parsed 
}

#[derive(Debug)]
pub enum CmdExpr {
    Command(String),
    Env(EnvInfo),
    Arg(String),
}
