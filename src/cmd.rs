use std::{env::VarError, ffi::{CString, NulError}};

use libc::{
    _exit, WEXITSTATUS, WIFEXITED, WIFSIGNALED, WTERMSIG, c_char, execve, fork, pid_t, waitpid,
};
use thiserror::Error;

use crate::{
    build_in::BuildInError, env::{manage_local_vars, resolve_dep, set_exit_status, set_local_var, Dep}, parser::{parse, ParserError}, posix::POSIX_NOT_FOUND
};

#[derive(Debug, Error)]
pub enum CmdError {
    #[error("command not found: {0}")]
    CommandNotFound(String),

    #[error("var not exists")]
    VarError(#[from] VarError),

    #[error("parser error: {0}")]
    ParserError(#[from] ParserError),

    #[error("cstring error, nul error: {0}")]
    NulError(#[from] NulError),

    #[error("build in internal error: {0}")]
    BuildInError(#[from] BuildInError),

    #[error("fork error")]
    ForkError,

    #[error("env error: {0}")]
    EnvError(#[from] crate::env::EnvError),
}

pub fn run_expr(expr: &str) -> Result<(), CmdError> {
    let mut expr = parse(expr)?;

    if expr.command.is_empty() {
        manage_local_vars(&expr.env)?;

        return Ok(());
    }

    let path = resolve_dep(&expr.command).ok_or_else(|| {
        set_exit_status(POSIX_NOT_FOUND);

        CmdError::CommandNotFound(expr.command)
    })?;

    expr.env
        .push(CString::new(format!("PATH={}", std::env::var("PATH")?))?);
    expr.env
        .push(CString::new(format!("TERM={}", std::env::var("TERM")?))?);

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

            let pid = execute_cmd(&s, &args, &env)?;

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

            set_exit_status(exit);
        }
        Dep::BuildIn(build_in) => {
            let status = build_in.run(&expr.args, &expr.env)?;

            set_exit_status(status);
        }
    }

    Ok(())
}

fn execute_cmd(path: &str, args: &[*const c_char], env: &[*const c_char]) -> Result<pid_t, CmdError> {
    let c_path = CString::new(path)?;
    let mut argv = vec![c_path.as_ptr()];
    argv.extend_from_slice(args);
    argv.push(std::ptr::null());

    let mut envp = env.to_vec();
    envp.push(std::ptr::null());

    unsafe {
        let pid = fork();

        if pid < 0 {
            return Err(CmdError::ForkError);
        }

        if pid == 0 {
            execve(
                c_path.as_ptr(),
                argv.as_slice().as_ptr() as *const *const c_char,
                envp.as_slice().as_ptr() as *const *const c_char,
            );
            _exit(-1);
        }

        Ok(pid)
    }
}

pub struct Argv(pub *const *const c_char);
pub struct Envp(pub *const *const c_char);
