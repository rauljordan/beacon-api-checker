use axum::http::StatusCode;
use prometheus::{self, Histogram, IntCounter, TextEncoder};

use lazy_static::lazy_static;
use prometheus::{register_histogram, register_int_counter};

lazy_static! {
    // Metrics for number of mismatched responses.
    pub static ref VALIDATORS_NOT_EQUAL_TOTAL: IntCounter = register_int_counter!(
        "api_checker_get_validators_unequal_total",
        "Mismatched get_validators responses",
    )
    .unwrap();
    pub static ref BALANCES_NOT_EQUAL_TOTAL: IntCounter = register_int_counter!(
        "api_checker_get_balances_unequal_total",
        "Mismatched get_balances responses",
    )
    .unwrap();
    pub static ref BLOCK_NOT_EQUAL_TOTAL: IntCounter = register_int_counter!(
        "api_checker_get_beacon_block_unequal_total",
        "Mismatched get_beacon_block responses",
    )
    .unwrap();

    // Latency metrics.
    pub static ref GET_VALIDATORS_LATENCY_MILLISECONDS: Histogram = register_histogram!(
        "api_checker_get_validators_latency_milliseconds",
        "Median latency of API responses for /eth/v1/beacon/validators in millis",
        vec![10.0, 50.0, 100.0, 150.0, 200.0, 300.0, 500.0, 1000.0, 5000.0, 10000.0, 30000.0, 60000.0],
    ).unwrap();
    pub static ref GET_BALANCES_LATENCY_MILLISECONDS: Histogram = register_histogram!(
        "api_checker_get_balances_latency_milliseconds",
        "Median latency of API responses for /eth/v1/beacon/balances in millis",
        vec![10.0, 50.0, 100.0, 150.0, 200.0, 300.0, 500.0, 1000.0, 5000.0, 10000.0, 30000.0, 60000.0],
    ).unwrap();
    pub static ref GET_BLOCK_LATENCY_MILLISECONDS: Histogram = register_histogram!(
        "api_checker_get_beacon_block_latency_milliseconds",
        "Median latency of API responses for /eth/v1/beacon/block in millis",
        vec![10.0, 50.0, 100.0, 150.0, 200.0, 300.0, 500.0, 1000.0, 5000.0, 10000.0, 30000.0, 60000.0],
    ).unwrap();
}

pub async fn handler() -> Result<String, StatusCode> {
    let encoder = TextEncoder::new();
    match encoder.encode_to_string(&prometheus::gather()) {
        Ok(s) => Ok(s),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
