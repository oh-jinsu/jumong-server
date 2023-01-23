use std::error::Error;

#[derive(Debug)]
pub enum Outgoing {
    HelloFromTcp { id: String },
    HelloFromUdp { id: String },
}

impl Outgoing {
    pub fn serilaize(self) -> Result<Vec<u8>, Box<dyn Error>> {
        match self {
            Outgoing::HelloFromTcp { id } => Ok([&[1, 0], id.as_bytes()].concat()),
            Outgoing::HelloFromUdp { id } => Ok([&[2, 0], id.as_bytes()].concat()),
        }
    }
}
