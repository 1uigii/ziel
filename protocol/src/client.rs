#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    #[error("invalid bytes")]
    InvalidBytes,
    #[error("{0}")]
    InvalidLogic(#[from] logic::Error),
}

/// Message that gets send from the client and is received from the server
#[derive(Debug, Clone, Copy)]
pub enum Message {
    HandShake,

    Acknowledge,

    ReturnShips(logic::Ships),
    ReturnTarget(logic::Position),
}

impl crate::raw::IntoMessage for Message {
    fn into_raw_message(self) -> crate::raw::Message {
        match self {
            Message::HandShake => crate::raw::HANDSHAKE.to_message(),
            Message::Acknowledge => crate::raw::ACKNOWLEDGE.to_message(),
            Message::ReturnShips(ships) => crate::raw::Message {
                type_marker: crate::raw::TYPE_REQ_RET_SHIPS,
                body: ships
                    .into_iter()
                    .flat_map(|ship| match ship.to_ship_plan() {
                        logic::ship::ShipPlan::Horizontal { pos, length } => {
                            [0, pos.to_byte(), length]
                        }
                        logic::ship::ShipPlan::Vertical { pos, length } => {
                            [1, pos.to_byte(), length]
                        }
                    })
                    .collect(),
            },
            Message::ReturnTarget(target) => crate::raw::Message {
                type_marker: crate::raw::TYPE_REQ_RET_TARGET,
                body: vec![target.to_byte()],
            },
        }
    }
}

impl crate::raw::TryFromMessage for Message {
    type Error = Error;

    fn try_from_raw_message(message: crate::raw::Message) -> Result<Self, Self::Error> {
        match message.as_match() {
            crate::raw::HANDSHAKE => Ok(Message::HandShake),
            crate::raw::ACKNOWLEDGE => Ok(Message::Acknowledge),
            crate::raw::MessageMatch {
                type_marker: crate::raw::TYPE_REQ_RET_SHIPS,
                body,
            } if body.len() == 15 => {
                let ships: [_; 5] = body
                    .chunks_exact(3)
                    .map(|chunk| {
                        logic::ship::Ship::try_from(if chunk[0] == 0 {
                            logic::ship::ShipPlan::Horizontal {
                                pos: chunk[1].try_into()?,
                                length: chunk[2],
                            }
                        } else {
                            logic::ship::ShipPlan::Vertical {
                                pos: chunk[1].try_into()?,
                                length: chunk[2],
                            }
                        })
                        .map_err(logic::Error::from)
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    .expect("body is already checked for size");
                Ok(Message::ReturnShips(
                    logic::Ships::try_from(ships).map_err(logic::Error::from)?,
                ))
            }
            crate::raw::MessageMatch {
                type_marker: crate::raw::TYPE_REQ_RET_TARGET,
                body: [pos],
            } => Ok(Message::ReturnTarget(
                pos.clone().try_into().map_err(logic::Error::from)?,
            )),
            _ => Err(Error::InvalidBytes),
        }
    }
}
