use std::{cell::RefCell, collections::HashMap};

use parking_lot::{ReentrantMutex, ReentrantMutexGuard};

use crate::{
    create_raw_string, free_raw_string, own_string,
    python::{func::pocketpy_bridge, var::var_to_pocketpyref},
    shared::PixelScript,
};

// Allow for the binidngs only
#[allow(unused)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
pub(self) mod pocketpy {
    include!(concat!(env!("OUT_DIR"), "/pocketpy_bindings.rs"));
}

mod func;
mod var;

thread_local! {
    static PYSTATE: ReentrantMutex<State> = ReentrantMutex::new(init_state());
}

/// This is the Pocketpy state. Each language gets it's own private state
struct State {
    /// Name to IDX lookup for pocketpy bridge
    name_to_idx: RefCell<HashMap<String, i32>>,
}

fn exec_main_py(code: &str, name: &str) -> String {
    let c_code = create_raw_string!(code);
    let c_name = create_raw_string!(name);
    unsafe {
        let res = pocketpy::py_exec(c_code, c_name, pocketpy::py_CompileMode_EXEC_MODE, std::ptr::null_mut());
        free_raw_string!(c_code);
        free_raw_string!(c_name);
        if !res {
            let py_res = pocketpy::py_formatexc();
            let py_res = own_string!(py_res);

            py_res
        } else {
            String::new()
        }
    }
}

/// Initialize Lua state per thread.
fn init_state() -> State {
    State {
        name_to_idx: RefCell::new(HashMap::new()),
    }
}

/// Get the state of Pocketpy.
pub(self) fn get_py_state() -> ReentrantMutexGuard<'static, State> {
    PYSTATE.with(|mutex| {
        let guard = mutex.lock();
        // Transmute the lifetime so the guard can be passed around the thread
        unsafe { std::mem::transmute(guard) }
    })
}

/// Add a new name => idx
pub(self) fn add_new_name_idx_fn(name: String, idx: i32) {
    let state = get_py_state();
    state.name_to_idx.borrow_mut().insert(name, idx);
}

/// Get a IDX from a name
pub(self) fn get_fn_idx_from_name(name: &str) -> Option<i32> {
    let state = get_py_state();
    let fn_idx = state.name_to_idx.borrow().get(name).cloned();
    fn_idx
}

pub struct PythonScripting;

impl PixelScript for PythonScripting {
    fn start() {
        // py initialize here
        unsafe {
            pocketpy::py_initialize();
        }
        let s = exec_main_py("1 + 1", "<init>");
        let _state = get_py_state();
    }

    fn stop() {
        unsafe {
            pocketpy::py_finalize();
        }
    }

    fn add_variable(name: &str, variable: &crate::shared::var::Var) {
        unsafe {
            let r0 = pocketpy::py_getreg(0);
            if r0.is_null() {
                return;
            }
            let cstr = create_raw_string!(name);
            let pyname = pocketpy::py_name(cstr);
            var_to_pocketpyref(r0, variable);
            pocketpy::py_setglobal(pyname, r0);
            // free cstr
            free_raw_string!(cstr);
        }
    }

    fn add_callback(name: &str, idx: i32) {
        // Save function
        add_new_name_idx_fn(name.to_string(), idx);

        // Create a "private" name
        let private_name = format!("_pxs_{name}");

        let c_name = create_raw_string!(private_name.clone());
        let c_main = create_raw_string!("__main__");
        let bridge_code = format!(
            r#"
def {name}(*args):
    print(*args)
    return {private_name}('{name}', *args)
"#
        );
        let c_brige_name = format!("<callback_bridge for {private_name}>");
        unsafe {
            let global_scope = pocketpy::py_getmodule(c_main);

            pocketpy::py_bindfunc(global_scope, c_name, Some(pocketpy_bridge));

            // Execute bridge
            let s = exec_main_py(&bridge_code, &c_brige_name);
            println!("{s}");
            free_raw_string!(c_name);
            free_raw_string!(c_main);
        }
    }

    fn add_module(source: std::sync::Arc<crate::shared::module::Module>) {
        todo!()
    }

    fn execute(code: &str, file_name: &str) -> String {
        let res = exec_main_py(code, file_name);
        res
    }
    
    fn start_thread() {
        unsafe {
            let idx = pocketpy::py_currentvm();
            pocketpy::py_switchvm(idx + 1);
        }
    }
    
    fn stop_thread() {
        unsafe {
            let idx = pocketpy::py_currentvm();
            pocketpy::py_resetvm();
            pocketpy::py_switchvm(idx - 1);
        }
    }
}
