use tracing::debug;

#[tokio::main]
async fn main() -> Result<(), distrox_gui::error::Error> {
    tracing_subscriber::fmt::init();

    let (sender, mut receiver) = tokio::sync::mpsc::channel(100);
    tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            debug!("Reveived: {message:?}");
        }
    });

    distrox_gui::start(sender)
}
