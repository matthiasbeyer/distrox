use anyhow::Result;

/// Our own CID type
///
/// Right now the ipfs_api crate does not use a CID type in its interface... hence we would need to
/// convert back-and-forth between String and cid::Cid,... but that's tedious.
///
/// Hence we just create our own "Cid type" and use that as long as the crate API is stringly
/// typed.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Cid(String);

#[cfg(test)]
impl AsRef<str> for Cid {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

pub trait TryToCid {
    fn try_to_cid(self) -> Result<Cid>;
}

impl TryToCid for ipfs_api_backend_hyper::response::AddResponse {
    fn try_to_cid(self) -> Result<Cid> {
        log::debug!("Transforming to CID => {:?}", self);
        string_to_cid(self.hash)
    }
}

/// Helper function that can be tested
///
/// Converts a String to a Cid
fn string_to_cid(s: String) -> Result<Cid> {
    Ok(Cid(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_cid() {
        let s = String::from("QmY2T5EfgLn8qWCt8eus6VX1gJuAp1nmUSdmoehgMxznAf");
        let r = string_to_cid(s);
        assert!(r.is_ok(), "Not OK = {:?}", r);
    }
}
