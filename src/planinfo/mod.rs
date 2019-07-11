

#[doc(inline)]
pub use super::error::Result;

pub use super::common::{Room, Teacher};

/// config struct for planinfo
pub struct Config {
    /// baseurl for planifo
    /// is a hidden setting, and default to `https://selbstlernportal.de/html/planinfo/planinfo_start.php`
    pub base_url: String,

    /// school id to identify the school to planinfo
    pub school_id: String,

    /// cookies for auth at planinfo
    pub cookies: String,

    /// verbose level
    pub verbose: u8,
}

impl Config {
    /// create a new instance of Config
    pub fn new() -> Self {
        Self {
            base_url: String::from("https://selbstlernportal.de/html/planinfo/planinfo_start.php"),
            school_id: String::new(),
            cookies: String::new(),
            verbose: 0,
        }
    }

    /// start parsing
    pub fn run(self) -> Result<()> {
        std::thread::spawn(move || {});
        Ok(())
    }
}

pub struct PlanInfo {
    /// teachers in planinfo
    pub teachers: Vec<Teacher>,
}


/* pub struct Teacher {
    /// symbol of the teacher
    pub symbol: String,

} */
