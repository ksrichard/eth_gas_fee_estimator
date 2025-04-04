use std::{sync::Arc, time::Duration};

use alloy::{
    rpc::client::RpcClient,
    transports::{RpcError, TransportErrorKind},
};
use async_trait::async_trait;
use log::{error, info};
use primitive_types::U256;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use super::Collector;

const LOG_TARGET: &str = "gas_price_collector::in_memory";

/// In memory gas price collector that fetches gas price through an ethereum JSON RPC call.
/// The current gas price is stored in memory only.
#[derive(Clone)]
pub struct InMemoryCollector {
    eth_client: RpcClient,
    gas_price: Arc<RwLock<U256>>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Ethereum client JSON-RPC transport error: {0}")]
    RpcTransport(#[from] RpcError<TransportErrorKind>),
}

impl InMemoryCollector {
    pub fn new(eth_rpc_client_url: url::Url) -> Self {
        let eth_client = alloy::rpc::client::ClientBuilder::default().http(eth_rpc_client_url);
        Self {
            eth_client,
            gas_price: Arc::new(RwLock::new(U256::zero())),
        }
    }

    pub async fn update_gas_price(&self) -> Result<(), Error> {
        let current_gas_price_wei: U256 = self.eth_client.request_noparams("eth_gasPrice").await?;
        let mut gas_price_lock = self.gas_price.write().await;
        *gas_price_lock = current_gas_price_wei;
        info!(target: LOG_TARGET, "Current gas price: {} wei", *gas_price_lock);

        Ok(())
    }
}

#[async_trait]
impl Collector for InMemoryCollector {
    type Error = Error;

    async fn start(&self, cancel_token: CancellationToken) -> Result<(), Self::Error> {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(error) = self.update_gas_price().await {
                        error!(target: LOG_TARGET, "Failed to update gas price: {error:?}");
                    }
                }
                _ = cancel_token.cancelled() => {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn gas_price(&self) -> U256 {
        let gas_price_lock = self.gas_price.read().await;
        *gas_price_lock
    }
}
