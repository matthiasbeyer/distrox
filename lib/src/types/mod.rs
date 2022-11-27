use crate::error::Error;

mod node;
pub use node::*;

mod datetime;
pub use datetime::*;

mod payload;
pub use payload::*;

pub trait IntoIPLD {
    fn into_ipld(self) -> libipld::Ipld;
}

pub trait FromIPLD: Sized {
    fn from_ipld(ipld: &libipld::Ipld) -> Result<Self, Error>;
}
