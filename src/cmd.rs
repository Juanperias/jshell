use std::ffi::CString;

use libc::{
    _exit, WEXITSTATUS, WIFEXITED, WIFSIGNALED, WTERMSIG, c_char, execve, fork, pid_t, waitpid,
};

use crate::{
    env::{Dep, manage_local_vars, resolve_dep, set_local_var},
    parser::parse,
    posix::POSIX_NOT_FOUND,
};

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
            set_local_var("?", POSIX_NOT_FOUND.to_string());
            return;
        }
    };

    expr.env
        .push(CString::new(format!("PATH={}", std::env::var("PATH").unwrap())).unwrap());
    expr.env
        .push(CString::new(format!("TERM={}", std::env::var("TERM").unwrap())).unwrap());

    match path {
        Dep::Path(s) => {
            let args: Vec<*const c_char> = expr
                .args
                .iter()
                .map(|x| x.as_ptr() as *const c_char)
                .collect();
            let env: Vec<*const c_char> = expr
                .env
                .iter()
                .map(|x| x.as_ptr() as *const c_char)
                .collect();

            let pid = execute_cmd(&s, &args, &env);

            let mut status = 0;

            unsafe {
                waitpid(pid, &mut status, 0);
            }

            let exit = {
                if WIFEXITED(status) {
                    WEXITSTATUS(status)
                } else if WIFSIGNALED(status) {
                    128 + WTERMSIG(status)
                } else {
                    status
                }
            };

            set_local_var("?", exit.to_string());
        }
        Dep::BuildIn(build_in) => {
            let status = build_in.run(&expr.args, &expr.env);

            set_local_var("?", status.to_string());
        }
    }
}

fn execute_cmd(path: &str, args: &[*const c_char], env: &[*const c_char]) -> pid_t {
    let c_path = CString::new(path).unwrap();
    let mut argv = vec![c_path.as_ptr()];
    argv.extend_from_slice(args);
    argv.push(std::ptr::null());

    let mut envp = env.to_vec();
    envp.push(std::ptr::null());

    unsafe {
        let pid = fork();

        if pid == 0 {
            execve(
                c_path.as_ptr(),
                argv.as_slice().as_ptr() as *const *const c_char,
                envp.as_slice().as_ptr() as *const *const c_char,
            );
            _exit(-1);
        }

        pid
    }
}

pub struct Argv(pub *const *const c_char);
pub struct Envp(pub *const *const c_char);
