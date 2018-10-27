use failure;
use std::{cell::RefCell, collections::HashMap, fs::File, io::Read, path::PathBuf};
use textwrap::dedent as d;

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
        m.insert(key.into(), d(value));
        StaticContext { aliases: m }
    }

    pub fn with(mut self, key: &str, value: &str) -> StaticContext {
        self.aliases.insert(key.into(), d(value));
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

impl DirContext {
    pub fn new(dir: PathBuf) -> DirContext {
        DirContext { dir }
    }
}

pub struct CachedContext<Context> {
    root: Box<Context>,
    cache: RefCell<HashMap<String, Option<String>>>,
}

impl<T> Context for CachedContext<T>
where
    T: Context,
{
    fn lookup(&self, key: &str) -> Result<String, failure::Error> {
        match self.cache.borrow().get(key) {
            Some(v) => match v {
                Some(v) => return Ok(v.to_string()),
                None => return Err(failure::err_msg("not found")),
            },
            None => {}
        };

        (&self.root)
            .lookup(key)
            .map(|v| {
                self.cache
                    .borrow_mut()
                    .insert(key.to_string(), Some(v.clone()));
                v
            }).map_err(|e| {
                self.cache.borrow_mut().insert(key.to_string(), None);
                e
            })
    }
}

impl<T> CachedContext<T>
where
    T: Context,
{
    pub fn new(root: Box<T>) -> CachedContext<T> {
        CachedContext {
            root,
            cache: RefCell::new(HashMap::new()),
        }
    }
}
