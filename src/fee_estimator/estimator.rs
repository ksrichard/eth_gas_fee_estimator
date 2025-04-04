use std::sync::Arc;

use ethereum::{AccessList, TransactionAction};
use primitive_types::U256;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::gas_price_collector::Collector;

use super::{eip1559, eip2930, gas_used_estimator, legacy};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Input estimator error: {0}")]
    InputEstimator(#[from] gas_used_estimator::Error),
    #[error("Max fee / gas is too low: {current}, calculated: {calculated}")]
    MaxFeePerGasTooLow { current: U256, calculated: U256 },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Transaction {
    /// Legacy transaction type
    Legacy(LegacyTransaction),
    /// EIP-2930 transaction
    EIP2930(EIP2930Transaction),
    /// EIP-1559 transaction
    EIP1559(EIP1559Transaction),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LegacyTransaction {
    pub gas_price: U256,
    pub gas_limit: U256,
    pub input: String,
    pub action: TransactionAction,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EIP2930Transaction {
    pub gas_price: U256,
    pub gas_limit: U256,
    pub input: String,
    pub action: TransactionAction,
    pub access_list: AccessList,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EIP1559Transaction {
    pub max_priority_fee_per_gas: U256,
    pub max_fee_per_gas: U256,
    pub gas_limit: U256,
    pub input: String,
    pub action: TransactionAction,
    pub access_list: AccessList,
}

/// The main gas cost estimator, it can estimate Legacy, EIP-2930 and EIP-1559 transactions.
#[derive(Clone)]
pub struct Estimator<C: Collector + Clone> {
    gas_price_collector: Arc<C>,
}

impl<C: Collector + Clone> Estimator<C> {
    pub fn new(gas_price_collector: Arc<C>) -> Self {
        Self {
            gas_price_collector,
        }
    }

    /// Estimates the cost of the given transaction in WEI.
    pub async fn estimate(&self, transaction: Transaction) -> Result<U256, Error> {
        match transaction {
            Transaction::Legacy(tx) => legacy::LegacyTransactionEstimator.estimate(tx),
            Transaction::EIP2930(tx) => eip2930::EIP2930TransactionEstimator.estimate(tx),
            Transaction::EIP1559(tx) => {
                eip1559::EIP1559TransactionEstimator::new(self.gas_price_collector.clone())
                    .estimate(tx)
                    .await
            }
        }
    }
}
