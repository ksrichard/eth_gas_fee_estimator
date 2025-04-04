use std::{io, sync::Arc, time::Duration};

use axum::{routing::post, Router};
use log::{error, info};
use thiserror::Error;
use tokio::task::{JoinError, JoinHandle};
use tokio_util::sync::CancellationToken;
use tower_http::timeout::TimeoutLayer;

use crate::{
    fee_estimator::Estimator,
    gas_price_collector::{self, in_memory::InMemoryCollector, Collector},
    Cli,
};

use super::handlers;

const LOG_TARGET: &str = "http_server";

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    IO(#[from] io::Error),
    #[error("Task join error: {0}")]
    TaskJoin(#[from] JoinError),
    #[error("In-memory gas price collector error: {0}")]
    InMemoryCollector(#[from] gas_price_collector::in_memory::Error),
}

/// The main server struct that manages HTTP server and it's related services.
pub struct HttpServer {
    cancel_token: CancellationToken,
    sub_tasks: Vec<JoinHandle<Result<(), Error>>>,
}

impl HttpServer {
    pub fn new() -> Self {
        Self {
            cancel_token: CancellationToken::new(),
            sub_tasks: vec![],
        }
    }

    /// Starts the HTTP server and all of its needed services.
    pub async fn start(&mut self, cli: &Cli) -> Result<(), Error> {
        let gas_price_collector =
            Arc::new(InMemoryCollector::new(cli.eth_json_rpc_client_url.clone()));

        // start collector
        let collector = gas_price_collector.clone();
        let cancel_token = self.cancel_token.clone();
        self.sub_tasks.push(tokio::spawn(async move {
            collector.start(cancel_token).await?;
            Ok(())
        }));

        // http server
        let estimator = Estimator::new(gas_price_collector);
        let app = Router::new()
            .route("/estimate", post(handlers::gas_fee_estimate::handler))
            .with_state(estimator)
            .layer((TimeoutLayer::new(Duration::from_secs(10)),));
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", cli.port)).await?;

        info!(target: LOG_TARGET, "Starting HTTP server at http://127.0.0.1:{}", cli.port);

        let cancel_token = self.cancel_token.clone();
        self.sub_tasks.push(tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(Self::shutdown_signal(cancel_token))
                .await?;
            Ok(())
        }));

        Ok(())
    }

    async fn shutdown_signal(cancel_token: CancellationToken) {
        cancel_token.cancelled().await
    }

    /// Stops the server and its services.
    pub async fn stop(self) -> Result<(), Error> {
        info!(target: LOG_TARGET, "Shutting down HTTP server...");
        self.cancel_token.cancel();
        for subtask in self.sub_tasks {
            subtask.await??;
        }

        Ok(())
    }
}
