#[derive(thiserror::Error, Debug)]
#[error("ui :: {0}")]
pub struct Error<I: UI>(I::Error);

impl<I: UI> Error<I> {
    pub fn to_ui_error(error: I::Error) -> Error<I> {
        Error::<I>(error)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ClientInfo<'i> {
    pub messages: &'i [Message],
    pub ships: &'i logic::Ships,
    pub client_hit_map: &'i [[Option<crate::AttackInfo>; 10]; 10],
    pub opponent_hit_map: &'i [[Option<crate::AttackInfo>; 10]; 10],
    pub opponent_ships: &'i [logic::ship::Ship],
}

impl<'i> From<&'i crate::Client> for ClientInfo<'i> {
    fn from(client: &'i crate::Client) -> Self {
        ClientInfo {
            messages: &client.messages,
            ships: &client.ships,
            client_hit_map: &client.client_hit_map,
            opponent_hit_map: &client.opponent_hit_map,
            opponent_ships: &client.opponent_ships,
        }
    }
}

pub trait UI {
    type Error: std::error::Error;

    fn request_ships(&mut self) -> Result<logic::Ships, Self::Error>;
    fn request_target(&mut self, info: ClientInfo) -> Result<logic::Position, Self::Error>;

    fn display_board(&mut self, info: ClientInfo) -> Result<(), Self::Error>;
    fn display_victory(&mut self, info: ClientInfo) -> Result<(), Self::Error>;
    fn display_loss(&mut self, info: ClientInfo) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Message {
    OpponentSelectsTarget,
    ClientMissedOpponent(logic::Position),
    OpponentMissedClient(logic::Position),
    ClientHitOpponent(logic::Position),
    OpponentHitClient(logic::Position),
    OpponentShipSunk(u8),
    ClientShipSunk(u8),
}
