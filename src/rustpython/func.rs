use rustpython_vm::{PyObjectRef, VirtualMachine, convert::{ToPyObject}, function::FuncArgs};

use crate::{_python::{pystr_leak, var::{pyobject_to_var, var_to_pyobject}}, shared::{PixelScriptRuntime, func::call_function, var::Var}};

/// Attach a function to a Python context.
/// 
/// Params:
/// - vm: VirtualMachine. Obviously the vm.
/// - fn_name: &str. The name of the function.
/// - fn_idx: i32. The idx of the function.
pub(super) fn create_function(vm: &VirtualMachine, fn_name: &str, fn_idx: i32) -> PyObjectRef {
    // Leak the name to turn it into a static.
    let static_name: &'static str = unsafe { pystr_leak(fn_name.to_string()) };

    // Create the function.
    let func =  vm.new_function(static_name, move |args: FuncArgs, vm: &VirtualMachine| {
        // Convert args -> argv
        let mut argv: Vec<Var> = vec![];
        
        // Pass in the runtime type
        argv.push(Var::new_i64(PixelScriptRuntime::Python as i64));

        // Now Python vars
        for arg in args.args {
            argv.push(pyobject_to_var(vm, arg).expect("Could not convert value into Var from Python."));
        }

        unsafe {
            let res = call_function(fn_idx, argv);
            var_to_pyobject(vm, &res).to_pyobject(vm)
        }
    });

    func.into()
}

