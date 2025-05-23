use super::Direction;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Drag {
    pub direction: Direction,
    pub distance: u32,
}

impl<'lua> rollback_mlua::FromLua<'lua> for Drag {
    fn from_lua(
        lua_value: rollback_mlua::Value<'lua>,
        _lua: &'lua rollback_mlua::Lua,
    ) -> rollback_mlua::Result<Self> {
        let table = match lua_value {
            rollback_mlua::Value::Table(table) => table,
            _ => {
                return Err(rollback_mlua::Error::FromLuaConversionError {
                    from: lua_value.type_name(),
                    to: "Drag",
                    message: None,
                })
            }
        };

        Ok(Drag {
            direction: table.get("direction").unwrap_or_default(),
            distance: table.get("distance").unwrap_or_default(),
        })
    }
}

impl<'lua> rollback_mlua::IntoLua<'lua> for Drag {
    fn into_lua(
        self,
        lua: &'lua rollback_mlua::Lua,
    ) -> rollback_mlua::Result<rollback_mlua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("direction", self.direction)?;
        table.set("distance", self.distance)?;

        Ok(rollback_mlua::Value::Table(table))
    }
}
