use testcontainers::{core::WaitFor, Image};

#[derive(Debug, Default)]
pub struct Ipfs;

impl Image for Ipfs {
    type Args = ();

    fn name(&self) -> String {
        "ipfs/kubo".to_owned()
    }

    fn tag(&self) -> String {
        "latest".to_owned()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("Daemon is ready")]
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![5001]
    }
}
