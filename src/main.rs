use std::net;

use clap::Parser;

const DEFAULTADDR: net::SocketAddr =
    net::SocketAddr::new(net::IpAddr::V4(net::Ipv4Addr::new(127, 0, 0, 1)), 8080);

/// ziel - battleship
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
    /// host a server
    Server {
        /// where to listen for clients
        #[arg(short, long, default_value_t = DEFAULTADDR)]
        addr: std::net::SocketAddr,
    },
    /// join a server
    Client {
        /// where to bind for the game server
        #[arg(short, long, default_value_t = DEFAULTADDR)]
        addr: std::net::SocketAddr,
    },
}

async fn run_server(addr: net::SocketAddr) {
    tracing_subscriber::fmt().with_thread_ids(true).init();
    match server::listen(addr).await {
        Ok(()) => {}
        Err(err) => tracing::error!("{err}"),
    }
}

async fn run_client(addr: net::SocketAddr) -> Result<bool, client::Error<tui::Tui>> {
    let mut tui = tui::Tui::init();
    let client = client::Client::handshake(&mut tui, addr).await?;
    client.play(&mut tui).await
}

#[tokio::main]
async fn main() {
    match Args::parse() {
        Args::Server { addr } => run_server(addr).await,
        Args::Client { addr } => match run_client(addr).await {
            Ok(true) => println!("congrats, you won"),
            Ok(false) => println!("you lost, maybe try again?"),
            Err(err) => eprintln!("{err}"),
        },
    }
}
