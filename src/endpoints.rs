use crate::types::*;
use beacon_api_client::{BlockId, Client, PublicKeyOrIndex, StateId, ValidatorStatus};
use ethereum_consensus::phase0::mainnet::SignedBeaconBlock;
use eyre::Result;
use tokio::time::Instant;
use tracing::{info, warn};
use url::Url;

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

    for u in urls.iter() {
        // TODO: Share the clients instead.
        let client = Client::new(u.clone());
        let block = client.get_beacon_block(BlockId::Head).await?;
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
        "Got equal /eth/v2/beacon/blocks responses across all {} endpoints",
        urls.len(),
    );
    Ok(())
}

pub async fn check_validators(urls: Vec<Url>) -> Result<()> {
    let mut responses: Vec<Vec<ValidatorSummaryExt>> = vec![];

    for u in urls.iter() {
        // TODO: Share the clients instead.
        let client = Client::new(u.clone());

        // TODO: Randomly generate and test different kinds of state ids.
        let indices: Vec<PublicKeyOrIndex> = vec![
            PublicKeyOrIndex::from(10912),
            PublicKeyOrIndex::from(0),
            PublicKeyOrIndex::from(400000),
        ];
        let filters: Vec<ValidatorStatus> = vec![];
        let start = Instant::now();
        let mut validators = client
            .get_validators(StateId::Head, &indices, &filters)
            .await?;
        info!(
            "Validators response took {} milliseconds",
            start.elapsed().as_millis()
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
        "Got equal /eth/v1/beacon/validators responses across all {} endpoints",
        urls.len(),
    );
    Ok(())
}

pub async fn check_balances(urls: Vec<Url>) -> Result<()> {
    let mut responses: Vec<Vec<BalanceSummaryExt>> = vec![];

    for u in urls.iter() {
        // TODO: Share the clients instead.
        let client = Client::new(u.clone());

        // TODO: Randomly generate and test different kinds of state ids.
        let indices: Vec<PublicKeyOrIndex> = vec![
            PublicKeyOrIndex::from(10912),
            PublicKeyOrIndex::from(0),
            PublicKeyOrIndex::from(400000),
        ];
        let mut balances = client.get_balances(StateId::Head, &indices).await?;
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
        "Got equal /eth/v1/beacon/balances responses across all {} endpoints",
        urls.len(),
    );
    Ok(())
}
