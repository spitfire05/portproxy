use color_eyre::Result;
use rlua::{Function, Lua, Table, Value};

macro_rules! impl_log {
    ($lua:expr, $level:ident) => {
        $lua.create_function(|_, (name, x): (String, String)| {
            log::$level!(target: format!("(plugin) {name}").as_str(), "{x}");

            Ok(())
        })?
    };
}

#[derive(Debug)]
pub struct Engine {
    lua: Lua,
}

impl Engine {
    pub fn on_rx(&self, data: &[u8]) {
        self.lua
            .context(|lua| {
                let f = lua.globals().get::<_, Table>("PLUGINS")?;
                for pair in f.pairs::<Value, Table>() {
                    let (_, v) = pair?;
                    let f: Option<Function> = v.get("on_rx")?;
                    if let Some(f) = f {
                        f.call(data.to_vec())?;
                    }
                }

                Ok::<_, color_eyre::eyre::Error>(())
            })
            .expect("lua err");
    }
}

pub fn init() -> Result<Engine> {
    let lua = Lua::new();
    lua.context(|lua| {
        let globals = lua.globals();

        let info = impl_log!(lua, info);
        let warn = impl_log!(lua, warn);
        let error = impl_log!(lua, error);

        lua.load(include_str!("../plugins/init.lua"))
            .set_name("plugin_init")?
            .exec()?;

        globals.set("LOG_INFO", info)?;
        globals.set("LOG_WARN", warn)?;
        globals.set("LOG_ERROR", error)?;

        Ok::<(), color_eyre::eyre::Error>(())
    })?;

    Ok(Engine { lua })
}
