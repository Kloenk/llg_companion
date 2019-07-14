#![feature(proc_macro_hygiene, decl_macro)]


#[doc(inline)]
pub use crate::error::Result;

/// struct holding server config
pub struct Server {}

impl Server {
    /// create new instance
    pub fn new(_conf: &super::Config) -> Self {
        Self {}
    }
    /// start server
    pub fn run(&self) -> Result<()> {
        rocket::ignite().mount("/", routes![index]).launch();
        Ok(())
    }
}

