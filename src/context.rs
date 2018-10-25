use failure;
use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

pub trait Context {
    fn lookup(&self, key: &str) -> Result<String, failure::Error>;
}

pub struct StaticContext {
    pub aliases: HashMap<String, String>,
}

impl Context for StaticContext {
    fn lookup(&self, key: &str) -> Result<String, failure::Error> {
        self.aliases
            .get(key)
            .map(|v| v.to_string())
            .ok_or_else(|| failure::err_msg("key not found"))
    }
}

impl StaticContext {
    pub fn new(key: &str, value: &str) -> StaticContext {
        let mut m = HashMap::new();
        m.insert(key.into(), value.into());
        StaticContext { aliases: m }
    }

    pub fn with(mut self, key: &str, value: &str) -> StaticContext {
        self.aliases.insert(key.into(), value.into());
        self
    }
}

pub struct DirContext {
    pub dir: PathBuf,
}

impl Context for DirContext {
    fn lookup(&self, key: &str) -> Result<String, failure::Error> {
        let mut file = File::open(self.dir.join(key))?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }
}

pub struct CachedContext<Context> {
    root: Box<Context>,
    cache: HashMap<String, String>,
}

//impl Context for CachedContext<Context> {
//    fn lookup(&self, key: &str) -> Result<String, failure::Error> {
//        (*self.root).lookup(key)
//    }
//}
