use std::error::Error;

use crate::math::Vector3;

const HELLO_FROM_TCP: &[u8] = &[1, 0];
const HELLO_FROM_UDP: &[u8] = &[2, 0];
const WELCOME: &[u8] = &[3, 0];
const GOOD_BYE: &[u8] = &[4, 0];
const INTRODUCE: &[u8] = &[5, 0];
const UPDATE_ORIGIN: &[u8] = &[6, 0];
const UPDATE_ROTATION: &[u8] = &[7, 0];

#[derive(Debug)]
pub enum Outgoing {
    HelloFromTcp { id: String },
    HelloFromUdp { id: String },
    Welcome { id: String },
    GoodBye { id: String },
    Introduce { ids: Vec<String> },
    UpdateOrigin { id: String, origin: Vector3 },
    UpdateRotation { id: String, y: f32 },
}

impl Outgoing {
    pub fn serilaize(self) -> Result<Vec<u8>, Box<dyn Error>> {
        match self {
            Outgoing::HelloFromTcp { id } => Ok([HELLO_FROM_TCP, &id.into_bytes()].concat()),
            Outgoing::HelloFromUdp { id } => Ok([HELLO_FROM_UDP, &id.into_bytes()].concat()),
            Outgoing::Welcome { id } => Ok([WELCOME, &id.into_bytes()].concat()),
            Outgoing::GoodBye { id } => Ok([GOOD_BYE, &id.into_bytes()].concat()),
            Outgoing::Introduce { ids } => Ok([
                INTRODUCE,
                &ids.into_iter()
                    .flat_map(|id| id.into_bytes())
                    .collect::<Vec<u8>>(),
            ]
            .concat()),
            Outgoing::UpdateOrigin { id, origin } => Ok([
                UPDATE_ORIGIN,
                &id.into_bytes(),
                &origin.x.to_le_bytes(),
                &origin.y.to_le_bytes(),
                &origin.z.to_le_bytes(),
            ]
            .concat()),
            Outgoing::UpdateRotation { id, y } => {
                Ok([UPDATE_ROTATION, &id.into_bytes(), &y.to_le_bytes()].concat())
            }
        }
    }
}
