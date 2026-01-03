// use mlua::{IntoLua, Lua};
use mlua::prelude::*;

// Pure Rust goes here
use crate::{
    shared::var::{Var, VarType},
};

impl IntoLua for Var {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        // Convert the Rust/C type into Lua. Once it's in LUA we can free our memory, lua copies it and handles it from here on out.
        match self.tag {
            VarType::Int32 => Ok(mlua::Value::Integer(self.get_i32().unwrap() as i64)),
            VarType::Int64 => Ok(mlua::Value::Integer(self.get_i64().unwrap())),
            VarType::UInt32 => Ok(mlua::Value::Integer(self.get_u32().unwrap() as i64)),
            VarType::UInt64 => Ok(mlua::Value::Integer(self.get_u64().unwrap() as i64)),
            VarType::String => {
                let contents = self.get_string().unwrap();
                let lua_str = lua.create_string(contents).expect("test");

                Ok(mlua::Value::String(lua_str))
            }
            VarType::Bool => Ok(mlua::Value::Boolean(self.get_bool().unwrap())),
            VarType::Float32 => Ok(mlua::Value::Number(self.get_f32().unwrap() as f64)),
            VarType::Float64 => Ok(mlua::Value::Number(self.get_f64().unwrap())),
            VarType::Null => Ok(mlua::Value::Nil),
            VarType::Object => {
                unsafe {
                    // This MUST BE A TABLE!
                    let table_ptr = self.value.object_val as *const LuaTable;
                    if table_ptr.is_null() {
                        return Err(mlua::Error::RuntimeError("Null pointer in Object".to_string()));
                    }

                    // Clone 
                    let lua_table = (&*table_ptr).clone();

                    // WooHoo we are back into lua
                    Ok(mlua::Value::Table(lua_table))
                }
            }
        }
    }
}

/// Add a variable by name to __main__ in lua.
pub fn add_variable(context: &Lua, name: &str, variable: Var) {
    context
        .globals()
        .set(
            name,
            variable
                .into_lua(context)
                .expect("Could not unwrap LUA vl from Var."),
        )
        .expect("Could not add variable to Lua global context.");
    // Listo!
}
