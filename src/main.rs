pub mod build_in;
pub mod cmd;
pub mod env;
pub mod parser;
pub mod posix;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use libc::{c_char, execve, fork, pid_t, waitpid};
use once_cell::sync::Lazy;
use rustyline::{DefaultEditor, config::Configurer};

use std::ffi::CString;

use crate::{
    cmd::run_expr,
    env::{manage_local_vars, resolve_dep},
    parser::parse,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rl = DefaultEditor::new()?;

    loop {
        let readline = rl.readline(">> ")?;

        if let Err(err) = run_expr(&readline) {
            eprintln!("jshell: {}", err);
        }
    }
}
