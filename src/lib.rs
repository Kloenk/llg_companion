//! llgCompanion parse, web server and calcdav server

#![feature(proc_macro_hygiene, decl_macro)]
#![feature(never_type)] // FIXME: remove

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use rocket::request::{self, Request, FromRequest};
use rocket::outcome::Outcome::*;

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

/// config for rocket assets directory
struct AssetsDir(String);

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

    /// secret key for rocket
    pub secret: String,

    /// directory for assets
    pub assets: String,
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
            secret: String::new(),
            assets: String::from("assets/"),
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
        database_config.insert(
            "url",
            Value::from(format!("mongodb://{}/", self.storage.url)),
        );
        databases.insert("llg_mongo", Value::from(database_config));

        let mut config = Config::build(Environment::Staging)
            .address(&self.address)
            .port(self.port)
            .extra("databases", databases)
            .finalize().unwrap();

        if !self.secret.is_empty() {
            config.set_secret_key(&self.secret).unwrap();
        }

        let dir = self.assets.clone();

        rocket::custom(config)
            .mount("/", routes![index, files])
            .mount("/admin/", routes![admin])
            .register(catchers![not_found])
            .attach(DbConn::fairing())
            .attach(rocket::fairing::AdHoc::on_attach("assets Config", |rocket| {
                Ok(rocket.manage(AssetsDir(dir)))
            }))
            .launch();
        Ok(())
    }
}

#[database("llg_mongo")]
pub struct DbConn(mongodb::db::Database);

#[catch(404)]
fn not_found(req: &rocket::Request) -> String {
    format!("Sorry, '{}' is not a valid path.", req.uri())
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}


#[get("/<file..>")]
fn files(file: std::path::PathBuf, assets_dir: rocket::State<AssetsDir>) -> Option<rocket::response::NamedFile> {
    rocket::response::NamedFile::open(std::path::Path::new(&assets_dir.0).join(file)).ok()
}

#[get("/")]
fn admin(admin: SuperUser) -> &'static str {
    "Hello, administrator. This is the admin panel!"
}

#[derive(Debug)]
struct SuperUser {
    id: usize,
}

use crate::rocket::outcome::IntoOutcome;

impl<'a, 'r> rocket::request::FromRequest<'a, 'r> for SuperUser {
    type Error = !;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, !> {
        request.cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|id| Self {id})
            .or_forward(())
    }
}
