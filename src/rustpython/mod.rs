// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::Arc,
};

use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use rustpython_vm::{
    Interpreter, PyObjectRef, PyRef, Settings, builtins::PyType,
    convert::ToPyObject, scope::Scope,
};

use crate::{
    _python::{func::create_function, module::create_module, overrides::{add_bundled_stdlib, override_import_loader}, var::var_to_pyobject},
    shared::{PixelScript},
};

mod func;
mod module;
mod object;
mod overrides;
mod var;

thread_local! {
    static PYSTATE: ReentrantMutex<State> = ReentrantMutex::new(init_state());
}

/// This is the Python State
struct State {
    /// The actual Python Interpreter
    engine: Interpreter,
    /// The global variable scope (for running in __main__)
    global_scope: PyObjectRef,
    /// Cached class types
    class_types: RefCell<HashMap<String, &'static PyRef<PyType>>>,
    /// Cached class ptrs
    class_ptrs: RefCell<Vec<*mut PyRef<PyType>>>,
    /// Cached leaked names.
    cached_leaks: RefCell<HashMap<String, *mut str>>,
}

/// Get the current threads python state
pub(self) fn get_state() -> ReentrantMutexGuard<'static, State> {
    PYSTATE.with(|mutex| {
        let guard = mutex.lock();
        // Transmute the lifetime so the guard can be passed around the thread
        unsafe { std::mem::transmute(guard) }
    })
}

/// Create a string for Python enviroment. String is cached and will be freed later automatically.
pub(self) unsafe fn pystr_leak(s: String) -> &'static str {
    let state = get_state();
    if let Some(&ptr) = state.cached_leaks.borrow().get(&s) {
        return unsafe { &*ptr };
    }

    // Convert to box
    let b = s.clone().into_boxed_str();
    let ptr = Box::into_raw(b);

    // Store in cache
    state.cached_leaks.borrow_mut().insert(s, ptr);

    // Return as static reference
    unsafe { &*ptr }
}

/// Get a class type from cache
pub(self) fn get_class_type_from_cache(type_name: &str) -> Option<&'static PyRef<PyType>> {
    let state = get_state();
    state.class_types.borrow().get(type_name).cloned()
}

/// Store a new class type in cache.
pub(self) fn store_class_type_in_cache(type_name: &str, class_type: PyRef<PyType>) {
    let state = get_state();

    // Leak class
    let class_static: &'static PyRef<PyType> = unsafe {
        let leaked_ptr = Box::into_raw(Box::new(class_type.clone()));
        state.class_ptrs.borrow_mut().push(leaked_ptr);
        // Cache ptr
        &*leaked_ptr // Dereference to get the static ref
    };
    state
        .class_types
        .borrow_mut()
        .insert(type_name.to_string(), class_static);
}

impl Drop for State {
    fn drop(&mut self) {
        for (_, ptr) in self.cached_leaks.borrow_mut().drain() {
            if !ptr.is_null() {
                unsafe {
                    let _ = Box::from_raw(ptr);
                }
            }
        }

        let ptrs_len = self.class_ptrs.borrow().len();
        for ptr in self.class_ptrs.borrow_mut().drain(0..ptrs_len) {
            if !ptr.is_null() {
                unsafe {
                    let _ = Box::from_raw(ptr);
                }
            }
        }
        self.class_ptrs.borrow_mut().clear();
    }
}
unsafe impl Send for State {}
unsafe impl Sync for State {}

// /// The State static variable for Lua.
// static STATE: OnceLock<ReentrantMutex<State>> = OnceLock::new();

/// Get the state of Python.
fn init_state() -> State {
    // Initialize state inside
    let mut settings = Settings::default();
    settings.path_list.push("".to_string());
    settings.write_bytecode = false;

    let interp = Interpreter::with_init(settings, |vm| {
        add_bundled_stdlib(vm);
        override_import_loader(vm);
    });
    // let interp = Interpreter::without_stdlib(settings);

    let scope = interp.enter(|vm| {
        let globals = vm.ctx.new_dict();
        // let sys_modules = vm.sys_module.get_attr("modules", vm).unwrap();

        // let modules_dict = sys_modules.downcast::<rustpython::vm::builtins::PyDict>().unwrap();

        // Remove dangerous modules from the cache so 'import os' fails
        // let _ = modules_dict.del_item("os", vm);
        // let _ = modules_dict.del_item("io", vm);
        // let _ = modules_dict.del_item("shutil", vm);

        globals.into()
    });

    State {
        engine: interp,
        global_scope: scope,
        class_types: RefCell::new(HashMap::new()),
        class_ptrs: RefCell::new(vec![]),
        cached_leaks: RefCell::new(HashMap::new()),
    }
}

