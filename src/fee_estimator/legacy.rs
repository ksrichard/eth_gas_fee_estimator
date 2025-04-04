use evm_runtime::Config;
use primitive_types::U256;

use super::{gas_used_estimator::GasUsedEstimator, Error, LegacyTransaction, Transaction};

pub struct LegacyTransactionEstimator;

impl LegacyTransactionEstimator {
    pub fn estimate(&self, transaction: LegacyTransaction) -> Result<U256, Error> {
        let estimator = GasUsedEstimator::new(Config::london(), transaction.gas_limit.as_u64());
        let gas_price = transaction
            .gas_price
            .saturating_mul(U256::from(1_000_000_000));
        let gas_used = estimator.estimate(Transaction::Legacy(transaction))?;
        Ok(gas_price.saturating_mul(gas_used.into()))
    }
}
