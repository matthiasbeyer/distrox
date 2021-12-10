use std::fmt::Debug;

use tokio::sync::mpsc::UnboundedSender as Sender;
use tokio::sync::mpsc::UnboundedReceiver as Receiver;

use crate::reactor::ReactorReply;
use crate::reactor::ReactorRequest;

/// Type for sending messages to a reactor
pub type ReactorSender<CustomRequest, CustomReply> = Sender<(ReactorRequest<CustomRequest>, ReplyChannel<CustomReply>)>;

/// Type that is used by a reactor for receiving messages
pub type ReactorReceiver<CustomRequest, CustomReply> = Receiver<(ReactorRequest<CustomRequest>, ReplyChannel<CustomReply>)>;

/// Type that represents the channel that has to be send with a request to a reactor for getting an
/// answer back
pub type ReplyChannel<CustomReply> = Sender<ReactorReply<CustomReply>>;

pub type ReplyReceiver<CustomReply> = Receiver<ReactorReply<CustomReply>>;

