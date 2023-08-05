slint::include_modules!();

pub mod error;

use crate::error::Error;

pub fn start() -> Result<(), Error> {
    let ui = AppWindow::new()?;
    install_callbacks(&ui)?;
    ui.run().map_err(Error::from)
}

fn install_callbacks(ui: &AppWindow) -> Result<(), Error> {
    ui.on_post_text_content(move |text| {
        println!("{}", text);
    });

    Ok(())
}
