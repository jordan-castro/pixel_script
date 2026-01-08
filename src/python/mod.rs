use std::{collections::HashMap, sync::{Arc, Mutex, OnceLock}};

use rustpython::vm::{self, Interpreter, PyObjectRef, convert::ToPyObject, scope::Scope};

use crate::{python::{func::create_function, module::create_module}, shared::PixelScript};

mod var;
mod func;
mod module;
mod object;

/// This is the Python State
struct State {
    /// The actual Python Interpreter
    engine: Interpreter,
    /// The global variable scope (for running in __main__)
    global_scope: Scope,
    /// Cached class types
    class_types: HashMap<String, PyObjectRef>,
    /// Cached leaked names.
    cached_leaks: HashMap<String,  *mut str>
}

impl State {
    unsafe fn new_str_leak(&mut self, s: String) -> &'static str {
        if let Some(&ptr) = self.cached_leaks.get(&s) {
            return unsafe {&*ptr};
        }

        // Convert String -> Box<str> -> *mut str
        let b = s.clone().into_boxed_str();
        let ptr = Box::into_raw(b);

        // Store in cache
        self.cached_leaks.insert(s, ptr);

        // Return as static reference
        unsafe {&*ptr}
    }
}

impl Drop for State {
    fn drop(&mut self) {
        self.class_types.clear();

        for (_, ptr) in self.cached_leaks.drain() {
            if !ptr.is_null() {
                unsafe {
                    let _ = Box::from_raw(ptr);
                }
            }
        }
    }
}
unsafe impl Send for State {}
unsafe impl Sync for State {}

/// The State static variable for Lua.
static STATE: OnceLock<Mutex<State>> = OnceLock::new();

/// Get the state of LUA.
fn get_state() -> std::sync::MutexGuard<'static, State> {
    let interp = rustpython::InterpreterConfig::new().interpreter();
    let scope = interp.enter(|vm| {
        let globals = vm.ctx.new_dict();

        Scope::new(None, globals)
    });
    let mutex = STATE.get_or_init(|| 
        Mutex::new(
            State { 
                engine: interp, 
                global_scope: scope,
                class_types: HashMap::new(),
                cached_leaks: HashMap::new()
            }
        )
    );

    // This will block the C thread if another thread is currently using Lua
    mutex.lock().expect("Failed to lock Python State")
}

pub struct PythonScripting {}

impl PixelScript for PythonScripting {
    fn start() {
        // Initalize the state
        let _ununsed = get_state();
    }

    fn stop() {
        // Nothing is really needed to be done here? Except maybe do some GC?
    }

    fn add_variable(name: &str, variable: &crate::shared::var::Var) {
        let state = get_state();
        state.engine.enter(|vm| {
            let var = variable.clone().to_pyobject(vm);
            state.global_scope.locals.set_item(name, var.into(), vm).expect("Could not set");
        });
    }

    fn add_callback(name: &str, fn_idx: i32) {
        let state = get_state();
        state.engine.enter(|vm| {
            let pyfunc = create_function(vm, name, fn_idx);
            // Attach it
            state.global_scope.locals.set_item(name, pyfunc.into(), vm).expect("Could not set");
        });
    }

    fn add_module(source: std::sync::Arc<crate::shared::module::Module>) {
        let state = get_state();
        state.engine.enter(|vm| {
            create_module(vm, Arc::clone(&source));
        });
    }

    fn execute(code: &str, file_name: &str) -> String {
        let state = get_state();
        state.engine.enter(|vm| {
            match vm.compile(code, vm::compiler::Mode::Exec, file_name.to_string()) {
                Ok(r) => {
                    r.to_string()
                },
                Err(e) => {
                    e.to_string()
                },
            }
        })
    }
}