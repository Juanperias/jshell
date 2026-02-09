use libc::{EXIT_FAILURE, EXIT_SUCCESS, chdir, exit};
use thiserror::Error;
use std::{ffi::{CString, NulError}, num::ParseIntError, str::Utf8Error};

use crate::env::{get_local_var, get_status_var};

pub trait BuildIn {
    fn run(&self, args: &[CString], env: &[CString]) -> Result<i32, BuildInError>;
}

pub struct Exit;

impl BuildIn for Exit {
    fn run(&self, args: &[CString], _env: &[CString]) -> Result<i32, BuildInError> {
        let code = if args.len() == 0 {
            get_status_var()
        } else {
            args.get(0)
                .unwrap()
                .to_str()?
                .to_string()
                .parse::<i32>()?
        };

        unsafe {
            exit(code);
        }
    }
}

pub struct Cd;

impl BuildIn for Cd {
    fn run(&self, args: &[CString], _env: &[CString]) -> Result<i32, BuildInError> {
        let path = {
            if args.len() == 0 {
                let home = match std::env::var("HOME") {
                    Ok(v) => v,
                    Err(_) => return Ok(EXIT_FAILURE),
                };

                CString::new(home)?
            } else {
                let path = args.get(0).unwrap();

                path.to_owned()
            }
        };

        let ch = unsafe { chdir(path.as_ptr()) };

        if ch < 0 {
            println!(
                "cd: no such file or directory: {}",
                path.as_c_str().to_str()?
            );
            return Ok(EXIT_FAILURE);
        }

        Ok(EXIT_SUCCESS)
    }
}

#[derive(Debug, Error)]
pub enum BuildInError {
    #[error("parse int error: {0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("utf8 error: {0}")]
    Utf8Error(#[from] Utf8Error),

    #[error("cstring error, nul error: {0}")]
    NulError(#[from] NulError),
}
