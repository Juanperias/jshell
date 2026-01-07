pub mod parser;
pub mod env;

use std::{collections::HashMap, path::{Path, PathBuf}, sync::{Arc, Mutex}};

use libc::{c_char, execve, fork, pid_t, waitpid};
use once_cell::sync::Lazy;

use std::ffi::CString;

use crate::{env::{manage_local_vars, resolve_dep}, parser::parse};

fn main() {
    let expresion = "HELLO=HELLO echo test";
    let mut parsed = parse(expresion);


    if parsed.command == String::new() {
        manage_local_vars(&parsed.env); 
    }
    
    
    // PUT ENV VARS LIKE HOME, USER, SHELL, PATH, EDITOR, PS1
    parsed.env.push(CString::new(format!("PATH={}", std::env::var("PATH").unwrap())).unwrap());


    let path = resolve_dep(&parsed.command).unwrap();

    let args: Vec<*const c_char> = parsed.args.iter().map(|x| x.as_ptr() as *const c_char).collect();
    let env: Vec<*const c_char> = parsed.env.iter().map(|x| x.as_ptr() as *const c_char).collect();

    let pid = execute_cmd(&path, &args, &env, |argv, envp, c_path| {
        let argv = argv.0;
        let envp = envp.0;
      
        unsafe { 
            execve(c_path.as_ptr(), argv, envp);
        }
    });

    let mut status = 0;
    
    unsafe {
        waitpid(pid, &mut status, 0);
    }

    if status != 0 {
        println!("command {path} failed");
    }
}

fn execute_cmd<F>(path: &str, args: &[*const c_char], env: &[*const c_char], child_code: F) -> pid_t
    where F: Fn(Argv, Envp, CString)
{

    let c_path = CString::new(path).unwrap();
    let mut argv = vec![c_path.as_ptr()];
    argv.extend_from_slice(args);
    argv.push(std::ptr::null());

    let mut envp = env.to_vec();
    envp.push(std::ptr::null());
   
    unsafe {
       let pid = fork();

       if pid == 0 {
            child_code(Argv(argv.as_ptr() as *const *const c_char), Envp(envp.as_ptr() as *const *const c_char), CString::new(path.to_string()).unwrap());
       }

       pid
    }
}

pub struct Argv(pub *const *const c_char);
pub struct Envp(pub *const *const c_char);


