use tokio::{io, net};

use protocol::{client, server};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("server :: io :: {0}")]
    Io(#[from] io::Error),
    #[error("server :: {0}")]
    ServerRequest(#[from] protocol::Error<server::Message>),
    #[error("server :: client response :: {0}")]
    ClientResponse(#[from] protocol::Error<client::Message>),
    #[error("server :: client response :: unexpected response :: {0:?} to {1:?}")]
    ClientResponseUnexpected(server::Message, client::Message),
    #[error("server :: client response :: logic :: {0}")]
    ClientResponseTargetAlreadyHit(#[from] logic::board::AlreadyHitError),
}

pub enum Response {
    Successful,

    ReturnShips(logic::Ships),
    ReturnTarget(logic::Position),
}

pub struct Stream {
    stream: net::TcpStream,
}

impl Stream {
    pub async fn handshake(mut stream: net::TcpStream) -> Result<Stream, Error> {
        match protocol::read(&mut stream).await? {
            client::Message::HandShake => {
                protocol::write(&mut stream, server::Message::Handshake).await?
            }
            res => {
                return Err(Error::ClientResponseUnexpected(
                    server::Message::Handshake,
                    res,
                ))
            }
        }
        Ok(Stream { stream })
    }

    pub async fn request(&mut self, req: protocol::server::Message) -> Result<Response, Error> {
        protocol::write(&mut self.stream, req.clone()).await?;
        let res = protocol::read(&mut self.stream).await?;

        match (req, res) {
            (
                server::Message::Invalid
                | server::Message::TerminateConnection
                | server::Message::InformTargetSelection
                | server::Message::InformTargetMissClient(..)
                | server::Message::InformTargetMissOpponent(..)
                | server::Message::InformTargetHitClient(..)
                | server::Message::InformTargetHitOpponent(..)
                | server::Message::InformLoss
                | server::Message::InformVictory,
                client::Message::Acknowledge,
            ) => Ok(Response::Successful),
            (server::Message::RequestShips, client::Message::ReturnShips(ships)) => {
                Ok(Response::ReturnShips(ships))
            }
            (server::Message::RequestTarget, client::Message::ReturnTarget(target)) => {
                Ok(Response::ReturnTarget(target))
            }
            (req, res) => Err(Error::ClientResponseUnexpected(req, res)),
        }
    }

    pub async fn request_board(&mut self) -> Result<logic::Ships, Error> {
        match self.request(server::Message::RequestShips).await? {
            Response::ReturnShips(ships) => Ok(ships),
            _ => unreachable!("request match statement fallible"),
        }
    }

    pub async fn request_target(&mut self) -> Result<logic::Position, Error> {
        match self.request(server::Message::RequestTarget).await? {
            Response::ReturnTarget(target) => Ok(target),
            _ => unreachable!("request match statement fallible"),
        }
    }

    pub async fn request_inform_target_selection(&mut self) -> Result<(), Error> {
        match self.request(server::Message::InformTargetSelection).await? {
            Response::Successful => Ok(()),
            _ => unreachable!("request match statement fallible"),
        }
    }

    pub async fn request_inform_victory(&mut self) -> Result<(), Error> {
        match self.request(server::Message::InformVictory).await? {
            Response::Successful => Ok(()),
            _ => unreachable!("request match statement fallible"),
        }
    }

    pub async fn request_inform_loss(&mut self) -> Result<(), Error> {
        match self.request(server::Message::InformLoss).await? {
            Response::Successful => Ok(()),
            _ => unreachable!("request match statement fallible"),
        }
    }

    pub async fn request_inform_attack_info_client(
        &mut self,
        attack_info: logic::board::AttackInfo,
        pos: logic::Position,
    ) -> Result<(), Error> {
        match self
            .request(match attack_info {
                logic::board::AttackInfo::Hit(sunken) => {
                    server::Message::InformTargetHitClient(pos, sunken)
                }
                logic::board::AttackInfo::Miss => server::Message::InformTargetMissClient(pos),
            })
            .await?
        {
            Response::Successful => Ok(()),
            _ => unreachable!("request match statement fallible"),
        }
    }

    pub async fn request_inform_attack_info_opponent(
        &mut self,
        attack_info: logic::board::AttackInfo,
        pos: logic::Position,
    ) -> Result<(), Error> {
        match self
            .request(match attack_info {
                logic::board::AttackInfo::Hit(sunken) => {
                    server::Message::InformTargetHitOpponent(pos, sunken)
                }
                logic::board::AttackInfo::Miss => server::Message::InformTargetMissOpponent(pos),
            })
            .await?
        {
            Response::Successful => Ok(()),
            _ => unreachable!("request match statement fallible"),
        }
    }
}
