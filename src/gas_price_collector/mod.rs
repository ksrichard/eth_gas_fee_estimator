use async_trait::async_trait;
use primitive_types::U256;
use tokio_util::sync::CancellationToken;

pub mod in_memory;

/// The trait that all gas price collector must implement.
/// It gives the chance to let the implementation handle how it gets the current gas price (through API or from an Ethereum node etc...).
#[async_trait]
pub trait Collector {
    type Error;

    /// Starts the collector.
    async fn start(&self, cancel_token: CancellationToken) -> Result<(), Self::Error>;

    // Returns actual gas price in WEI.
    async fn gas_price(&self) -> U256;
}
