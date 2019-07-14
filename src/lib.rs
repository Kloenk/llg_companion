//! llgCompanion parse, web server and calcdav server


#![feature(proc_macro_hygiene, decl_macro)]


#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;


/// common data types
pub mod common;

/// dsb parser, loader and config
pub mod dsb;

/// planinfo parser, loader and config
pub mod planinfo;

/// error structs
pub mod error;

/// storage backend
pub mod storage;

#[doc(inline)]
pub use error::Result;

pub struct Config {
    /// verbose level to run
    pub verbose: u8,
    /// config for dsb parser
    pub dsb: dsb::Config,

    /// config for planinfo parser
    pub planino: planinfo::Config,

    /// config for storage
    pub storage: storage::Config,

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
            storage: storage::Config::new(),
            impressum: String::from("localhost"),
            port: 8080,
            address: String::from("0.0.0.0"),
        }
    }

    /// run function of the lib
    pub fn run(&self) -> Result<()> {
        println!("llgCompanion: {}", env!("CARGO_PKG_VERSION"));

        let mongo = self.storage.connect()?;

        self.dsb.run(mongo.clone())?;

        self.planino.run(mongo.clone())?;



        // rocket foo
        use std::collections::HashMap;
        let mut database_config = HashMap::new();
        let mut databases = HashMap::new();

        use rocket::config::{Config, Environment, Value};

        // This is the same as the following TOML:
        // my_db = { url = "database.sqlite" }
        database_config.insert("url", Value::from(format!("mongodb://{}/", self.storage.url)));
        databases.insert("llg_mongo", Value::from(database_config));


        let config = Config::build(Environment::Staging)
            .address(&self.address)
            .port(self.port)
            .extra("databases", databases)
            .finalize().unwrap();

        rocket::custom(config)
            .mount("/", routes![index])
            .attach(DbConn::fairing())
            .launch();
        Ok(())
    }
}


#[database("llg_mongo")]
pub struct DbConn(mongodb::db::Database);


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}