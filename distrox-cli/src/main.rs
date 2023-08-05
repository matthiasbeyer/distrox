mod error;

use crate::error::Error;

use futures::FutureExt;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let xdg = xdg::BaseDirectories::with_prefix("distrox")?;

    let (sender, receiver) = tokio::sync::mpsc::channel(100);

    let app = distrox_lib::application::Application::load_from_xdg(xdg).await?;

    let gui_task = tokio::task::spawn_blocking(|| distrox_gui::start(sender).map_err(Error::from))
        .map(|r| match r {
            Ok(res) => res,
            Err(join) => Err(Error::Join(join)),
        });
    let app_task = app.run(receiver).map(|r| r.map_err(Error::from));

    tokio::try_join!(gui_task, app_task)?;
    Ok(())
}
