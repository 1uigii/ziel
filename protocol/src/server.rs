#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    #[error("invalid bytes")]
    InvalidBytes,
    #[error("{0}")]
    InvalidLogic(#[from] logic::Error),
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Handshake,

    Invalid,
    TerminateConnection,

    RequestShips,
    RequestTarget,

    InformTargetSelection,
    InformTargetMissClient(logic::Position),
    InformTargetMissOpponent(logic::Position),
    InformTargetHitClient(logic::Position),
    InformTargetHitOpponent(logic::Position),
    InformShipSunkenClient(logic::ship::Ship),
    InformShipSunkenOpponent(logic::ship::Ship),
    InformLoss,
    InformVictory,
}

impl crate::raw::IntoMessage for Message {
    fn into_raw_message(self) -> crate::raw::Message {
        match self {
            Message::Handshake => crate::raw::HANDSHAKE.to_message(),
            Message::Invalid => crate::raw::INVALID.to_message(),
            Message::TerminateConnection => crate::raw::TERMINATE.to_message(),
            Message::RequestShips => crate::raw::REQUEST_SHIPS.to_message(),
            Message::RequestTarget => crate::raw::REQUEST_TARGET.to_message(),
            Message::InformTargetSelection => crate::raw::INFORM_TARGET_SELECTION.to_message(),
            Message::InformTargetMissClient(pos) => crate::raw::Message {
                type_marker: crate::raw::TYPE_INFORM_MISS,
                body: vec![0, pos.to_byte()],
            },
            Message::InformTargetMissOpponent(pos) => crate::raw::Message {
                type_marker: crate::raw::TYPE_INFORM_MISS,
                body: vec![1, pos.to_byte()],
            },
            Message::InformTargetHitClient(pos) => crate::raw::Message {
                type_marker: crate::raw::TYPE_INFORM_HIT,
                body: vec![0, pos.to_byte()],
            },
            Message::InformTargetHitOpponent(pos) => crate::raw::Message {
                type_marker: crate::raw::TYPE_INFORM_HIT,
                body: vec![1, pos.to_byte()],
            },
            Message::InformShipSunkenClient(ship) => crate::raw::Message {
                type_marker: crate::raw::TYPE_INFORM_SHIP_SUNKEN,
                body: match ship.to_ship_plan() {
                    logic::ship::ShipPlan::Horizontal { pos, length } => {
                        vec![0, 0, pos.to_byte(), length]
                    }
                    logic::ship::ShipPlan::Vertical { pos, length } => {
                        vec![0, 1, pos.to_byte(), length]
                    }
                },
            },
            Message::InformShipSunkenOpponent(ship) => crate::raw::Message {
                type_marker: crate::raw::TYPE_INFORM_SHIP_SUNKEN,
                body: match ship.to_ship_plan() {
                    logic::ship::ShipPlan::Horizontal { pos, length } => {
                        vec![1, 0, pos.to_byte(), length]
                    }
                    logic::ship::ShipPlan::Vertical { pos, length } => {
                        vec![1, 1, pos.to_byte(), length]
                    }
                },
            },
            Message::InformLoss => crate::raw::INFORM_LOSS.to_message(),
            Message::InformVictory => crate::raw::INFORM_VICTORY.to_message(),
        }
    }
}

impl crate::raw::TryFromMessage for Message {
    type Error = Error;

    fn try_from_raw_message(message: crate::raw::Message) -> Result<Self, Self::Error> {
        match message.as_match() {
            crate::raw::HANDSHAKE => Ok(Message::Handshake),
            crate::raw::INVALID => Ok(Message::Invalid),
            crate::raw::TERMINATE => Ok(Message::TerminateConnection),
            crate::raw::REQUEST_SHIPS => Ok(Message::RequestShips),
            crate::raw::REQUEST_TARGET => Ok(Message::RequestTarget),
            crate::raw::INFORM_TARGET_SELECTION => Ok(Message::InformTargetSelection),
            crate::raw::MessageMatch {
                type_marker: crate::raw::TYPE_INFORM_MISS,
                body: [0, pos],
            } => Ok(Message::InformTargetMissClient(
                pos.clone().try_into().map_err(logic::Error::from)?,
            )),
            crate::raw::MessageMatch {
                type_marker: crate::raw::TYPE_INFORM_MISS,
                body: [1, pos],
            } => Ok(Message::InformTargetMissOpponent(
                pos.clone().try_into().map_err(logic::Error::from)?,
            )),
            crate::raw::MessageMatch {
                type_marker: crate::raw::TYPE_INFORM_HIT,
                body: [0, pos],
            } => Ok(Message::InformTargetHitClient(
                pos.clone().try_into().map_err(logic::Error::from)?,
            )),
            crate::raw::MessageMatch {
                type_marker: crate::raw::TYPE_INFORM_HIT,
                body: [1, pos],
            } => Ok(Message::InformTargetHitOpponent(
                pos.clone().try_into().map_err(logic::Error::from)?,
            )),
            crate::raw::MessageMatch {
                type_marker: crate::raw::TYPE_INFORM_SHIP_SUNKEN,
                body: [0, rotation, pos, length],
            } => {
                let pos = pos.clone().try_into().map_err(logic::Error::from)?;
                let length = length.clone();

                Ok(Message::InformShipSunkenClient(
                    if *rotation == 0 {
                        logic::ship::ShipPlan::Horizontal { pos, length }
                    } else {
                        logic::ship::ShipPlan::Vertical { pos, length }
                    }
                    .try_into()
                    .map_err(logic::Error::from)?,
                ))
            }
            crate::raw::MessageMatch {
                type_marker: crate::raw::TYPE_INFORM_SHIP_SUNKEN,
                body: [1, rotation, pos, length],
            } => {
                let pos = pos.clone().try_into().map_err(logic::Error::from)?;
                let length = length.clone();

                Ok(Message::InformShipSunkenOpponent(
                    if *rotation == 0 {
                        logic::ship::ShipPlan::Horizontal { pos, length }
                    } else {
                        logic::ship::ShipPlan::Vertical { pos, length }
                    }
                    .try_into()
                    .map_err(logic::Error::from)?,
                ))
            }
            crate::raw::INFORM_LOSS => Ok(Message::InformLoss),
            crate::raw::INFORM_VICTORY => Ok(Message::InformVictory),
            _ => Err(Error::InvalidBytes),
        }
    }
}
