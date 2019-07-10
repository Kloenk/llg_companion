
#[doc(inline)]
pub use super::error::Result;

/// config struct for planinfo
pub struct Config {
    /// baseurl for planifo
    /// is a hidden setting, and default to `https://selbstlernportal.de/html/planinfo/planinfo_start.php`
    pub base_url: String,

    /// school id to identify the school to planinfo
    pub school_id: String,

    /// cookies for auth at planinfo
    pub cookies: String,
}

impl Config {
    /// create a new instance of Config
    pub fn new() -> Self {
        Self {
            base_url: String::from("https://selbstlernportal.de/html/planinfo/planinfo_start.php"),
            school_id: String::new(),
            cookies: String::new(),
        }
    }

    /// start parsing
    pub fn run(self) -> Result<()> {
        std::thread::spawn(move || {
            
        });
        Ok(())
    }
}