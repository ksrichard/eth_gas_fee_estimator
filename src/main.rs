use clap::Parser;
use http::server::HttpServer;
use simple_logger::SimpleLogger;
use std::sync::mpsc::channel;

mod fee_estimator;
mod gas_price_collector;
mod http;

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    /// HTTP port where API is exposed
    #[arg(short = 'p', long, default_value_t=9999)]
    port: u16,

    /// Ethereum client JSON-RPC URL.
    /// Example: https://mainnet.infura.io/v3/<YOUR_API_KEY>
    #[arg(short = 'u', long)]
    eth_json_rpc_client_url: url::Url,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    SimpleLogger::new()
        .init()
        .expect("Failed to initialize logging");
    log::set_max_level(log::LevelFilter::Info);

    let cli = Cli::parse();

    // termination handler
    let (term_tx, term_rx) = channel();
    ctrlc::set_handler(move || term_tx.send(()).expect("Can't send signal on channel"))?;

    // start HTTP server
    let mut server = HttpServer::new();
    server.start(&cli).await?;

    // wait for termination
    term_rx.recv()?;

    // stop server
    server.stop().await?;

    Ok(())
}
