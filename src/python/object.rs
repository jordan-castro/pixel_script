use std::sync::Arc;

use rustpython::vm::{PyObject, PyObjectRef, PyPayload, VirtualMachine, builtins::PyType, types::{PyTypeFlags, PyTypeSlots}};

use crate::{python::get_state, shared::object::PixelObject};

/// Create a object type in the Python Runtime.
/// 
/// obj_name: Obviously the name of the ojbect
/// idx: is the saved object.
/// source: is the object methods
pub(super) fn create_object(vm: VirtualMachine, module_name: Option<&str>, obj_name: &str, idx: i32, source: Arc<PixelObject>) -> PyObjectRef {
    // Get class types
    let mut state = get_state();
    let class_types = &state.class_types;

    // Check if this is object inside a module.
    let full_name = {
        if let Some(module_name) = module_name {
            format!("{module_name}:{obj_name}").as_str().to_owned()
        } else {
            obj_name.to_string()
        }
    };
    let full_name = full_name.as_str();

    // Check if obj_name is already a class type
    if class_types.contains_key(full_name) {
        return class_types.get(full_name).unwrap().clone();
    }

    // Create a static name for PyTypeSlots.
    let static_name: &'static str = Box::leak(obj_name.to_owned().into_boxed_str());
    // Define basic slots
    let slots = PyTypeSlots::new(static_name, PyTypeFlags::HEAPTYPE | PyTypeFlags::BASETYPE | PyTypeFlags::HAS_DICT);

    // Class type does not yet exist, so create it.
    let class_type = vm.ctx.new_class(module_name, obj_name, vm.ctx.types.object_type.to_owned(), slots);
    // let object = vm.ctx.new_class(module, name, base, slots)
    
    // Add methods to class
    for method in source.callbacks.iter() {
        // TODO: create_function() method
        // let cbk = vm.new_fun
    }

    // Save class_type
    state.add_class_type(full_name, class_type.into());

    vm.ctx.none()
}

// use std::sync::Arc;

// use crate::{lua::func::internal_add_callback, shared::{func::get_function_lookup, object::PixelObject}};
// use mlua::prelude::*;

// pub(super) fn create_object(lua: &Lua, idx: i32, source: Arc<PixelObject>) -> LuaTable {
//     let table = lua.create_table().expect("Could not create table.");

//     // For methods within the creation of objects, the language needs to own the function since they are created at runtime
//     let mut function_lookup = get_function_lookup();

//     for callback in source.callbacks.iter() {
//         // Get internals
//         let func = callback.func.func;
//         let opaque = callback.func.opaque;

//         let fn_idx = function_lookup.add_function(func, opaque);

//         let lua_function = internal_add_callback(lua, fn_idx, Some(idx));
//         table.set(callback.name.as_str(), lua_function).expect("Could not set callback to object");
//     }

//     table
// }
