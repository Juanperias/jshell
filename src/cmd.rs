use std::ffi::CString;

use libc::{c_char, pid_t, execve, fork, waitpid, _exit};

use crate::{env::{manage_local_vars, resolve_dep}, parser::parse};



pub fn run_expr(expr: &str) {
    let mut expr = parse(expr);

    if &expr.command == "" {
      manage_local_vars(&expr.env);

       return;
    }

    let path = match resolve_dep(&expr.command) {
        Some(v) => v,
        None => {
            println!("Command {} not found", &expr.command);
            return;
        }
    };

    expr.env.push(CString::new(format!("PATH={}", std::env::var("PATH").unwrap())).unwrap());
    expr.env.push(CString::new(format!("TERM={}", std::env::var("TERM").unwrap())).unwrap());

    let args: Vec<*const c_char> = expr.args.iter().map(|x| x.as_ptr() as *const c_char).collect();
    let env: Vec<*const c_char> = expr.env.iter().map(|x| x.as_ptr() as *const c_char).collect();

    let pid = execute_cmd(&path, &args, &env);

    let mut status = 0;
    
    unsafe {
        waitpid(pid, &mut status, 0);
    }

    if status != 0 {
        println!("command {} failed", &expr.command);
    }
}


fn execute_cmd(path: &str, args: &[*const c_char], env: &[*const c_char]) -> pid_t
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
           execve(c_path.as_ptr(), argv.as_slice().as_ptr() as *const *const c_char, envp.as_slice().as_ptr() as *const *const c_char);
           _exit(-1);
       }

       pid
    }
}

pub struct Argv(pub *const *const c_char);
pub struct Envp(pub *const *const c_char);


