use axum::http::StatusCode;
use prometheus::{self, IntCounter, TextEncoder};

use lazy_static::lazy_static;
use prometheus::register_int_counter;

lazy_static! {
    pub static ref VALIDATORS_NOT_EQUAL_TOTAL: IntCounter = register_int_counter!(
        "api_checker_get_validators_unequal_total",
        "Mismatched get_validators responses",
    )
    .unwrap();
    pub static ref BALANCES_NOT_EQUAL_TOTAL: IntCounter = register_int_counter!(
        "api_checker_get_balances_unequal_total",
        "Mismatched get_balances_responses",
    )
    .unwrap();
    pub static ref BLOCKS_NOT_EQUAL_TOTAL: IntCounter = register_int_counter!(
        "api_checker_get_blocks_unequal_total",
        "Mismatched get_blocks_responses",
    )
    .unwrap();
    pub static ref ATTESTATIONS_NOT_EQUAL_TOTAL: IntCounter = register_int_counter!(
        "api_checker_get_attestations_unequal_total",
        "Mismatched get_attestations_responses",
    )
    .unwrap();
}

pub async fn handler() -> Result<String, StatusCode> {
    let encoder = TextEncoder::new();
    match encoder.encode_to_string(&prometheus::gather()) {
        Ok(s) => Ok(s),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
