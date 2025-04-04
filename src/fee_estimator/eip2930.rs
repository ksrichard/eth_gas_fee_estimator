use evm_runtime::Config;
use primitive_types::U256;

use super::{gas_used_estimator::GasUsedEstimator, EIP2930Transaction, Error, Transaction};

pub struct EIP2930TransactionEstimator;

impl EIP2930TransactionEstimator {
    pub fn estimate(&self, transaction: EIP2930Transaction) -> Result<U256, Error> {
        let estimator = GasUsedEstimator::new(Config::london(), transaction.gas_limit.as_u64());
        let gas_price = transaction
            .gas_price
            .saturating_mul(U256::from(1_000_000_000));
        let gas_used = estimator.estimate(Transaction::EIP2930(transaction))?;
        Ok(gas_price.saturating_mul(gas_used.into()))
    }
}
