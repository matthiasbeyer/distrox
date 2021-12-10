use std::fmt::Debug;

use tokio::sync::mpsc::UnboundedSender as Sender;
use tokio::sync::mpsc::UnboundedReceiver as Receiver;

/// Type for sending messages to a reactor
pub type ReactorSender<CustomRequest, CustomReply> = Sender<(ReactorRequest<CustomRequest>, ReplyChannel<CustomReply>)>;

/// Type that is used by a reactor for receiving messages
pub type ReactorReceiver<CustomRequest, CustomReply> = Receiver<(ReactorRequest<CustomRequest>, ReplyChannel<CustomReply>)>;

/// Type that represents the channel that has to be send with a request to a reactor for getting an
/// answer back
pub type ReplyChannel<CustomReply> = Sender<ReactorReply<CustomReply>>;

pub type ReplyReceiver<CustomReply> = Receiver<ReactorReply<CustomReply>>;

/// Send control messages to the reactor
#[derive(Debug)]
pub enum ReactorRequest<CustomRequest: Debug + Send + Sync> {
    /// check if the reactor still responds
    Ping,

    /// Quit the reactor
    Exit,

    Custom(CustomRequest),
}

#[derive(Debug)]
pub enum ReactorReply<CustomReply: Debug + Send + Sync> {
    Pong,
    Exiting,

    Custom(CustomReply),
}
