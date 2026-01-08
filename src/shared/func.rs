use crate::shared::PtrMagic;

use super::var::Var;
use std::{collections::HashMap, ffi::c_void, sync::{Mutex, OnceLock}};

/// Function reference used in C.
/// 
/// argc: i32, The number of args.
/// argc: *const *mut Var, a C Array of args.
/// opaque: *mut c_void, opaque user data. 
/// 
/// Func handles it's own memory, so no need to free the *mut Var returned or the argvs.
/// 
/// But if you use any Vars within the function, you will have to free them before the function returns.
pub type Func = unsafe extern "C" fn(
    argc: usize, 
    argv: *mut *mut Var, 
    opaque: *mut c_void
) -> *mut Var;

/// Basic rust structure to track Funcs and opaques together.
pub struct Function {
    pub name: String,
    pub func: Func,
    pub opaque: *mut c_void
}

unsafe impl Send for Function {}
unsafe impl Sync for Function {}

/// Lookup state structure
pub struct FunctionLookup {
    /// Function hash shared between all runtimes.
    /// 
    /// Negative numbers are valid here.
    pub function_hash: HashMap<i32, Function>
}

impl FunctionLookup {
    pub fn get_function(&self, idx: i32) -> Option<&Function> {
        self.function_hash.get(&idx)
    }
    pub fn add_function(&mut self, name: &str, func: Func, opaque: *mut c_void) -> i32 {
        // TODO: Allow for negative idxs.
        self.function_hash.insert(self.function_hash.len() as i32, Function {name: name.to_string(), func, opaque });

        return (self.function_hash.len() - 1) as i32;
    }
}

/// The function lookup!
static FUNCTION_LOOKUP: OnceLock<Mutex<FunctionLookup>> = OnceLock::new();

/// Get the function lookup global state. Shared between all runtimes.
fn get_function_lookup() -> std::sync::MutexGuard<'static, FunctionLookup> {
    FUNCTION_LOOKUP.get_or_init(|| {
        Mutex::new(FunctionLookup {
            function_hash: HashMap::new(),
        })
    })
    .lock()
    .unwrap()
}

/// Add a function to the lookup
pub fn lookup_add_function(name: &str, func: Func, opaque: *mut c_void) -> i32 {
    let mut lookup = get_function_lookup();
    let idx = lookup.function_hash.len();
    lookup.function_hash.insert(idx as i32, Function { name: name.to_string(), func, opaque });
    idx as i32
}

/// Clear function lookup hash
pub fn clear_function_lookup() {
    let mut lookup = get_function_lookup();
    lookup.function_hash.clear();
}

/// Call a function that is saved in the lookup by a idx.
/// 
/// This should only be used within languages and never from a end user.
pub unsafe fn call_function(fn_idx: i32, args: Vec<Var>) -> Var {
    let (func, opaque) = {
        let fl = get_function_lookup();
        let function = fl.get_function(fn_idx);
        if function.is_none() {
            return Var::new_null();
        }

        let function = function.unwrap();

        (function.func, function.opaque)
    };

    let argc = args.len();
    let argv = Var::make_pointer_array(args);
    
    unsafe {
        let res = func(argc, argv, opaque);
        // Free ptr array
        Var::free_pointer_array(argv, argc);

        if res.is_null() {
            Var::new_null()
        } else {
            Var::from_raw(res)
        }
    }
}