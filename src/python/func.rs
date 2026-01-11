use crate::{borrow_string, python::{get_fn_idx_from_name, pocketpy, var::pocketpyref_to_var, var_to_pocketpyref}, shared::{PixelScriptRuntime, func::call_function, var::Var}};

/// The size of the Python bug in debug modes.
#[cfg(debug_assertions)]
const PY_SIZE_DEBUG: usize = 24;
/// The size of the Python bug in release modes.
#[cfg(not(debug_assertions))]
const PY_SIZE_RELEASE: usize = 24;

/// Use instead of the py_arg macro.
pub unsafe fn py_get_arg(argv: pocketpy::py_StackRef, i: usize) -> pocketpy::py_StackRef {
    // 1. Convert the pointer to a raw byte address (u8)
    let base_addr = argv as *mut u8;

    #[cfg(debug_assertions)]
    let pysize = PY_SIZE_DEBUG;
    #[cfg(not(debug_assertions))]
    let pysize = PY_SIZE_RELEASE;

    // 2. Manually offset by (index * 16 bytes)
    // 16 is the standard size of py_TValue in pocketpy
    let offset_addr = unsafe {base_addr.add(i * pysize) };
    
    // 3. Cast it back to the pointer type the VM expects
    offset_addr as pocketpy::py_StackRef
}

/// The pocketpy bridge 
pub(super) unsafe extern "C" fn pocketpy_bridge(argc: i32, argv: pocketpy::py_StackRef) -> bool {
    // let pyref_size = pocketpy::get_py_TValue_size();
    if argc < 1 {
        let ret_slot = unsafe { pocketpy::py_retval() };
        var_to_pocketpyref(ret_slot, &Var::new_null());
        return false;
    }
    let c_name = unsafe{pocketpy::py_tostr(py_get_arg(argv, 0))};
    let name = borrow_string!(c_name);
    let fn_idx = get_fn_idx_from_name(name);
    if fn_idx.is_none() {
        let ret_slot = unsafe { pocketpy::py_retval() };
        var_to_pocketpyref(ret_slot, &Var::new_null());
        return false;
    }
    let fn_idx = fn_idx.unwrap();

    // Convert argv into Vec<Var>
    let mut vars: Vec<Var> = vec![];

    // Add the runtime
    vars.push(Var::new_i64(PixelScriptRuntime::Python as i64));

    for i in 1..argc {
        let arg_ref = unsafe { py_get_arg(argv, i as usize) };
        vars.push(pocketpyref_to_var(arg_ref));
    }

    // Call internal function
    unsafe {
        let res = call_function(fn_idx, vars);
        let ret_slot = pocketpy::py_retval();
        
        var_to_pocketpyref(ret_slot, &res);
    }
    true
}