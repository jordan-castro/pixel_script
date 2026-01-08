use std::sync::Arc;

use rustpython::vm::{PyObjectRef, TryFromObject, VirtualMachine, convert::ToPyObject, function::FuncArgs, types::{PyTypeFlags, PyTypeSlots}};

use crate::{python::get_state, shared::{PixelScriptRuntime, func::call_function, object::PixelObject, var::Var}};

/// Create object callback methods
fn create_object_method(vm: &VirtualMachine, fn_name: &str, fn_idx: i32) -> PyObjectRef {
    let mut state = get_state();
    let static_name = unsafe {
     state.new_str_leak(fn_name.to_string())
    };

    vm.new_function(static_name, move |args: FuncArgs, vm: &VirtualMachine| {
        // First arg is a object
        let pyobj = args.args[0].clone();
        let obj_id = pyobj.get_attr("_id", vm).expect("Could not get _id from Python.");
        // try_to_value::<i64>
        let obj_id = obj_id.try_to_value::<i64>(vm).expect("Could not get Int from Python.");

        let mut argv = vec![];

        // Runtime
        argv.push(Var::new_i64(PixelScriptRuntime::Python as i64));
        // Object id
        argv.push(Var::new_i64(obj_id));

        for arg in args.args {
            argv.push(Var::try_from_object(vm, arg).expect("Could not convert value into Var from Python."));
        }

        unsafe {
            // Call actual function
            let res = call_function(fn_idx, argv);
            res.to_pyobject(vm)
        }
    }).into()
}

/// Create a object type in the Python Runtime.
/// 
/// obj_name: Obviously the name of the ojbect
/// idx: is the saved object.
/// source: is the object methods
pub(super) fn create_object(vm: &VirtualMachine, idx: i32, source: Arc<PixelObject>) -> PyObjectRef {
    // Look to see if class type already exists.
    let mut state = get_state();
    let class_type = state.class_types.get(&source.type_name);
    if let Some(class_type) = class_type {
        // One exists, create a new one and add _id
        let res = class_type.clone().call(FuncArgs::default(), vm).expect("Could not instantiate Python class");
        res.set_attr("_id", vm.ctx.new_int(idx), vm).expect("Could not set _id to Python object.");

        return res;
    }

    // Otherwise need to create a new one NOW
    let static_name = unsafe {
        state.new_str_leak(source.type_name.clone())
    };
    // Define basic slots
    let slots = PyTypeSlots::new(static_name, PyTypeFlags::HEAPTYPE | PyTypeFlags::BASETYPE | PyTypeFlags::HAS_DICT);

    // Create class type
    let class_type = vm.ctx.new_class(None, &source.type_name, vm.ctx.types.object_type.to_owned(), slots);

    // Add class methods
    for method in source.callbacks.iter() {
        let pyfunc = create_object_method(vm, &method.name, method.idx);
        // add
        let intern_name = vm.ctx.intern_str(method.name.clone());
        class_type.set_attr(intern_name, pyfunc.into());
    }

    let pyobj: PyObjectRef = class_type.clone().into();
    // Save globally
    state.class_types.insert(source.type_name.clone(), pyobj.clone());

    // Attach _id
    let res= pyobj.call(FuncArgs::default(), vm).expect("Could not instantiate Python class");
    res.set_attr("_id", vm.ctx.new_int(idx), vm).expect("Could not set ID to Python object.");

    res
}
