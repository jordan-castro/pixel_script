use std::sync::Arc;

use crate::{
    lua::{func::internal_add_callback, get_state},
    shared::module::Module,
};
use mlua::prelude::*;

/// Add a module to Lua!
pub fn add_module(module: Arc<Module>) {
    // First get lua state
    let state = get_state();

    let mod_name = module.name.clone();
    let module_for_lua = Arc::clone(&module);

    // Let's create a table
    let package: LuaTable = state
        .engine
        .globals()
        .get("package")
        .expect("Could not grab the Package table");
    let preload: LuaTable = package
        .get("preload")
        .expect("Could not grab the Preload table");

    // create the loader function for require()
    let loader = state
        .engine
        .create_function(move |lua, _: ()| {
            let module_table = lua.create_table()?;

            // Add variables
            for variable in module_for_lua.variables.iter() {
                module_table
                    .set(
                        variable.name.to_owned(),
                        variable
                            .var
                            .clone()
                            .into_lua(lua)
                            .expect("Could not convert variable to Lua."),
                    )
                    .expect("Could not set variable to module.");
            }

            // Add callbacks
            for callback in module_for_lua.callbacks.iter() {
                // Get internals
                let func = callback.func.func;
                let opaque = callback.func.opaque;

                // Create lua function
                let lua_function = internal_add_callback(lua, func, opaque);
                module_table
                    .set(callback.name.as_str(), lua_function)
                    .expect("Could not set callback to module");
            } 

            // Return module
            Ok(module_table)
        })
        .expect("Could not load LUA module.");

    // Pre-load it
    preload
        .set(mod_name, loader)
        .expect("Could not set Lua module loader.");
}