pub struct PythonScripting {}

impl PixelScript for PythonScripting {
    fn start() {
        // Initalize the state
        let _state = get_state();
    }

    fn stop() {
        // let state = get_state();
        // // Run the GC
        // state.engine.enter(|vm| {
        //     if let Ok(gc_module) = vm.import("gc", 0) {
        //         if let Ok(collect_func) = gc_module.get_attr("collect", vm) {
        //             let _ = collect_func.call((), vm);
        //         }
        //     }
        // });
    }

    fn add_variable(name: &str, variable: &crate::shared::var::Var) {
        let state = get_state();
        state.engine.enter(|vm| {
            let var = var_to_pyobject(vm, variable);
            println!("Got var");
            state.global_scope.set_item(name, var, vm).expect("Could not set Var Python.");
        });
    }

    fn add_callback(name: &str, fn_idx: i32) {
        let state = get_state();
        state.engine.enter(|vm| {
            let pyfunc = create_function(vm, name, fn_idx);
            // Attach it
            vm.builtins
                .set_attr(unsafe { pystr_leak(name.to_string()) }, pyfunc, vm)
                .expect("Could not set callback Python.");
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
        let res = state.engine.enter(|vm| {
            let dict = state
                .global_scope
                .clone()
                .downcast::<rustpython_vm::builtins::PyDict>()
                .expect("Could not downcast to Dict, Python.");
            let scope = Scope::with_builtins(None, dict, vm);

            match vm.run_code_string(scope, code, file_name.to_string()) {
                Ok(_) => String::from(""),
                Err(e) => e
                    .to_pyobject(vm)
                    .str(vm)
                    .expect("Could not get error string Python.")
                    .as_str()
                    .to_string(),
            }
        });

        res
    }
}

// // TODO: this but for Python
// impl ObjectMethods for PythonScripting {
//     fn object_call(var: &Var, method: &str, args: Vec<Var>) -> Result<Var, anyhow::Error> {
//         // Get Python obj
//         let pyobj = unsafe {
//             if var.is_host_object() {

//             } else {
//                 // Just grab the ptr from itself
//                 let ptr = var.value.object_val as *const PyObjectRef;
//                 // Deferenciate
                
//             }
//         };
//     }
// }

// impl ObjectMethods for LuaScripting {
//     fn object_call(
//         var: &crate::shared::var::Var,
//         method: &str,
//         args: Vec<crate::shared::var::Var>,
//     ) -> Result<crate::shared::var::Var, anyhow::Error> {
//         // Get the lua table.
//         let table = unsafe {
//             if var.is_host_object() {
//                 // This is from the PTR!
//                 let pixel_object = get_object(var.value.host_object_val).expect("No HostObject found.");
//                 let lang_ptr = pixel_object.lang_ptr.lock().unwrap();
//                 // Get as table.
//                 let table_ptr = *lang_ptr as *const LuaTable;
//                 // Return table
//                 (&*table_ptr).clone()
//             } else {
//                 // Just grab it from the ptr itself
//                 let table_ptr = var.value.object_val as *const LuaTable;
//                 (&*table_ptr).clone()
//             }
//         };

//         // Call method
//         let mut lua_args = vec![];
//         {
//             // State start
//             let state = get_lua_state();
//             for arg in args {
//                 lua_args.push(arg.into_lua(&state.engine).expect("Could not convert Var into Lua Var"));
//             }
//             // State drop
//         }
//         // The function could potentially call the state
//         let res = table.call_function(method, lua_args).expect("Could not call function on Lua Table.");

//         // State start again
//         let state = get_lua_state();
//         let pixel_res = Var::from_lua(res, &state.engine).expect("Could not convert LuaVar into PixelScript Var.");

//         Ok(pixel_res)
//         // Drop state
//     }
// }
