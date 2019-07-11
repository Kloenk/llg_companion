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
        loop {
            std::thread::sleep(std::time::Duration::from_secs(999999));
        }
        Ok(())
    }
}
