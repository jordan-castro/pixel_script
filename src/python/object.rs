// Copyright 2026 Jordan Castro <jordan@grupojvm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use std::{sync::Arc};

use crate::{python::{PythonScripting, add_new_defined_object, eval_main_py, exec_main_py, is_object_defined, make_private}, shared::{PixelScript, object::PixelObject}};

/// Create a object type in the Python Runtime.
/// 
/// idx: is the saved object.
/// source: is the object methods
pub(super) fn create_object(idx: i32, source: Arc<PixelObject>) {
    // Check if object is defined.
    let obj_exists = is_object_defined(&source.type_name);
    if obj_exists {
        eval_main_py(format!("_{}({})", source.type_name, idx).as_str(), format!("<create_{}>",&source.type_name).as_str());
        return;
    }
    
    // Object does not exist
    // First register callbacks
    let mut methods_str = String::new();
    for method in source.callbacks.iter() {
        let private_name = make_private(&method.name);
        PythonScripting::add_callback(&method.name, method.idx);
        methods_str.push_str(format!(r#"
    def {}(self, *args):
        return {}('{}', self.ptr, *args)
        
"#, method.name, private_name, method.name).as_str());
    }

    let object_string = format!(r#"
# Bridge for pocketpy
class _{}:
    def __init__(self, ptr):
        # Set the ptr
        self.ptr = ptr

{}

"#, source.type_name, methods_str);

// _{}({})
// , source.type_name, idx
    // println!("{object_string}");

    // Execute it
    let res = exec_main_py(&object_string, format!("<first_{}>", &source.type_name).as_str());
    if !res.is_empty() {
        return;
    }

    // add it
    add_new_defined_object(&source.type_name);

    // Ok but just create it now
    let res = eval_main_py(format!("_{}({})", source.type_name, idx).as_str(), "<create_{}>");
    if !res.is_empty() {
        println!("PYTHONSDNAOSDNOIDRes: {res}");
    }
}

// use rustpython_vm::{AsObject, Py, PyObjectRef, VirtualMachine, builtins::PyType, function::{FuncArgs, PyMethodFlags}, types::{PyTypeFlags, PyTypeSlots}};

// use crate::{_python::{get_class_type_from_cache, pystr_leak, store_class_type_in_cache, var::pyobject_to_var, var_to_pyobject}, shared::{PixelScriptRuntime, func::call_function, object::PixelObject, var::Var}};

// /// Create object callback methods
// fn create_object_method(vm: &VirtualMachine, fn_name: &str, fn_idx: i32, static_class: &'static Py<PyType>) -> PyObjectRef {
//     let static_name = unsafe { pystr_leak(fn_name.to_string()) };

//     vm.ctx.new_method_def(static_name, move |args: FuncArgs, vm: &VirtualMachine| {
//         // First arg is a object
//         let pyobj = args.args[0].clone();
//         let obj_id = pyobj.get_attr("_id", vm).expect("Could not get _id from Python.");
//         // try_to_value::<i64>
//         let obj_id = obj_id.try_to_value::<i64>(vm).expect("Could not get Int from Python.");

//         let mut argv = vec![];

//         // Runtime
//         argv.push(Var::new_i64(PixelScriptRuntime::Python as i64));
//         // Object id
//         argv.push(Var::new_i64(obj_id));

//         for arg in args.args.iter().skip(1) {
//             let var_arg = pyobject_to_var(vm, arg.to_owned()).expect("Could not convert value into Var from Python.");
//             argv.push(var_arg);
//         }

//         unsafe {
//             // Call actual function
//             let res = call_function(fn_idx, argv);
//             var_to_pyobject(vm, &res)
//         }
//     }, PyMethodFlags::METHOD, None).build_method(static_class, vm).into()
// }

// /// Create a object type in the Python Runtime.
// /// 
// /// obj_name: Obviously the name of the ojbect
// /// idx: is the saved object.
// /// source: is the object methods
// pub(super) fn create_object(vm: &VirtualMachine, idx: i32, source: Arc<PixelObject>) -> PyObjectRef {
//     // Look to see if class type already exists.
//     let class_type = get_class_type_from_cache(&source.type_name);
//     if let Some(class_type) = class_type {
//         let class_object: PyObjectRef = class_type.clone().into();
//         // One exists, create a new one and add _id
//         let res = class_object.clone().call(FuncArgs::default(), vm).expect("Could not instantiate Python class");
//         res.set_attr("_id", vm.ctx.new_int(idx), vm).expect("Could not set _id to Python object.");

//         return res;
//     }

//     // Otherwise need to create a new one NOW
//     let static_name = unsafe { pystr_leak(source.type_name.clone()) };
//     // Define basic slots
//     let slots = PyTypeSlots::new(static_name, PyTypeFlags::HEAPTYPE | PyTypeFlags::BASETYPE | PyTypeFlags::HAS_DICT);

//     // Create class type
//     let class_type = vm.ctx.new_class(None, &source.type_name, vm.ctx.types.object_type.to_owned(), slots);

//     // Add class methods
//     // Store class type
//     store_class_type_in_cache(&source.type_name, class_type);
//     // Get it again
//     let class_type = get_class_type_from_cache(&source.type_name).unwrap();
//     for method in source.callbacks.iter() {
//         let pyfunc = create_object_method(vm, &method.name, method.idx, class_type);
//         // add to __dict__
//         let intern_name = vm.ctx.new_str(method.name.clone());
//         class_type.as_object().set_attr(&intern_name, pyfunc, vm).expect("Could not attach method to Python class.");
//     }

//     let pyobj: PyObjectRef = class_type.clone().into();
//     // Attach _id
//     let res= pyobj.call(FuncArgs::default(), vm).expect("Could not instantiate Python class");
//     res.set_attr("_id", vm.ctx.new_int(idx), vm).expect("Could not set ID to Python object.");

//     res
// }
