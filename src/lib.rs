use shared::{func::Func, var::Var};
use std::{
    ffi::{CStr, CString, c_char, c_void},
    ptr,
    sync::{Arc, Mutex, MutexGuard, OnceLock},
};

use crate::{
    lua::LuaScripting,
    shared::{PixelScript, PtrMagic, class::PixelClass, module::Module},
};

pub mod shared;

#[cfg(feature = "lua")]
pub mod lua;

/// Macro to wrap features
macro_rules! with_feature {
    ($feature:expr, $logic:block) => {
        #[cfg(feature=$feature)]
        {
            $logic
        }
    };
}

/// Convert a borrowed C string (const char *) into a Rust &str.
macro_rules! borrow_string {
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
macro_rules! own_string {
    ($cstr:expr) => {{
        if $cstr.is_null() {
            String::new()
        } else {
            let owned_string = unsafe { CString::from_raw($cstr) };

            owned_string
                .into_string()
                .unwrap_or_else(|_| String::from("Invalid UTF-8"))
        }
    }};
}

/// Create a raw string from &str.
///
/// Remember to FREE THIS!
macro_rules! create_raw_string {
    ($rstr:expr) => {{ CString::new($rstr).unwrap().into_raw() }};
}

/// Current pixelscript version.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_version() -> u32 {
    0x00010000 // 1.0.0
}

/// Add a variable to the __main__ context.
/// Gotta pass in a name, and a Variable value.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_add_variable(name: *const c_char, variable: &Var) {
    // Get string as rust.
    let r_str = borrow_string!(name);
    if r_str.is_empty() {
        return;
    }

    // Add variable to lua context
    with_feature!("lua", {
        LuaScripting::add_variable(r_str, variable);
    });
}

/// Add a callback to the __main__ context.
/// Gotta pass in a name, Func, and a optionl *void opaque data type
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_add_callback(name: *const c_char, func: Func, opaque: *mut c_void) {
    // Get rust name
    let name_str = borrow_string!(name);
    if name_str.is_empty() {
        return;
    }

    // Add Function to lua context
    with_feature!("lua", {
        LuaScripting::add_callback(name_str, func, opaque);
    });
}

/// Execute some lua code. Will return a String, an empty string means that the
/// code executed succesffuly
///
/// The result needs to be freed by calling `pixelscript_free_str`
#[unsafe(no_mangle)]
#[cfg(feature = "lua")]
pub extern "C" fn pixelscript_exec_lua(
    code: *const c_char,
    file_name: *const c_char,
) -> *const c_char {
    // First convert code and file_name to rust strs
    let code_str = borrow_string!(code);
    if code_str.is_empty() {
        return create_raw_string!("Code is empty");
    }
    let file_name_str = borrow_string!(file_name);
    if file_name_str.is_empty() {
        return create_raw_string!("File name is empty");
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
    let name_str = borrow_string!(name);

    Module::new(name_str.to_owned()).into_raw()
}

/// Add a callback to a module.
///
/// Pass in the modules pointer and callback paramaters.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_module_add_callback(
    module_ptr: *mut Module,
    name: *const c_char,
    func: Func,
    opaque: *mut c_void,
) {
    if module_ptr.is_null() {
        return;
    }

    if name.is_null() {
        return;
    }

    // Get actual data
    let module = unsafe { Module::from_borrow(module_ptr) };
    let name_str = borrow_string!(name);

    // Now add callback
    module.add_callback(name_str, func, opaque);
}

/// Add a Varible to a module.
///
/// Pass in the module pointer and variable params.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_module_add_variable(
    module_ptr: *mut Module,
    name: *const c_char,
    variable: &Var,
) {
    if module_ptr.is_null() {
        return;
    }

    if name.is_null() {
        return;
    }

    let module = unsafe { Module::from_borrow(module_ptr) };
    let name_str = borrow_string!(name);

    // Now add variable
    module.add_variable(name_str, variable);
}

/// Add a Module to a Module
///
/// This transfers ownership.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_module_add_module(parent_ptr: *mut Module, child_ptr: *mut Module) {
    if parent_ptr.is_null() || child_ptr.is_null() {
        return;
    }

    let parent = unsafe { Module::from_borrow(parent_ptr) };
    // Own child
    let child = Module::from_raw(child_ptr);

    parent.add_module(child);

    // Child is now owned by parent
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
    with_feature!("lua", {
        LuaScripting::add_module(Arc::clone(&module));
    });

    // Module gets dropped here, and that is good!
}

/// Optionally free a module if you changed your mind.
#[unsafe(no_mangle)]
pub extern "C" fn pixelmods_free_module(module_ptr: *mut Module) {
    if module_ptr.is_null() {
        return;
    }

    let _ = Module::from_raw(module_ptr);
}

/// Create a new class.
///
/// Caller does not own memory.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_new_class(name: *const c_char) -> *mut PixelClass {
    if name.is_null() {
        return ptr::null_mut();
    } else {
        // Borrow the name
        let name_borrow = borrow_string!(name);
        PixelClass::new(name_borrow.to_string()).into_raw()
    }
}

/// Add a callback to a class.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_class_add_callback(
    class_ptr: *mut PixelClass,
    name: *const c_char,
    callback: Func,
    opaque: *mut c_void,
) {
    if class_ptr.is_null() || name.is_null() {
        return;
    }

    // borrow class
    let class_borrow = unsafe { PixelClass::from_borrow(class_ptr) };
    // Borrow name
    let name_borrow = borrow_string!(name);

    // Add callback to class
    class_borrow.add_callback(name_borrow, callback, opaque);
}

/// Add a variable to a class
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_class_add_variable(
    class_ptr: *mut PixelClass,
    name: *const c_char,
    variable: &Var,
) {
    if class_ptr.is_null() || name.is_null() {
        return;
    }

    // Borrow class
    let class_borrow = unsafe { PixelClass::from_borrow(class_ptr) };
    // Borrow name
    let name_borrow = borrow_string!(name);

    // Add var
    class_borrow.add_variable(name_borrow, variable);
}

/// Add a class to the runtime.
///
/// Classes are special because some VMs don't have a easy way of building classes. This might be pseudocoded.
///
/// This transfers ownership.
#[unsafe(no_mangle)]
pub extern "C" fn pixelscript_add_class(class_ptr: *mut PixelClass) {
    if class_ptr.is_null() {
        return;
    }

    let class_owned = Arc::new(PixelClass::from_raw(class_ptr));

    with_feature!("lua", {
        LuaScripting::add_class(Arc::clone(&class_owned));
    });

    // Class is dropped
}
