use tokio::{io, net};

pub(crate) mod stream;
pub(crate) use stream::Stream;
mod game;

async fn handle_connection(
    stream1: net::TcpStream,
    stream2: net::TcpStream,
) -> Result<(), stream::Error> {
    let stream1 = Stream::handshake(stream1).await?;
    let stream2 = Stream::handshake(stream2).await?;

    let game = game::Game::new(stream1, stream2).await?;

    tokio::spawn(async move {
        let _ = game.play().await.unwrap();
        todo!()
    });

    Ok(())
}

pub async fn listen(addr: std::net::SocketAddr) -> Result<(), stream::Error> {
    let listener = net::TcpListener::bind(addr).await?;

    tracing::info!("LISTENING AT [{addr}]");
    loop {
        let (stream1, addr1) = listener.accept().await?;
        tracing::info!("ACCEPTED [{addr1}]; waiting for player two");
        let (stream2, addr2) = listener.accept().await?;
        tracing::info!("ACCEPTED [{addr2}]; beginning match");
        tracing::info!("MATCH [{addr1}] vs [{addr2}]");

        let res = handle_connection(stream1, stream2).await;

        match res {
            Ok(()) => tracing::info!("successfully handled connection"),
            Err(err) => tracing::error!("error handling connection {err}"),
        }
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
