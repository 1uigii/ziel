#[derive(thiserror::Error, Debug)]
pub enum Error<I: UI> {
    #[error("ui :: {0}")]
    UI(I::Error),
    #[error("ui :: target already hit")]
    InvalidTarget,
}

impl<I: UI> Error<I> {
    pub fn to_ui_error(error: I::Error) -> Error<I> {
        Error::<I>::UI(error)
    }
}

/// Info about the client and game status
#[derive(Clone, Copy, Debug)]
pub struct ClientInfo<'i> {
    /// Messages received from the server will be collected here. The messages
    /// get pushed to a vector, so the last one is the newest.
    pub messages: &'i [Message],
    /// The player's ships.
    pub ships: &'i logic::Ships,
    /// It specifies where the client is already hit. The array is indexed by `[y][x]`
    pub client_hit_map: &'i [[Option<crate::AttackInfo>; 10]; 10],
    /// It specifies where the opponent is already hit. The array is indexed by `[y][x]`
    pub opponent_hit_map: &'i [[Option<crate::AttackInfo>; 10]; 10],
    /// Opponent ships that sunk will be collected in this slice.
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

/// This trait needs to be implemented for a working UI implementation.
/// Any of the return values will get verified, and will return an error, if
/// invalid.
///
/// The connection with the server will be terminated if any of the called
/// functions of this trait return an error.
/// ([`display_loss`] & [`display_victory`] are no longer connected)
///
/// Initialization and termination of the UI happen before the implementation
/// gets bound, and after it gets freed from the [`Client`].
pub trait UI {
    /// The error all the functions will return.
    /// Panicking is not wanted, instead generate nice and readable errors.
    type Error: std::error::Error;

    /// The player will select where to place the ships. This funtion is
    /// blocking, and will wait until the player has positioned all their ships.
    fn request_ships(&mut self) -> Result<logic::Ships, Self::Error>;
    /// The player will select where to attack. This funtion is blocking, and
    /// will wait until the player has selected a target.
    fn request_target(&mut self, info: ClientInfo) -> Result<logic::Position, Self::Error>;

    /// The bard will be displayed. This function should only render one frame,
    /// before returning, as it is already called every 50ms.
    fn display_board(&mut self, info: ClientInfo) -> Result<(), Self::Error>;
    /// The bard will be displayed, including a victory screen. This function
    /// should block until the player terminates the program.
    fn display_victory(&mut self, info: ClientInfo) -> Result<(), Self::Error>;
    /// The bard will be displayed, including a loss screen. This function
    /// should block until the player terminates the program.
    fn display_loss(&mut self, info: ClientInfo) -> Result<(), Self::Error>;
}

/// [`ClientInfo`] will contain messages. These are received from the server
/// and _can_ be displayed by the UI. Not all messages need to be displayed.
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
