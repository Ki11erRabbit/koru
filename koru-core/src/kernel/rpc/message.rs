use mlua::{IntoLua, Lua, LuaSerdeExt};
use crate::kernel::rpc::RpcClient;

pub struct Message {
    destination: RpcClient,
    source: RpcClient,
    args: serde_json::Value
}

impl Message {
    pub fn new(destination: RpcClient, source: RpcClient, args: serde_json::Value) -> Self {
        Message { destination, source, args }
    }
    
    pub fn into_lua(self, lua: &Lua) -> (Vec<impl IntoLua>, Vec<impl IntoLua>) {
        let mut args = Vec::new();
        let mut vaargs = Vec::new();
        args.push(self.source.0.into_lua(lua).unwrap());
        let msg_args = self.args.as_array().unwrap();
        for arg in msg_args {
            vaargs.push(lua.to_value(&arg).unwrap());
        }
        (args, vaargs)
    }
}