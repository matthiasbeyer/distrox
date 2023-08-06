pub type CommandReceiver = tokio::sync::mpsc::Receiver<Command>;
pub type CommandSender = tokio::sync::mpsc::Sender<Command>;

/// A command gets send from the frontend to the backend
#[derive(Debug)]
pub enum Command {
    QuitApp,

    PostText { text: String },

    ConnectTo { uri: String },
}
