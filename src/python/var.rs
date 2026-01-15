use std::{ffi::c_void, sync::Arc};

use crate::{borrow_string, create_raw_string, free_raw_string, python::{func::py_assign, object::create_object, pocketpy}, shared::{object::get_object, var::{Var, VarType}}};

/// Convert a PocketPy ref into a Var
pub(super) fn pocketpyref_to_var(pref: pocketpy::py_Ref) -> Var {
    let tp = unsafe { pocketpy::py_typeof(pref) };
    match tp as i32 {
        pocketpy::py_PredefinedType_tp_int => {
            let val: i64 = unsafe { pocketpy::py_toint(pref) };
            Var::new_i64(val)
        },
        pocketpy::py_PredefinedType_tp_float => {
            let val = unsafe { pocketpy::py_tofloat(pref) };
            Var::new_f64(val)
        },
        pocketpy::py_PredefinedType_tp_bool => {
            let val = unsafe { pocketpy::py_tobool(pref) };
            Var::new_bool(val)
        },
        pocketpy::py_PredefinedType_tp_str => {
            let cstr_ptr = unsafe { pocketpy::py_tostr(pref) };
            let r_str = borrow_string!(cstr_ptr).to_string();

            Var::new_string(r_str)
        },
        pocketpy::py_PredefinedType_tp_NoneType => {
            Var::new_null()
        }
        _ => {
            // Just save the pointer
            Var::new_object(pref as *mut c_void)
        }
    }
    // if pocketpy::py_istype(pref, pocketpy::py)
}

/// Convert a Var into a PocketPy ref
pub(super) fn var_to_pocketpyref(out: pocketpy::py_Ref, var: &Var) {
    unsafe {
        match var.tag {
            VarType::Int32 | VarType::Int64 | VarType::UInt32 | VarType::UInt64 => {
                pocketpy::py_newint(out, var.get_bigint());
            },
            VarType::Float64 | VarType::Float32 => {
                pocketpy::py_newfloat(out, var.get_bigfloat());
            },
            crate::shared::var::VarType::Bool => {
                pocketpy::py_newbool(out, var.get_bool().unwrap());
            },
            crate::shared::var::VarType::String => {
                let s = var.get_string().unwrap();
                let c_str = create_raw_string!(s);
                pocketpy::py_newstr(out, c_str);
                // Free raw string
                free_raw_string!(c_str);
            },
            crate::shared::var::VarType::Null => {
                pocketpy::py_newnone(out);
            },
            crate::shared::var::VarType::Object => {
                if var.value.object_val.is_null() {
                    pocketpy::py_newnone(out);
                } else {
                    // This is a Python object that already exists, just that it's pointer was passed around.
                    let ptr = var.value.object_val as pocketpy::py_Ref;
                    py_assign(out, ptr);
                    // UNSAFE UNSAFE UNSAFE UNSAFE!!!!
                }
            },
            crate::shared::var::VarType::HostObject => {
                let idx = var.value.host_object_val;
                let pixel_object = get_object(idx).unwrap();
                // DO NOT FREE POCKETPY memory.
                pixel_object.update_free_lang_ptr(false);
                let lang_ptr_is_null = pixel_object.lang_ptr.lock().unwrap().is_null();
                if lang_ptr_is_null {
                    // TODO: Create the object for the first time...
                    create_object(idx, Arc::clone(&pixel_object));
                    // Get py_retval
                    let pyobj = pocketpy::py_retval();
                    // Set that as the pointer
                    pixel_object.update_lang_ptr(pyobj as *mut c_void);
                }
                // Get PTR again
                let lang_ptr = pixel_object.lang_ptr.lock().unwrap();
                // Assign again
                *out = *(*lang_ptr as pocketpy::py_Ref);
            },
        }
    }
}

// RUST PYTHON OLD VERSION
            // crate::shared::var::VarType::HostObject => {
            //     unsafe {
            //         let idx = var.value.host_object_val;
            //         let pixel_object = get_object(idx).unwrap();
            //         let lang_ptr_is_null = pixel_object.lang_ptr.lock().unwrap().is_null();
            //         if lang_ptr_is_null {
            //             // Create the object for the first and mutate the pixel object TODO.
            //             let pyobj = create_object(vm, idx, Arc::clone(&pixel_object));
            //             // Set pointer
            //             pixel_object.update_lang_ptr(pyobj.into_raw() as *mut c_void);
            //         }

            //         // Get PTR again
            //         let lang_ptr = pixel_object.lang_ptr.lock().unwrap();
            //         // Get as PyObject and grab dict
            //         let pyobj_ptr = *lang_ptr as *const PyObject;

            //         PyObjectRef::from_raw(pyobj_ptr)
            //     }
            // },
