
pub mod parser;
pub mod env;
pub mod cmd;

use std::{collections::HashMap, path::{Path, PathBuf}, sync::{Arc, Mutex}};

use libc::{c_char, execve, fork, pid_t, waitpid};
use once_cell::sync::Lazy;
use rustyline::{config::Configurer, DefaultEditor};

use std::ffi::CString;

use crate::{cmd::run_expr, env::{manage_local_vars, resolve_dep}, parser::parse};

fn main() {
    let mut rl = DefaultEditor::new().unwrap();

    
    loop {
        let readline = rl.readline(">> ").unwrap();

        run_expr(&readline);   
    }
}

