slint::include_modules!();

pub mod error;
use crate::error::Error;

pub fn start() -> Result<(), Error> {
    let ui = AppWindow::new()?;
    ui.run().map_err(Error::from)
}
