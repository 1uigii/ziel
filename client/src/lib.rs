use protocol::{client, server};
use tokio::{io, net};

#[derive(thiserror::Error, Debug)]
pub enum Error<I: UI> {
    #[error("client :: server request :: {0}")]
    Protocol(#[from] protocol::Error<server::Message>),
    #[error("client :: networking :: {0}")]
    Networking(#[from] io::Error),
    #[error("client :: {0}")]
    UIError(#[from] UIError<I>),
    #[error("client :: server request :: unexpected request")]
    ServerRequestUnexpected,
}

#[derive(thiserror::Error, Debug)]
#[error("ui :: {0}")]
pub struct UIError<I: UI>(I::Error);

impl<I: UI> UIError<I> {
    pub fn to_ui_error(error: I::Error) -> UIError<I> {
        UIError::<I>(error)
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

pub enum Message {}

pub struct ClientInfo<'i> {
    messages: &'i [Message],
    board: &'i logic::Board,
    hitmap: &'i [[bool; 10]; 10],
}

impl<'i, I: UI> From<&'i Client<I>> for ClientInfo<'i> {
    fn from(client: &'i Client<I>) -> Self {
        ClientInfo {
            messages: &client.messages,
            board: &client.board,
            hitmap: &client.hitmap,
        }
    }
}

pub struct Client<I: UI> {
    stream: net::TcpStream,

    ui: I,
    messages: Vec<Message>,

    board: logic::Board,
    hitmap: [[bool; 10]; 10],
}

impl<I: UI> Client<I> {
    pub async fn handshake(mut ui: I, addr: std::net::SocketAddr) -> Result<Client<I>, Error<I>> {
        let board = logic::Board::from_ships(ui.request_ships().map_err(UIError::to_ui_error)?);

        let mut stream = net::TcpStream::connect(addr).await?;
        protocol::write(&mut stream, client::Message::HandShake).await?;
        match protocol::read(&mut stream).await? {
            server::Message::Handshake => {}
            _ => return Err(Error::<I>::ServerRequestUnexpected),
        }

        Ok(Client {
            stream,
            ui,
            messages: vec![],
            board,
            hitmap: [[false; 10]; 10],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
