use libc::exit;
use std::ffi::CString;

use crate::env::get_local_var;

pub trait BuildIn {
    // Result!
    fn run(&self, args: &[CString], env: &[CString]) -> i32;
}

pub struct Exit;

impl BuildIn for Exit {
    fn run(&self, _args: &[CString], _env: &[CString]) -> i32 {
        unsafe {
            exit(get_local_var("?").unwrap().parse::<i32>().unwrap());
        }
    }
}
