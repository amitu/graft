#[cfg(test)]
#[macro_use]
extern crate serde_json;
#[cfg(test)]
#[macro_use]
extern crate assert_snapshot;
extern crate failure;
extern crate serde;
#[cfg(not(test))]
extern crate serde_json;
extern crate sorted_json;
#[cfg(test)]
extern crate textwrap;

mod context;
mod convert;
mod section;

pub use context::Context;
pub use convert::convert;
