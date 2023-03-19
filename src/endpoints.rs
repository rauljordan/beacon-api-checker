use beacon_api_client::{Client, PublicKeyOrIndex, StateId, ValidatorStatus};
use eyre::Result;
use tracing::info;
use url::Url;

// TODO: Need to compare endpoints.
// TODO: Attestations, blocks, block headers, state root, sync committees, committees, checkpoint
pub async fn check_validators(u: Url) -> Result<()> {
    let client = Client::new(u);
    let indices: Vec<PublicKeyOrIndex> = vec![PublicKeyOrIndex::from(32)];
    let filters: Vec<ValidatorStatus> = vec![];
    let resp = client
        .get_validators(StateId::Head, &indices, &filters)
        .await?;
    info!("Got validators result {:?}", resp);
    crate::metrics::VALIDATORS_NOT_EQUAL_TOTAL.inc();
    Ok(())
}

pub async fn check_balances(u: Url) -> Result<()> {
    let client = Client::new(u);
    let indices: Vec<PublicKeyOrIndex> = vec![PublicKeyOrIndex::from(32)];
    let resp = client.get_balances(StateId::Head, &indices).await?;
    let balances: Vec<u64> = resp.into_iter().map(|b| b.balance as u64).collect();
    info!("Got balances result {:?}", balances);
    crate::metrics::VALIDATORS_NOT_EQUAL_TOTAL.inc();
    Ok(())
}

pub async fn check_blocks(u: Url) -> Result<()> {
    let client = Client::new(u);
    let indices: Vec<PublicKeyOrIndex> = vec![PublicKeyOrIndex::from(32)];
    let resp = client.get_balances(StateId::Head, &indices).await?;
    let balances: Vec<u64> = resp.into_iter().map(|b| b.balance as u64).collect();
    info!("Got balances result {:?}", balances);
    crate::metrics::VALIDATORS_NOT_EQUAL_TOTAL.inc();
    Ok(())
}
