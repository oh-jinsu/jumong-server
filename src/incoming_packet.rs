use std::error::Error;

#[derive(Debug)]
pub enum Incoming {
    TcpHello { token: String },
    UdpHello { token: String },
}

impl Incoming {
    pub fn deserialize(buf: &[u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        if buf.len() < 2 {
            return Err(format!("buffer too short to deserialize, {buf:?}").into());
        }

        let body = &buf[2..];

        match &buf[..2] {
            [1, 0] => {
                if body.len() != 76 {
                    return Err(format!("invalid size of body").into());
                }

                Ok(Self::TcpHello {
                    token: String::from_utf8_lossy(body).to_string(),
                })
            }
            [2, 0] => {
                if body.len() != 76 {
                    return Err(format!("invalid size of body").into());
                }

                Ok(Self::UdpHello {
                    token: String::from_utf8_lossy(body).to_string(),
                })
            }
            n => Err(format!("unexpected packet arrived, {n:?}").into()),
        }
    }
}
