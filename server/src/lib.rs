use tokio::net;

pub(crate) mod stream;
pub(crate) use stream::Stream;
mod game;

async fn handle_connection(
    stream1: net::TcpStream,
    stream2: net::TcpStream,
) -> Result<(), stream::Error> {
    let stream1 = Stream::handshake(stream1).await?;
    let stream2 = Stream::handshake(stream2).await?;
    tracing::info!("HANDSHAKE successful");

    let game = game::Game::new(stream1, stream2).await?;
    tracing::info!("board initialization successful");

    tokio::spawn(async move {
        match game.play().await {
            Ok(()) => tracing::info!("game thread :: finished successfully"),
            Err(err) => tracing::error!("game thread :: {err}"),
        }
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
