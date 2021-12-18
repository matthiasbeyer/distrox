use std::sync::Arc;
use std::fmt::Debug;

use anyhow::Result;
use tokio::sync::RwLock;

use crate::profile::Profile;

mod gossip;
mod device;
mod account;
mod ctrl;

use ctrl::ReactorReceiver;

#[async_trait::async_trait]
pub trait Reactor {
    type Request: Debug + Send + Sync;
    type Reply: Debug + Send + Sync;

    async fn run(self) -> Result<()>;
}

pub trait ReactorBuilder {
    type Reactor: Reactor;

    fn build_with_receiver(self, rr: ReactorReceiver<<Self::Reactor as Reactor>::Request, <Self::Reactor as Reactor>::Reply>) -> Self::Reactor;
}
