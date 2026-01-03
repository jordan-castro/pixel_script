use std::{ffi::{CStr, CString, c_char, c_void}, ptr, sync::{Arc, Mutex, MutexGuard, OnceLock}};
use shared::{
    var::Var,
    func::Func
};

use crate::{lua::LuaScripting, shared::{PixelScript, PtrMagic, module::Module}};

pub mod shared;
pub mod lua;

/// Convert a borrowed C string (const char *) into a Rust &str.
macro_rules! convert_borrowed_string {
    ($cstr:expr) => {{
        if $cstr.is_null() {
            "" 
        } else {
            unsafe {
                let c_str = CStr::from_ptr($cstr);
                c_str.to_str().unwrap_or("")
            }
        }
    }};
}

/// Convert a owned C string (i.e. owned by us now.) into a Rust String.
/// 
/// The C memory will be freed automatically, and you get a nice clean String!
macro_rules! convert_owned_string {
    ($cstr:expr) => {{
        if $cstr.is_null() {
            String::new()
        } else {
            let owned_string = unsafe { CString::from_raw($cstr) };

            owned_string.into_string().unwrap_or_else(|_| String::from("Invalid UTF-8"))  
        }
    }};
}

/// Create a raw string from &str.
/// 
/// Remember to FREE THIS!
macro_rules! create_raw_string {
    ($rstr:expr) => {{
        CString::new($rstr).unwrap().into_raw()   
    }};
}

// // Static for LuaScripting
// /// The State static variable for Lua.
// static LUA_STATE: OnceLock<Mutex<LuaScripting>> = OnceLock::new();
// /// Get the state of LUA.
// fn lua_get_state() -> MutexGuard<'static, LuaScripting> {
//     let mutex = LUA_STATE.get_or_init(|| {
//         Mutex::new(LuaScripting {})
//     });
    
//     // This will block the C thread if another thread is currently using Lua
//     mutex.lock().expect("Failed to lock Lua State")
// }

/// Current pixelscript version.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_version() -> u32 {
    0x00010000 // 1.0.0
}

/// Add a variable to the __main__ context.
/// Gotta pass in a name, and a Variable value.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_add_variable(name: *const c_char, variable: Var) {
    // Get string as rust.
    let r_str = convert_borrowed_string!(name);
    if r_str.is_empty() {
        return;
    }

    // Add variable to lua context
    LuaScripting::add_variable(r_str, variable);
}

/// Add a callback to the __main__ context.
/// Gotta pass in a name, Func, and a optionl *void opaque data type
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_add_callback(name: *const c_char, func: Func, opaque: *mut c_void) {
    // Get rust name
    let name_str = convert_borrowed_string!(name);
    if name_str.is_empty() {
        return;
    }

    // Add Function to lua context
    LuaScripting::add_callback(name_str, func, opaque);
}

/// Execute some lua code. Will return a String, an empty string means that the 
/// code executed succesffuly
/// 
/// The result needs to be freed by calling `pixelscript_free_str` 
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_exec_lua(code: *const c_char, file_name: *const c_char) -> *const c_char {
    // First convert code and file_name to rust strs
    let code_str = convert_borrowed_string!(code);
    if code_str.is_empty() {
        return create_raw_string!("Code is empty")
    }
    let file_name_str = convert_borrowed_string!(file_name);
    if file_name_str.is_empty() {
        return create_raw_string!("File name is empty")
    }

    // Execute and get result
    let result = lua::execute(code_str, file_name_str);

    create_raw_string!(result)
}

/// Free the string created by the pixelscript library
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_free_str(string: *mut c_char) {
    if !string.is_null() {
        unsafe {
            // Let the string go out of scope to be dropped
            let _ = CString::from_raw(string);
        }
    }
}

/// Create a new pixelscript Module.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_new_module(name: *const c_char) -> *mut Module {
    if name.is_null() {
        return ptr::null_mut();
    }
    let name_str = convert_borrowed_string!(name);

    Module::new(name_str.to_owned()).into_raw()
}

/// Add a callback to a module.
/// 
/// Pass in the modules pointer and callback paramaters.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_module_add_callback(module_ptr: *mut Module, name: *const c_char, func: Func, opaque: *mut c_void) {
    if module_ptr.is_null() {
        return;
    }

    if name.is_null() {
        return;
    }

    // Get actual data
    let module = unsafe {Module::from_borrow(module_ptr)};
    let name_str = convert_borrowed_string!(name);

    // Now add callback
    module.add_callback(name_str, func, opaque);
}

/// Add a Varible to a module.
/// 
/// Pass in the module pointer and variable params.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_module_add_variable(module_ptr: *mut Module, name: *const c_char, variable: Var) {
    if module_ptr.is_null() {
        return;
    }

    if name.is_null() {
        return;
    }

    let module = unsafe {Module::from_borrow(module_ptr)};
    let name_str = convert_borrowed_string!(name);

    // Now add variable
    module.add_variable(name_str, variable);
}

/// Add the module finally to the runtime.
/// 
/// After this you can forget about the ptr since PM handles it.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_add_module(module_ptr: *mut Module) {
    if module_ptr.is_null() {
        return;
    }

    let module = Arc::new(Module::from_raw(module_ptr));

    // LUA
    LuaScripting::add_module(Arc::clone(&module));
    // lua::module::add_module(Arc::clone(&module));

    // Module gets dropped here, and that is good!
}