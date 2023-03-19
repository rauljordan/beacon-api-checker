use crate::types::*;
use beacon_api_client::{BlockId, Client, PublicKeyOrIndex, StateId, ValidatorStatus};
use ethereum_consensus::{clock, phase0::mainnet::SignedBeaconBlock, primitives::ValidatorIndex};
use eyre::Result;
use human_duration::human_duration;
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use tokio::time::Instant;
use tracing::{info, warn};
use url::Url;

// TODO: Dump the mismatched request if a mismatch happens to debug logs.

pub async fn check_finality_checkpoints(urls: Vec<Url>) -> Result<()> {
    Ok(())
}

pub async fn check_state_roots(urls: Vec<Url>) -> Result<()> {
    Ok(())
}

pub async fn check_state_fork(urls: Vec<Url>) -> Result<()> {
    Ok(())
}

pub async fn check_block_header(urls: Vec<Url>) -> Result<()> {
    Ok(())
}

pub async fn check_block(urls: Vec<Url>) -> Result<()> {
    let mut responses: Vec<SignedBeaconBlock> = vec![];

    // TODO: Capture median latency.
    //
    let id = random_block_id();
    for u in urls.iter() {
        // TODO: Share the clients instead.
        let client = Client::new(u.clone());
        let start = Instant::now();
        info!(
            "Calling /eth/v1/beacon/{}/block endpoint={}",
            id.clone().inner,
            u,
        );
        let block = client.get_beacon_block(id.clone().inner).await?;
        info!(
            "/eth/v1/beacon/{}/block response_time={}, endpoint={}",
            id.clone().inner,
            human_duration(&start.elapsed()),
            u,
        );
        responses.push(block);
    }

    for (i, v1) in responses.iter().enumerate() {
        for (j, v2) in responses[i..].iter().enumerate() {
            if v1 != v2 {
                let e1 = urls.get(i).unwrap();
                let e2 = urls.get(j).unwrap();
                warn!(
                    "Urls {} and {} got mismatched /eth/v2/beacon/block responses",
                    e1, e2
                );
                crate::metrics::BLOCK_NOT_EQUAL_TOTAL.inc();
                return Ok(());
            }
        }
    }
    info!(
        "Got equal /eth/v2/beacon/blocks across {} endpoints",
        urls.len(),
    );
    Ok(())
}

pub async fn check_validators(urls: Vec<Url>) -> Result<()> {
    let mut responses: Vec<Vec<ValidatorSummaryExt>> = vec![];

    let indices = random_validator_indices();
    let id = random_state_id();
    let filters: Vec<ValidatorStatus> = vec![];
    for u in urls.iter() {
        // TODO: Share the clients instead.
        let client = Client::new(u.clone());

        let start = Instant::now();
        info!(
            "Calling /eth/v1/beacon/{}/validators endpoint={}, num_indices={}",
            id.clone().inner,
            u,
            indices.len(),
        );
        let mut validators = client
            .get_validators(id.clone().inner, &indices, &filters)
            .await?;
        info!(
            "/eth/v1/beacon/{}/validators response_time={}, endpoint={}, num_indices={}",
            id.clone().inner,
            human_duration(&start.elapsed()),
            u,
            indices.len(),
        );
        // Sort by validator index.
        validators.sort_by(|a, b| a.index.cmp(&b.index));
        let ext = validators
            .into_iter()
            .map(|v| ValidatorSummaryExt { inner: v })
            .collect();
        responses.push(ext);
    }

    for (i, v1) in responses.iter().enumerate() {
        for (j, v2) in responses[i..].iter().enumerate() {
            if v1 != v2 {
                let e1 = urls.get(i).unwrap();
                let e2 = urls.get(j).unwrap();
                warn!("Urls {} and {} got mismatched validators responses", e1, e2);
                crate::metrics::VALIDATORS_NOT_EQUAL_TOTAL.inc();
                return Ok(());
            }
        }
    }
    info!(
        "Got equal /eth/v1/beacon/validators across {} endpoints",
        urls.len(),
    );
    Ok(())
}

pub async fn check_balances(urls: Vec<Url>) -> Result<()> {
    let mut responses: Vec<Vec<BalanceSummaryExt>> = vec![];

    let indices = random_validator_indices();
    let id = random_state_id();
    for u in urls.iter() {
        // TODO: Share the clients instead.
        let client = Client::new(u.clone());
        let start = Instant::now();
        info!(
            "Calling /eth/v1/beacon/{}/balances endpoint={}, num_indices={}",
            id.clone().inner,
            u,
            indices.len(),
        );
        let mut balances = client.get_balances(id.clone().inner, &indices).await?;
        info!(
            "/eth/v1/beacon/{}/validators response_time={}, endpoint={}, num_indices={}",
            id.clone().inner,
            human_duration(&start.elapsed()),
            u,
            indices.len(),
        );
        // Sort by validator index.
        balances.sort_by(|a, b| a.index.cmp(&b.index));
        let ext = balances
            .into_iter()
            .map(|v| BalanceSummaryExt { inner: v })
            .collect();
        responses.push(ext);
    }

    for (i, v1) in responses.iter().enumerate() {
        for (j, v2) in responses[i..].iter().enumerate() {
            if v1 != v2 {
                let e1 = urls.get(i).unwrap();
                let e2 = urls.get(j).unwrap();
                warn!("Urls {} and {} got mismatched balances responses", e1, e2);
                crate::metrics::BALANCES_NOT_EQUAL_TOTAL.inc();
                return Ok(());
            }
        }
    }
    info!(
        "Got equal /eth/v1/beacon/balances across {} endpoints",
        urls.len(),
    );
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
