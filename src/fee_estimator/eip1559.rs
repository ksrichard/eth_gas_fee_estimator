use std::sync::Arc;

use evm_runtime::Config;
use primitive_types::U256;

use crate::gas_price_collector::Collector;

use super::{gas_used_estimator::GasUsedEstimator, EIP1559Transaction, Error, Transaction};

pub struct EIP1559TransactionEstimator<C: Collector> {
    gas_price_collector: Arc<C>,
}

impl<C: Collector> EIP1559TransactionEstimator<C> {
    pub fn new(gas_price_collector: Arc<C>) -> Self {
        Self {
            gas_price_collector,
        }
    }

    pub async fn estimate(&self, transaction: EIP1559Transaction) -> Result<U256, Error> {
        let max_fee_per_gas = transaction
            .max_fee_per_gas
            .saturating_mul(U256::from(1_000_000_000));
        let estimator = GasUsedEstimator::new(Config::london(), transaction.gas_limit.as_u64());
        let gas_price = transaction
            .max_priority_fee_per_gas
            .saturating_mul(U256::from(1_000_000_000))
            .saturating_add(self.gas_price_collector.gas_price().await);

        if max_fee_per_gas.lt(&gas_price) {
            return Err(Error::MaxFeePerGasTooLow {
                current: max_fee_per_gas,
                calculated: gas_price,
            });
        }

        let gas_used = estimator.estimate(Transaction::EIP1559(transaction))?;
        let fee = gas_price.saturating_mul(gas_used.into());

        Ok(fee)
    }
}
