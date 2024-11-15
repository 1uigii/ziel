use protocol::{client, server};
use tokio::{
    io::{self, AsyncWriteExt},
    net,
};

pub mod ui;
pub use ui::UI;

#[derive(thiserror::Error, Debug)]
pub enum Error<I: UI> {
    #[error("client :: server request :: {0}")]
    Protocol(#[from] protocol::Error<server::Message>),
    #[error("client :: networking :: {0}")]
    Networking(#[from] io::Error),
    #[error("client :: {0}")]
    UIError(#[from] ui::Error<I>),
    #[error("client :: server request :: unexpected request :: {0:?}")]
    UnexpectedRequest(server::Message),
    #[error("client :: server request :: unexpected termination")]
    UnexpectedTerminationRequest,
}

pub struct Client {
    stream: net::TcpStream,

    messages: Vec<ui::Message>,

    ships: logic::Ships,
    client_hit_map: [[Option<logic::board::AttackInfo>; 10]; 10],
    opponent_hit_map: [[Option<logic::board::AttackInfo>; 10]; 10],
}

impl Client {
    pub async fn handshake<I: UI>(
        ui: &mut I,
        addr: std::net::SocketAddr,
    ) -> Result<Client, Error<I>> {
        let ships = ui.request_ships().map_err(ui::Error::to_ui_error)?;

        let mut stream = net::TcpStream::connect(addr).await?;
        protocol::write(&mut stream, client::Message::HandShake).await?;
        match protocol::read(&mut stream).await? {
            server::Message::Handshake => {}
            req => return Err(Error::UnexpectedRequest(req)),
        }

        Ok(Client {
            stream,
            ships,
            messages: vec![],
            opponent_hit_map: [[None; 10]; 10],
            client_hit_map: [[None; 10]; 10],
        })
    }

    async fn handle_request<I: UI>(&mut self, ui: &mut I) -> Result<Option<bool>, Error<I>> {
        let mut state = None;
        let request = protocol::read(&mut self.stream).await?;
        let response = match request {
            server::Message::RequestShips => client::Message::ReturnShips(self.ships.clone()),
            server::Message::RequestTarget => client::Message::ReturnTarget(
                ui.request_target((self as &Client).into())
                    .map_err(ui::Error::to_ui_error)?,
            ),
            server::Message::InformTargetSelection => {
                self.messages.push(ui::Message::OpponentSelectsTarget);
                client::Message::Acknowledge
            }
            server::Message::InformTargetMissClient(pos) => {
                self.client_hit_map[pos] = Some(logic::board::AttackInfo::Miss);
                self.messages.push(ui::Message::OpponentMissedClient);
                client::Message::Acknowledge
            }
            server::Message::InformTargetMissOpponent(pos) => {
                self.opponent_hit_map[pos] = Some(logic::board::AttackInfo::Miss);
                self.messages.push(ui::Message::ClientMissedOpponent);
                client::Message::Acknowledge
            }
            server::Message::InformTargetHitClient(pos, sunken) => {
                self.client_hit_map[pos] = Some(logic::board::AttackInfo::Hit(sunken));
                self.messages.push(ui::Message::OpponentHitClient);
                if sunken {
                    self.messages.push(ui::Message::ClientShipSunk);
                }
                client::Message::Acknowledge
            }
            server::Message::InformTargetHitOpponent(pos, sunken) => {
                self.opponent_hit_map[pos] = Some(logic::board::AttackInfo::Hit(sunken));
                self.messages.push(ui::Message::ClientHitOpponent);
                if sunken {
                    self.messages.push(ui::Message::OpponentShipSunk);
                }
                client::Message::Acknowledge
            }
            server::Message::InformLoss => {
                state = Some(false);
                client::Message::Acknowledge
            }
            server::Message::InformVictory => {
                state = Some(true);
                client::Message::Acknowledge
            }
            server::Message::Invalid => {
                return Err(Error::UnexpectedRequest(server::Message::Invalid))
            }
            server::Message::TerminateConnection => {
                return Err(Error::UnexpectedTerminationRequest)
            }
            req => return Err(Error::UnexpectedRequest(req)),
        };

        protocol::write(&mut self.stream, response).await?;

        Ok(state)
    }

    pub async fn play<I: UI>(mut self, ui: &mut I) -> Result<bool, Error<I>> {
        loop {
            match self.handle_request(ui).await {
                Ok(Some(victory)) => {
                    if let Ok(server::Message::TerminateConnection) =
                        protocol::read(&mut self.stream).await
                    {
                        let _ =
                            protocol::write(&mut self.stream, client::Message::Acknowledge).await;
                    };
                    self.stream.shutdown().await?;
                    if victory {
                        ui.display_victory((&self).into())
                            .map_err(ui::Error::to_ui_error)?;
                    } else {
                        ui.display_loss((&self).into())
                            .map_err(ui::Error::to_ui_error)?;
                    }
                    return Ok(victory);
                }
                Ok(None) => continue,
                Err(err) => return Err(err),
            }
        }
    }
}
