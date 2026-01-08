use mlua::prelude::*;
// use mlua::{Integer, IntoLua, Lua, MultiValue, Value::Nil, Variadic};

use crate::{lua::get_state, shared::{PixelScriptRuntime, func::call_function, var::Var}};

/// For internal use since modules also need to use the same logic for adding a Lua callback.
pub(super) fn internal_add_callback(lua: &Lua, fn_idx: i32) -> LuaFunction {
    lua.create_function(move |lua, args: LuaMultiValue| {
        // Convert args -> argv for pixelmods
        let mut argv: Vec<Var> = vec![];

        // Pass in the runtime type
        argv.push(Var::new_i64(PixelScriptRuntime::Lua as i64));

        // Objects are handled a little differently know. It's kinda repeated code but oh well.
        // // If a obj is passed
        // if let Some(obj) = obj {
        //     // Add the pointer.
        //     argv.push(Var::new_i64(obj as i64));
        // }

        for arg in args {
            argv.push(Var::from_lua(arg, lua).expect("Could not convert value into Var from Lua."));
        }        

        unsafe {
            let res = call_function(fn_idx, argv);

            let lua_val = res.into_lua(lua);
            lua_val
            // Memory will drop here, and Var will be automatically freed!
        }
    }).expect("Could not create lua function")
}

/// Add a callback to lua __main__ context.
pub(super) fn add_callback(name: &str, fn_idx: i32) {
    let state = get_state();
    let lua_func = internal_add_callback(&state.engine, fn_idx);
    state.engine.globals().set(name, lua_func).expect("Could not add callback to Lua.");
}