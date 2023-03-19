use crate::types::*;
use beacon_api_client::{BlockId, Client, PublicKeyOrIndex, StateId, ValidatorStatus};
use ethereum_consensus::{clock, phase0::mainnet::SignedBeaconBlock, primitives::ValidatorIndex};
use eyre::Result;
use human_duration::human_duration;
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use std::time::Duration;
use tokio::time::Instant;
use tracing::{info, warn};
use url::Url;

// pub async fn check_finality_checkpoints(urls: Vec<Url>) -> Result<()> {
//     Ok(())
// }

// pub async fn check_state_roots(urls: Vec<Url>) -> Result<()> {
//     Ok(())
// }

// pub async fn check_state_fork(urls: Vec<Url>) -> Result<()> {
//     Ok(())
// }

// pub async fn check_block_header(urls: Vec<Url>) -> Result<()> {
//     Ok(())
// }

pub async fn check_block(urls: Vec<Url>) -> Result<()> {
    let mut responses: Vec<SignedBeaconBlock> = vec![];
    let mut latencies = vec![];

    let id = random_block_id();
    let method = format!("/eth/v2/beacon/{}/block", id.inner);
    for u in urls.iter() {
        // TODO: Share the clients instead.
        let client = Client::new(u.clone());
        let start = Instant::now();
        info!("Calling {} endpoint={}", method, u);
        let block = client.get_beacon_block(id.clone().inner).await?;
        latencies.push(start.elapsed().as_millis() as u64);
        responses.push(block);
    }
    let median_latency = Duration::from_millis(median(&mut latencies));
    info!(
        "{} median_response_time={}",
        method,
        human_duration(&median_latency),
    );
    crate::metrics::GET_BLOCK_LATENCY_MILLISECONDS.observe(median_latency.as_millis() as f64);

    if mismatched_responses(&method, &urls, responses) {
        crate::metrics::BLOCK_NOT_EQUAL_TOTAL.inc();
        warn!("MISMATCHED REQUEST: endpoint={}", method);
    }
    Ok(())
}

pub async fn check_validators(urls: Vec<Url>) -> Result<()> {
    let mut responses: Vec<Vec<ValidatorSummaryExt>> = vec![];
    let mut latencies = vec![];

    let indices = random_validator_indices();
    let id = random_state_id();
    let method = format!("/eth/v2/beacon/{}/validators", id.inner);
    let filters: Vec<ValidatorStatus> = vec![];

    for u in urls.iter() {
        // TODO: Share the clients instead.
        let client = Client::new(u.clone());

        let start = Instant::now();
        info!(
            "Calling {} endpoint={}, num_indices={}",
            method,
            u,
            indices.len(),
        );
        let mut validators = client
            .get_validators(id.clone().inner, &indices, &filters)
            .await?;
        latencies.push(start.elapsed().as_millis() as u64);

        // Sort by validator index.
        validators.sort_by(|a, b| a.index.cmp(&b.index));
        let ext = validators
            .into_iter()
            .map(|v| ValidatorSummaryExt { inner: v })
            .collect();
        responses.push(ext);
    }
    let median_latency = Duration::from_millis(median(&mut latencies));
    info!(
        "{} median_response_time={}",
        method,
        human_duration(&median_latency),
    );
    crate::metrics::GET_VALIDATORS_LATENCY_MILLISECONDS.observe(median_latency.as_millis() as f64);

    if mismatched_responses(&method, &urls, responses) {
        crate::metrics::VALIDATORS_NOT_EQUAL_TOTAL.inc();
        warn!(
            "MISMATCHED REQUEST: endpoint={}, indices={:?}",
            method, indices
        );
    }
    Ok(())
}

