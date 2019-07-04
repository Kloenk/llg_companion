
#[doc(inline)]
pub use super::error::Result;

/// config struct for dsb informations
pub struct Config {
    /// userid to use
    pub user_id: String,

    /// password for dsb
    pub password: String,

    /// cookie for dsb authentification
    pub cookie: String,

    /// url for dsb
    /// only use when you use another host for dsb
    /// defaults to `https://www.dsbmobile.de/JsonHandlerWeb.ashx/GetData`
    pub url: String,
}

impl Config {
    /// create a new instance of Config
    pub fn new() -> Self {
        Self {
            user_id: String::new(),
            password: String::new(),
            cookie: String::new(),
            url: String::from("https://www.dsbmobile.de/JsonHandlerWeb.ashx/GetData"),
        }
    }
}