use std::{collections::HashMap, ffi::CString, path::Path, sync::Mutex};

use once_cell::sync::Lazy;

use crate::build_in::{BuildIn, Exit};

pub static LOCAL_VARS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
    let mut map = HashMap::new();

    map.insert("?".to_string(), "0".to_string());

    Mutex::new(map)
});

pub fn manage_local_vars(vars: &[CString]) {
    let mut local_vars = LOCAL_VARS.lock().unwrap();
    for var_expr in vars {
        let var_expr = var_expr.as_c_str().to_str().unwrap();
        let (var_name, var_val) = {
            let mut s = var_expr.splitn(2, "=");

            (s.next().unwrap(), s.next().unwrap())
        };

        local_vars.insert(var_name.to_string(), var_val.to_string());
    }
}

pub fn var(var: &str) -> Option<String> {
    get_local_var(var).or_else(|| std::env::var(var).ok())
}

pub fn get_local_var(var: &str) -> Option<String> {
    let local_vars = LOCAL_VARS.lock().unwrap();

    local_vars.get(var).cloned()
}

pub fn set_local_var(var: &str, val: String) {
    let mut local_vars = LOCAL_VARS.lock().unwrap();

    *local_vars.get_mut(var).unwrap() = val;
}

pub fn resolve_dep(cmd: &str) -> Option<Dep> {
    match cmd {
        "exit" => {
            return Some(Dep::BuildIn(Box::new(Exit)));
        }
        _ => {}
    }

    let path = match std::env::var("PATH") {
        Ok(v) => v,
        Err(_) => return None,
    };

    for key in path.split(":") {
        let v = format!("{key}/{cmd}");
        let p = Path::new(&v);

        if p.exists() {
            return Some(Dep::Path(v));
        }
    }

    None
}

pub enum Dep {
    Path(String),
    BuildIn(Box<dyn BuildIn>),
}
