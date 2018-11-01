#[cfg(test)]
#[macro_use]
extern crate serde_json;
extern crate comrak;
extern crate failure;
extern crate serde;
#[cfg(not(test))]
extern crate serde_json;
extern crate serde_yaml;
extern crate textwrap;

mod context;
mod convert;
mod section;

pub use context::*;
pub use convert::convert;
