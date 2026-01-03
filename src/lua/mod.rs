pub mod var;
pub mod func;
pub mod module;
pub mod class;

use std::sync::{Arc, Mutex, OnceLock};
use mlua::prelude::*;

use crate::shared::{PixelScript, class::PixelClass};

/// This is the Lua state. Each language gets it's own private state
struct State {
    /// The lua engine.
    engine: Lua,
}

/// The State static variable for Lua.
static STATE: OnceLock<Mutex<State>> = OnceLock::new();

/// Get the state of LUA.
fn get_state() -> std::sync::MutexGuard<'static, State> {
    let mutex = STATE.get_or_init(|| {
        Mutex::new(State {
            engine: Lua::new(),
        })
    });
    
    // This will block the C thread if another thread is currently using Lua
    mutex.lock().expect("Failed to lock Lua State")
}

/// Execute some orbituary lua code.
/// Returns a String. Empty means no error happened and was successful!
pub fn execute(code: &str, file_name: &str) -> String {
    let state = get_state();
    let res = state.engine.load(code).exec();
    if res.is_err() {
        let error_str = format!("Error in LUA: {}, for file: {}", res.unwrap_err().to_string(), file_name);
        return error_str;
    }

    String::from("")
}

pub struct LuaScripting {}

impl PixelScript for LuaScripting {
    fn add_variable(name: &str, variable: &crate::shared::var::Var) {
        var::add_variable(&get_state().engine, name, variable.clone());
    }

    fn add_callback(name: &str, callback: crate::shared::func::Func, opaque: *mut std::ffi::c_void) {
        func::add_callback(name, callback, opaque);
    }

    fn add_module(source: std::sync::Arc<crate::shared::module::Module>) {
        module::add_module(source);
    }

    fn add_class(source: std::sync::Arc<PixelClass>) {
        let state = get_state();
        let table = class::create_class(&state.engine, Arc::clone(&source));

        state.engine.globals().set(source.name.to_owned(), table).expect("Could not add Table to globals LUA");
    }

    fn execute(code: &str, file_name: &str) -> String {
        execute(code, file_name)
    }
}