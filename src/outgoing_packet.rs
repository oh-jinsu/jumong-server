use std::error::Error;

#[derive(Debug)]
pub enum Outgoing {
    Hello { id: String },
}

impl Outgoing {
    pub fn serilaize(self) -> Result<Vec<u8>, Box<dyn Error>> {
        match self {
            Outgoing::Hello { id } => Ok([&vec![34 as u8, 0, 1, 0], id.as_bytes()].concat()),
        }
    }
}
