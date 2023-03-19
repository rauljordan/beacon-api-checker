use std::iter::zip;

use beacon_api_client::{
    BalanceSummary, Client, PublicKeyOrIndex, StateId, ValidatorStatus, ValidatorSummary,
};
use eyre::Result;
use tracing::{info, warn};
use url::Url;

// TODO: Need to compare endpoints.
// TODO: Attestations, blocks, block headers, state root, sync committees, committees, checkpoint
pub async fn check_validators(urls: Vec<Url>) -> Result<()> {
    let mut responses: Vec<Vec<ValidatorSummary>> = vec![];

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
        let mut validators = client
            .get_validators(StateId::Head, &indices, &filters)
            .await?;
        // Sort by validator index.
        validators.sort_by(|a, b| a.index.cmp(&b.index));
        responses.push(validators);
    }

    for (i, v1) in responses.iter().enumerate() {
        for (j, v2) in responses[i..].iter().enumerate() {
            if !compare_validators_response(v1, v2) {
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

fn compare_validators_response(resp1: &[ValidatorSummary], resp2: &[ValidatorSummary]) -> bool {
    if resp1.len() != resp2.len() {
        return false;
    }
    for (v1, v2) in zip(resp1, resp2) {
        if v1.index != v2.index {
            return false;
        }
        if v1.balance != v2.balance {
            return false;
        }
        // TODO: Needs the partial eq trait defined.
        // if v1.status != v2.status {
        //     return false;
        // }
        if v1.validator.public_key != v2.validator.public_key {
            return false;
        }
        if v1.validator.slashed != v2.validator.slashed {
            return false;
        }
    }
    true
}

pub async fn check_balances(urls: Vec<Url>) -> Result<()> {
    let mut responses: Vec<Vec<BalanceSummary>> = vec![];

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
        responses.push(balances);
    }

    for (i, v1) in responses.iter().enumerate() {
        for (j, v2) in responses[i..].iter().enumerate() {
            if !compare_balances_response(v1, v2) {
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

fn compare_balances_response(resp1: &[BalanceSummary], resp2: &[BalanceSummary]) -> bool {
    if resp1.len() != resp2.len() {
        return false;
    }
    for (v1, v2) in zip(resp1, resp2) {
        if v1.index != v2.index {
            return false;
        }
        if v1.balance != v2.balance {
            return false;
        }
    }
    true
}
