use axum::{extract::State, http::StatusCode, Json};
use primitive_types::U256;
use serde::{Deserialize, Serialize};

use crate::{
    fee_estimator::{Estimator, Transaction},
    gas_price_collector::in_memory::InMemoryCollector,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct EstimateResponse {
    estimated_fee_wei: U256,
    error: Option<String>,
}

impl EstimateResponse {
    pub fn success(estimated_fee_wei: U256) -> Self {
        Self {
            estimated_fee_wei,
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            estimated_fee_wei: U256::zero(),
            error: Some(error),
        }
    }
}

/// Handler for gas fee estimation endpoint.
pub async fn handler(
    State(estimator): State<Estimator<InMemoryCollector>>,
    Json(transaction): Json<Transaction>,
) -> (StatusCode, Json<EstimateResponse>) {
    (
        StatusCode::OK,
        Json(match estimator.estimate(transaction).await {
            Ok(fee) => EstimateResponse::success(fee),
            Err(error) => EstimateResponse::error(error.to_string()),
        }),
    )
}
