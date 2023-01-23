use std::error::Error;

#[derive(Debug)]
pub enum Outgoing {
    TcpHello { id: String },
    UdpHello { id: String },
}

impl Outgoing {
    pub fn serilaize(self) -> Result<Vec<u8>, Box<dyn Error>> {
        match self {
            Outgoing::TcpHello { id } => Ok([&vec![34 as u8, 0, 1, 0], id.as_bytes()].concat()),
            Outgoing::UdpHello { id } => Ok([&vec![34 as u8, 0, 1, 0], id.as_bytes()].concat()),
        }
    }
}
