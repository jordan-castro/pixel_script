use std::sync::Arc;

use mlua::prelude::*;
use crate::{lua::func::internal_add_callback, shared::class::Class};

pub fn create_class(lua: &Lua, source: Arc<Class>) -> LuaTable {
    let table = lua.create_table().expect("Could not create table.");

    for variable in source.vars.iter() {
        table.set(
            variable.name.to_owned(),
            variable.var.clone().into_lua(lua).expect("Could not convert variable to Lua.")
        ).expect("Could not set variable to module.");
    }
    // Add callbacks
    for callback in source.callbacks.iter() {
            // Get internals
            let func = callback.func.func;
            let opaque = callback.func.opaque;

            // Create lua function
            let lua_function = internal_add_callback(lua, func, opaque);
                table
                .set(callback.name.as_str(), lua_function)
                .expect("Could not set callback to module");
        } 

    table
}