use std::{collections::HashMap, sync::{Mutex, OnceLock}};

use rustpython::vm::{Interpreter, PyObjectRef, convert::ToPyObject, scope::Scope};

use crate::shared::PixelScript;

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
    /// HostObject class types
    class_types: HashMap<String, PyObjectRef>
}

impl State {
    pub fn add_class_type(&mut self, name: &str, class_type: PyObjectRef) {
        self.class_types.insert(name.to_string(), class_type);
    }
}

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
                class_types: HashMap::new() 
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
        // TODO: Stop python
        let _state = get_state();
    }

    fn add_variable(name: &str, variable: &crate::shared::var::Var) {
        let state = get_state();
        state.engine.enter(|vm| {
            let var = variable.clone().to_pyobject(vm);
            state.global_scope.locals.set_item(name, var.into(), vm).expect("Could not set");
        });
    }

    fn add_object_variable(name: &str, idx: i32) {
        todo!()
    }

    fn add_callback(name: &str, fn_idx: i32) {
        todo!()
    }

    fn add_module(source: std::sync::Arc<crate::shared::module::Module>) {
        todo!()
    }

    fn add_object(name: &str, callback: crate::shared::func::Func, opaque: *mut std::ffi::c_void) {
        todo!()
    }

    fn execute(code: &str, file_name: &str) -> String {
        todo!()
    }
}