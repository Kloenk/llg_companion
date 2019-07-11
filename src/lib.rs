//! llgCompanion parse, web server and calcdav server

/// common data types
pub mod common;

/// dsb parser, loader and config
pub mod dsb;

/// planinfo parser, loader and config
pub mod planinfo;

/// error structs
pub mod error;

/// http server
pub mod server;

#[doc(inline)]
pub use error::Result;

pub struct Config {
    /// verbose level to run
    pub verbose: u8,
    /// config for dsb parser
    pub dsb: dsb::Config,

    /// config for planinfo parser
    pub planino: planinfo::Config,

    /// url to impressum of host
    pub impressum: String,

    /// port to listen on
    pub port: u16,

    /// address to listen on
    pub address: String,
}

impl Config {
    /// create a new instance of config
    pub fn new() -> Self {
        Self {
            verbose: 0,
            dsb: dsb::Config::new(),
            planino: planinfo::Config::new(),
            impressum: String::from("localhost"),
            port: 8080,
            address: String::from("0.0.0.0"),
        }
    }

    /// run function of the lib
    pub fn run(&self) -> Result<()> {
        println!("llgCompanion: {}", env!("CARGO_PKG_VERSION"));

        self.dsb.run()?;

        // run server
        let server = server::Server::new(&self);
        server.run();

        Ok(())
    }
}
