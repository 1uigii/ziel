#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub type_marker: u8,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageMatch<'b> {
    pub type_marker: u8,
    pub body: &'b [u8],
}

impl Message {
    pub fn as_match(&self) -> MessageMatch<'_> {
        MessageMatch {
            type_marker: self.type_marker,
            body: &self.body,
        }
    }
}

impl MessageMatch<'_> {
    pub fn to_message(&self) -> Message {
        Message {
            type_marker: self.type_marker,
            body: self.body.to_vec(),
        }
    }
}

pub trait IntoMessage {
    fn into_raw_message(self) -> Message;
}

pub trait TryFromMessage: Sized {
    type Error: std::error::Error;

    fn try_from_raw_message(message: Message) -> Result<Self, Self::Error>;
}

pub const HANDSHAKE: MessageMatch = MessageMatch {
    type_marker: 1,
    body: b"HELO",
};

pub const ACKNOWLEDGE: MessageMatch = MessageMatch {
    type_marker: 2,
    body: b"ACK",
};

pub const INVALID: MessageMatch = MessageMatch {
    type_marker: 1,
    body: b"INVALID",
};

pub const TERMINATE: MessageMatch = MessageMatch {
    type_marker: 1,
    body: b"TERM",
};

pub const TYPE_REQ_RET_SHIPS: u8 = 100;
pub const TYPE_REQ_RET_TARGET: u8 = 101;

pub const REQUEST_SHIPS: MessageMatch = MessageMatch {
    type_marker: TYPE_REQ_RET_SHIPS,
    body: b"REQ SHIP",
};
pub const REQUEST_TARGET: MessageMatch = MessageMatch {
    type_marker: TYPE_REQ_RET_TARGET,
    body: b"REQ TARG",
};

pub const INFORM_TARGET_SELECTION: MessageMatch = MessageMatch {
    type_marker: 150,
    body: b"OPP SELEC TARG",
};

pub const TYPE_INFORM_MISS: u8 = 151;
pub const TYPE_INFORM_HIT: u8 = 152;

pub const INFORM_LOSS: MessageMatch = MessageMatch {
    type_marker: 152,
    body: b"LOSS",
};

pub const INFORM_VICTORY: MessageMatch = MessageMatch {
    type_marker: 152,
    body: b"VICTORY",
};
