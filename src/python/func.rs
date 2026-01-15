use crate::{
    borrow_string, create_raw_string, free_raw_string,
    python::{
        exec_py, get_fn_idx_from_name, pocketpy, var::pocketpyref_to_var, var_to_pocketpyref,
    },
    shared::{PixelScriptRuntime, func::call_function, read_file, var::Var},
};

/// The size of the Python bug in debug modes.
#[cfg(debug_assertions)]
const PY_SIZE_DEBUG: usize = 24;
/// The size of the Python bug in release modes.
#[cfg(not(debug_assertions))]
const PY_SIZE_RELEASE: usize = 24;

/// Use instead of the py_arg macro.
pub(super) unsafe fn py_get_arg(argv: pocketpy::py_StackRef, i: usize) -> pocketpy::py_StackRef {
    // 1. Convert the pointer to a raw byte address (u8)
    let base_addr = argv as *mut u8;

    #[cfg(debug_assertions)]
    let pysize = PY_SIZE_DEBUG;
    #[cfg(not(debug_assertions))]
    let pysize = PY_SIZE_RELEASE;

    // 2. Manually offset by (index * 16 bytes)
    // 16 is the standard size of py_TValue in pocketpy
    let offset_addr = unsafe { base_addr.add(i * pysize) };

    // 3. Cast it back to the pointer type the VM expects
    offset_addr as pocketpy::py_StackRef
}

/// Use instead of the py_assign macro.
pub(super) unsafe fn py_assign(left: pocketpy::py_Ref, right: pocketpy::py_Ref) {
    unsafe {
        *left = *right;
    }
}

/// Use instead of py_setattr
pub(super) unsafe fn py_setattr(_self: pocketpy::py_Ref, name: &str, val: pocketpy::py_Ref) {
    let name = create_raw_string!(name);
    unsafe {
        let pyname = pocketpy::py_name(name);
        pocketpy::py_setattr(_self, pyname, val);
        free_raw_string!(name);
    }
}

pub(super) unsafe fn raise(msg: &str) -> bool {
    let c_msg = create_raw_string!(msg);
    unsafe {
        let ret_slot = pocketpy::py_retval();
        pocketpy::py_newstr(ret_slot, c_msg);
        free_raw_string!(c_msg);

        return true;
    }
}

/// The pocketpy bridge
pub(super) unsafe extern "C" fn pocketpy_bridge(argc: i32, argv: pocketpy::py_StackRef) -> bool {
    // let pyref_size = pocketpy::get_py_TValue_size();
    if argc < 1 {
        unsafe {
            return raise("Python: argc < 1");
        }
        // var_to_pocketpyref(ret_slot, &Var::new_null());
    }
    let c_name = unsafe { pocketpy::py_tostr(py_get_arg(argv, 0)) };
    let name = borrow_string!(c_name);
    let fn_idx = get_fn_idx_from_name(name);
    if fn_idx.is_none() {
        return unsafe { raise("Python: fn_idx is empty.") };
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

pub(super) unsafe extern "C" fn virtual_module_loader(_argc: i32, argv: pocketpy::py_StackRef) -> bool {
    let modname_ref = unsafe { py_get_arg(argv, 0) };
    let modname = unsafe{ pocketpy::py_tostr(modname_ref) };
    // let modname = borrow_string!(modname);

    // Check if module already exists (Could potentially disable this)
    let existing = unsafe { pocketpy::py_getmodule(modname) };
    if !existing.is_null() {
        unsafe {
            py_assign(pocketpy::py_retval(), existing);
        }
    }

    // Convert modname from '.' into '/'
    let modname_str = borrow_string!(modname);
    let modname_str = modname_str.replace(".", "/");

    // Try and read the file
    let contents = read_file(format!("{modname_str}.py").as_str());
    if contents.is_empty() {
        // If empty, means not found. Default to builtin
        unsafe {
            let res= pocketpy::py_import(modname);
            if res  == 1{
                true
            } else {
                false
            }
        }
    } else {
        // We have a module, so let's compile it
        unsafe {
            let nmod = pocketpy::py_newmodule(modname);

            // In rustpython we use __name__, __package__, __loader__, __spec__. So let's just do the same here.

            let r0 = pocketpy::py_getreg(0);
            pocketpy::py_newstr(r0, modname);
            py_setattr(nmod, "__name__", r0);

            // Let's do file too
            let file = create_raw_string!(format!("v://{modname_str}.py",));
            let r1 = pocketpy::py_getreg(1);
            pocketpy::py_newstr(r1, file);
            free_raw_string!(file);
            py_setattr(nmod, "__name__", r1);

            let r2 = pocketpy::py_getreg(2);
            pocketpy::py_newnone(r2);
            py_setattr(nmod, "__package__", r2);

            let r3 = pocketpy::py_getreg(3);
            pocketpy::py_newnone(r3);
            py_setattr(nmod, "__loader__", r3);

            let r4 = pocketpy::py_getreg(4);
            pocketpy::py_newnone(r4);
            py_setattr(nmod, "__spec__", r4);

            // Execute code to "compile"
            exec_py(&contents, &modname_str, &modname_str);

            py_assign(pocketpy::py_retval(), nmod);

            true
        }
    }
}
