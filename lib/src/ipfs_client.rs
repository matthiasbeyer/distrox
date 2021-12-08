#[cfg(not(test))]
pub type IpfsClient = ipfs::Ipfs<ipfs::Types>;

#[cfg(test)]
pub type IpfsClient = ipfs::Ipfs<ipfs::TestTypes>;
