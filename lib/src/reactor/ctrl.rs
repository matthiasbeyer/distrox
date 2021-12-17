use tokio::sync::mpsc::UnboundedSender as Sender;
use tokio::sync::mpsc::UnboundedReceiver as Receiver;

/// Type for sending messages to a reactor
pub type ReactorSender<Request, Reply> = Sender<(Request, ReplySender<Reply>)>;

/// Type that is used by a reactor for receiving messages
pub type ReactorReceiver<Request, Reply> = Receiver<(Request, ReplySender<Reply>)>;

/// Type that represents the channel that has to be send with a request to a reactor for getting an
/// answer back
pub type ReplySender<Reply> = Sender<Reply>;

pub type ReplyReceiver<Reply> = Receiver<Reply>;

