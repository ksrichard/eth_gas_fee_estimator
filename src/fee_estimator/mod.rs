use lazy_static::lazy_static;

pub mod eip1559;
pub mod eip2930;
mod estimator;
pub mod gas_used_estimator;
pub mod legacy;
pub use estimator::*;

lazy_static! {
    pub static ref BASE_GAS_COUNT: primitive_types::U256 = primitive_types::U256::from(21_000);
    pub static ref CONTRACT_CREATION_GAS: primitive_types::U256 =
        primitive_types::U256::from(32_000);
}
