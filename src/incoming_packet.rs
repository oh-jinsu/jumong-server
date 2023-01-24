use std::error::Error;

use crate::math::Vector3;

#[derive(Debug)]
pub enum Incoming {
    TcpHello { token: String },
    UdpHello { token: String },
    UpdateOrigin { origin: Vector3 },
    UpdateRotation { y: f32 },
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
            [3, 0] => {
                if body.len() != 12 {
                    return Err(format!("invalid size of body").into());
                }

                let x = f32::from_le_bytes([body[0], body[1], body[2], body[3]]);

                let y = f32::from_le_bytes([body[4], body[5], body[6], body[7]]);

                let z = f32::from_le_bytes([body[8], body[9], body[10], body[11]]);

                let origin = Vector3::new(x, y, z);

                Ok(Self::UpdateOrigin { origin })
            }
            [4, 0] => {
                if body.len() != 4 {
                    return Err(format!("invalid size of body").into());
                }

                let y = f32::from_le_bytes([body[0], body[1], body[2], body[3]]);

                Ok(Self::UpdateRotation { y })
            }
            n => Err(format!("unexpected packet arrived, {n:?}").into()),
        }
    }
}
