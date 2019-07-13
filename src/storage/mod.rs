use std::sync::Arc;

use mongodb::coll::results::InsertOneResult;
use mongodb::db::ThreadedDatabase;
use mongodb::Bson;
use mongodb::Client;
use mongodb::ThreadedClient;

use serde::Serialize;

pub use super::error::Error;
#[doc(inline)]
pub use super::error::Result;

pub use super::common::{Hour, Room, Teacher};

#[derive(Serialize)]
pub struct Config {
    /// url to connect to cluster
    pub url: String,

    /// database for storage
    pub database: String,

    /// collection for dsb
    pub dsb_coll: String,
}

impl Config {
    /// create new instance
    pub fn new() -> Self {
        Default::default()
    }

    pub fn connect(&self) -> Result<MongoDB> {
        let client = mongodb::Client::with_uri(&format!("mongodb://{}/", self.url))?;

        Ok(Arc::new(MongoDBInner {
            client: client,
            database: self.database.clone(),
            dsb_collection: self.dsb_coll.clone(),
        }))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: String::from("localhost:27017"),
            database: String::from("llg_companion"),
            dsb_coll: String::from("dsb"),
        }
    }
}

#[derive(Serialize)]
struct dsb_update_field {
    updated_at: chrono::NaiveDateTime,
}

impl dsb_update_field {
    pub fn new(time: &chrono::NaiveDateTime) -> Self {
        Self { updated_at: *time }
    }
}

pub type MongoDB = std::sync::Arc<MongoDBInner>;

pub struct MongoDBInner {
    client: Client,
    database: String,
    dsb_collection: String,
}

impl MongoDBInner {
    pub fn Client(&self) -> Client {
        self.client.clone()
    }

    pub fn db(&self) -> mongodb::db::Database {
        self.client.db(&self.database)
    }

    pub fn dsb_coll(&self) -> mongodb::coll::Collection {
        self.db().collection(&self.dsb_collection)
    }

    pub fn dsb_write(&self, document: &super::dsb::DSB) -> Result<()> {
        let bson = mongodb::to_bson(&dsb_update_field::new(&document.updated_at)).unwrap();
        let bson = bson.as_document().unwrap();
        let dsb_in_cache = self.dsb_coll().find_one(Some(bson.clone()), None).unwrap();
        if dsb_in_cache == None {
            let bson = mongodb::to_bson(document).unwrap();
            let bson = bson.as_document().unwrap();
            self.dsb_coll().insert_one(bson.clone(), None)?;
        }
        Ok(())
    }

    pub fn planinfo_write_table(
        &self,
        table: &super::planinfo::Table,
        collection: &str,
    ) -> Result<()> {
        let bson = mongodb::to_bson(table).unwrap();
        let bson = bson.as_document().unwrap();
        let in_cache = self
            .db()
            .collection(collection)
            .find_one(Some(bson.clone()), None)
            .unwrap();
        if in_cache == None {
            let ret = self
                .db()
                .collection(collection)
                .insert_one(bson.clone(), None)?;
        }
        Ok(())
    }
}
