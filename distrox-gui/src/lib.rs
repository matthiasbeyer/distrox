slint::include_modules!();

pub mod error;

use crate::error::Error;
use distrox_lib::command::Command;
use distrox_lib::command::CommandSender;

pub fn start(sender: CommandSender) -> Result<(), Error> {
    let ui = AppWindow::new()?;
    install_callbacks(&ui, sender)?;
    ui.run().map_err(Error::from)
}

fn install_callbacks(ui: &AppWindow, sender: CommandSender) -> Result<(), Error> {
    ui.on_post_text_content(move |text| {
        let sender = sender.clone();
        tokio::spawn(async move {
            let text = text.to_string();
            let _ = sender.send(Command::PostText { text }).await;
        });
    });

    Ok(())
}