pub async fn check_balances(urls: Vec<Url>) -> Result<()> {
    let mut responses: Vec<Vec<BalanceSummaryExt>> = vec![];
    let mut latencies = vec![];

    let indices = random_validator_indices();
    let id = random_state_id();
    let method = format!("/eth/v1/beacon/{}/balances", id.inner);
    for u in urls.iter() {
        // TODO: Share the clients instead.
        let client = Client::new(u.clone());
        let start = Instant::now();
        info!(
            "Calling {} endpoint={}, num_indices={}",
            method,
            u,
            indices.len(),
        );
        let mut balances = client.get_balances(id.clone().inner, &indices).await?;
        latencies.push(start.elapsed().as_millis() as u64);
        // Sort by validator index.
        balances.sort_by(|a, b| a.index.cmp(&b.index));
        let ext = balances
            .into_iter()
            .map(|v| BalanceSummaryExt { inner: v })
            .collect();
        responses.push(ext);
    }
    let median_latency = Duration::from_millis(median(&mut latencies));
    info!(
        "{} median_response_time={}, num_indices={}",
        method,
        human_duration(&median_latency),
        indices.len(),
    );
    crate::metrics::GET_BALANCES_LATENCY_MILLISECONDS.observe(median_latency.as_millis() as f64);

    if mismatched_responses(&method, &urls, responses) {
        crate::metrics::BALANCES_NOT_EQUAL_TOTAL.inc();
        warn!(
            "MISMATCHED REQUEST: endpoint={}, indices={:?}",
            method, indices
        );
    }
    Ok(())
}

// Random slot in the last 100 slots on mainnet.
fn random_slot(rng: &mut ThreadRng) -> u64 {
    let mainnet_clock = clock::for_mainnet();
    let curr = mainnet_clock.current_slot();
    rng.gen_range(curr - 100..curr)
}

fn random_state_id() -> StateIdExt {
    let mut ids = vec![StateId::Finalized, StateId::Justified, StateId::Head];
    let mut rng = rand::thread_rng();
    let slot: u64 = random_slot(&mut rng);
    ids.push(StateId::Slot(slot));
    match ids.choose(&mut rng).unwrap() {
        &StateId::Finalized => StateIdExt {
            inner: StateId::Finalized,
        },
        &StateId::Justified => StateIdExt {
            inner: StateId::Justified,
        },
        &StateId::Head => StateIdExt {
            inner: StateId::Head,
        },
        &StateId::Slot(x) => StateIdExt {
            inner: StateId::Slot(x),
        },
        _ => unreachable!(),
    }
}

fn random_validator_indices() -> Vec<PublicKeyOrIndex> {
    let mut indices: Vec<PublicKeyOrIndex> = vec![];
    let mut rng = rand::thread_rng();
    let num_elems: u64 = rng.gen_range(0..100);
    for _ in 0..num_elems {
        let idx: usize = rng.gen_range(0..500_000);
        indices.push(PublicKeyOrIndex::from(ValidatorIndex::from(idx)));
    }
    indices
}

fn random_block_id() -> BlockIdExt {
    let mut ids = vec![BlockId::Finalized, BlockId::Head];
    let mut rng = rand::thread_rng();
    let slot: u64 = random_slot(&mut rng);
    ids.push(BlockId::Slot(slot));
    match ids.choose(&mut rng).unwrap() {
        &BlockId::Genesis => BlockIdExt {
            inner: BlockId::Genesis,
        },
        &BlockId::Finalized => BlockIdExt {
            inner: BlockId::Finalized,
        },
        &BlockId::Head => BlockIdExt {
            inner: BlockId::Head,
        },
        &BlockId::Slot(x) => BlockIdExt {
            inner: BlockId::Slot(x),
        },
        _ => unreachable!(),
    }
}

pub fn mismatched_responses<T: Eq>(method: &str, endpoints: &[Url], v: Vec<T>) -> bool {
    for (i, v1) in v.iter().enumerate() {
        for (j, v2) in v[i..].iter().enumerate() {
            if v1 != v2 {
                let e1 = endpoints.get(i).unwrap();
                let e2 = endpoints.get(j).unwrap();
                warn!("Urls {} and {} got mismatched {} responses", method, e1, e2);
                return true;
            }
        }
    }
    info!("Got equal {} across {} endpoints", method, endpoints.len());
    false
}

fn median(latencies: &mut Vec<u64>) -> u64 {
    latencies.sort();
    if (latencies.len() % 2) == 0 {
        let ind_left = latencies.len() / 2 - 1;
        let ind_right = latencies.len() / 2;
        (latencies[ind_left] + latencies[ind_right]) / 2
    } else {
        latencies[(latencies.len() / 2)]
    }
}
