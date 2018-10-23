use serde_json;
use std::collections::HashMap;

pub struct Context {
    pub aliases: HashMap<String, serde_json::Value>,
}

impl Context {
    pub fn read_dir(dir: &str) -> Context {
        unimplemented!()
    }

    pub fn new(key: &str, value: serde_json::Value) -> Context {
        let mut m = HashMap::new();
        m.insert(key.into(), value);
        Context { aliases: m }
    }

    pub fn with(mut self, key: &str, value: serde_json::Value) -> Context {
        self.aliases.insert(key.into(), value);
        self
    }
}
