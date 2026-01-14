use libc::{chdir, exit, EXIT_FAILURE, EXIT_SUCCESS};
use std::ffi::CString;

use crate::env::get_local_var;

pub trait BuildIn {
    // Result!
    fn run(&self, args: &[CString], env: &[CString]) -> i32;
}

pub struct Exit;

impl BuildIn for Exit {
    fn run(&self, args: &[CString], _env: &[CString]) -> i32 {
        let code = if args.len() == 0 {
            get_local_var("?").unwrap().parse::<i32>().unwrap()
        } else {
            args.get(0)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                .parse::<i32>()
                .unwrap()
        };

        unsafe {
            exit(code);
        }
    }
}

pub struct Cd;

impl BuildIn for Cd {
    fn run(&self, args: &[CString], env: &[CString]) -> i32 {
        let path = {
            if args.len() == 0 {
                let home = match std::env::var("HOME") {
                    Ok(v) => v,
                    Err(_) => {
                        return EXIT_FAILURE
                    },
                };

                CString::new(home).unwrap()
            } else {
                let path = args.get(0).unwrap();

                path.to_owned()
            }
        };

        let ch = unsafe {
            chdir(path.as_ptr())
        };

        if ch < 0 {
            println!("cd: no such file or directory: {}", path.as_c_str().to_str().unwrap());
            return EXIT_FAILURE;
        }

        EXIT_SUCCESS
    }
}
