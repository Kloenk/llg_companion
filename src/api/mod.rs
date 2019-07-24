use rocket::data::FromData;
use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::outcome::Outcome;
use rocket::outcome::Outcome::*;
use rocket::request::{self, FromRequest, Request};
use rocket_contrib::json::Json;

use serde::{Deserialize, Serialize};

use super::planinfo::Table;
use super::{DataBase, DbConn};

use mongodb::db::ThreadedDatabase;
use mongodb::ThreadedClient;
//use rocket_contrib::databases::mongodb::ThreadedClient;

use super::common::Hour;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Name {
    pub name: String,
    pub dbidx: isize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: usize,
    pub name: String,
    pub hash: Option<String>,
    pub device: Option<String>,
    pub known_devices: Option<Vec<String>>,
    pub activ: bool,
    pub is_admin: Option<bool>,
}

impl User {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a, 'r> rocket::request::FromRequest<'a, 'r> for User {
    type Error = String;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, String> {
        /* request.cookies()
        .get_private("user_id")
        .and_then(|cookie| cookie.value().parse().ok())
        .map(|id| Self {id})
        .or_forward(()) */
        let id = match request.cookies().get_private("id") {
            Some(cookie) => cookie,
            None => {
                //return Outcome::Failure((Status::new(401, "auth required"), String::from("no cookie field id")));
                return Outcome::Forward(());
            }
        };
        let id = id.value();
        let id = id.parse().unwrap_or(0);
        let pool = super::DbConn::from_request(request);
        let pool = match pool {
            Success(pool) => pool,
            _ => {
                (return Outcome::Failure((
                    Status::new(500, "could not connect to db"),
                    String::from("test"),
                )))
            }
        };
        let pool: mongodb::db::Database = pool.clone();
        let doc = doc! {
            "id": id
        };
        if let Ok(result) = pool.collection("users").find_one(Some(doc.clone()), None) {
            if let Some(result) = result {
                if let Ok(result) = bson::from_bson::<User>(bson::Bson::Document(result)) {
                    return Outcome::Success(result);
                }
            }
        }

        Outcome::Forward(())
    }
}

impl Default for User {
    fn default() -> Self {
        Self {
            name: String::new(),
            id: 0,
            hash: None,
            device: None,
            known_devices: None,
            activ: false,
            is_admin: None,
        }
    }
}

#[get("/names/<name>/<kind>")]
pub fn name(
    name: String,
    kind: u8,
    user: User,
    conn: DbConn,
    db: rocket::State<DataBase>,
) -> Option<Json<Vec<Name>>> {
    /*let bson = mongodb::to_bson(&name::new(&name)).unwrap();
    let bson = bson.as_document().unwrap();
    let conn: mongodb::Client = *conn;
    let from_db = conn.db(&db.0).collection("users").find_one(Some(bson.clone()), None).unwrap();
    if let Some(plan) = from_db {
        return Some(Json(plan));
    }

    let conn: mongodb::db::Database = conn.clone();
    let from_db = conn.collection("students").find_one(Some(doc.clone()), None).unwrap();
    let from_db = bson::from_bson::<Table>(bson::Bson::Document(from_db.unwrap()));
    if let Ok(table) = from_db {
        return Some(Json(table));
    }*/

    let doc = doc! {
        "$text": { "$regex": name }
    };

    let name = match kind {
        0 => "teachers",
        1 => "room",
        2 => "students",
        _ => return None,
    };

    let mut return_value = Vec::new();

    let conn: mongodb::db::Database = conn.clone();
    if let Ok(results) = conn.collection(name).find(Some(doc.clone()), None) {
        for result in results {
            if let Ok(item) = result {
                let item = bson::from_bson::<Name>(bson::Bson::Document(item));
                if let Ok(item) = item {
                    return_value.push(item);
                } else if let Err(err) = item {
                    eprintln!("pars error: {}", err);
                }
            }
        }
    }

    if return_value.len() != 0 {
        return Some(Json(return_value));
    }
    None
}

#[get("/names/<name>", rank = 2)]
pub fn name_all(
    name: String,
    user: User,
    conn: DbConn,
    db: rocket::State<DataBase>,
) -> Option<Json<Vec<Name>>> {
    let doc = doc! {
        "$text": { "$search": name, "$language": "none" }
    };

    let mut return_value = Vec::new();

    let conn: mongodb::db::Database = conn.clone();
    if let Ok(results) = conn.collection("teachers").find(Some(doc.clone()), None) {
        for result in results {
            if let Ok(item) = result {
                if let Ok(item) = bson::from_bson::<Name>(bson::Bson::Document(item)) {
                    return_value.push(item);
                }
            }
        }
    }
    if let Ok(results) = conn.collection("room").find(Some(doc.clone()), None) {
        for result in results {
            if let Ok(item) = result {
                if let Ok(item) = bson::from_bson::<Name>(bson::Bson::Document(item)) {
                    return_value.push(item);
                }
            }
        }
    }
    if let Ok(results) = conn.collection("students").find(Some(doc.clone()), None) {
        for result in results {
            if let Ok(item) = result {
                if let Ok(item) = bson::from_bson::<Name>(bson::Bson::Document(item)) {
                    return_value.push(item);
                }
            }
        }
    }

    if return_value.len() != 0 {
        return Some(Json(return_value));
    }
    None
}

#[get("/plan/<id>", rank = 2)]
pub fn plan(id: i64, user: User, conn: DbConn) -> Option<Json<Table>> {
    let doc = doc! {
        "dbidx": id
    };

    println!("requesting {}", id);

    let conn: mongodb::db::Database = conn.clone();
    if let Ok(results) = conn
        .collection("teachers")
        .find_one(Some(doc.clone()), None)
    {
        if let Some(item) = results {
            if let Ok(item) = bson::from_bson::<Table>(bson::Bson::Document(item)) {
                return Some(Json(item));
            }
        }
    }
    if let Ok(results) = conn.collection("room").find_one(Some(doc.clone()), None) {
        if let Some(item) = results {
            if let Ok(item) = bson::from_bson::<Table>(bson::Bson::Document(item)) {
                return Some(Json(item));
            }
        }
    }
    if let Ok(results) = conn
        .collection("students")
        .find_one(Some(doc.clone()), None)
    {
        if let Some(item) = results {
            if let Ok(item) = bson::from_bson::<Table>(bson::Bson::Document(item)) {
                return Some(Json(item));
            }
        }
    }

    None
}
