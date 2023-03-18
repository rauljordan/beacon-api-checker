use beacon_api_client::{Client, PublicKeyOrIndex, StateId, ValidatorStatus};
use clap::Parser;
use eyre::Report;
use std::future::Future;
use std::pin::Pin;
use url::Url;

#[derive(Parser, Debug)]
#[command(name = "api-checker")]
#[command(author = "rauljordan")]
#[command(version = "0.0.1")]
#[command(
    about = "CLI tool for cross-checking beacon API responses across clients",
    long_about = None,
)]
struct Cli {
    #[arg(short, long)]
    beacon_api_endpoints: Vec<String>,
}

type ToolResult = Result<(), Report>;
type AsyncResult = Pin<Box<dyn Future<Output = ToolResult>>>;
type CheckerFn = Box<dyn Fn(Url) -> AsyncResult>;

#[derive(Default)]
pub struct TestSuiteBuilder {
    endpoints: Vec<Url>,
}

impl TestSuiteBuilder {
    pub fn new(endpoints: Vec<Url>) -> TestSuiteBuilder {
        TestSuiteBuilder { endpoints }
    }
    pub async fn run_pipeline(self, checkers: Vec<CheckerFn>) -> ToolResult {
        for endpoint in self.endpoints.iter() {
            for f in checkers.iter() {
                f(endpoint.clone()).await?;
            }
        }
        Ok(())
    }
}

macro_rules! boxed_vec{
    (($name: tt),*) => {
        vec![Box::new($name),*]
    };
}

// TODO: Use prom metrics, keep track of different failures
// Try max size, try old epoch requests, try all kinds of state id checks.
// Customize number of parallel requests per client via tokio threads.
// Join handles to check completion.
// Single binary.
// Docker.
#[tokio::main]
async fn main() -> eyre::Result<(), Report> {
    let cli = Cli::parse();
    let endpoints: Result<Vec<Url>, _> = cli
        .beacon_api_endpoints
        .into_iter()
        .map(|e| Url::parse(&e))
        .collect();
    let endpoints = endpoints.unwrap();

    let pipeline: Vec<CheckerFn> = boxed_vec!(check_validators_async);
    let _suite = TestSuiteBuilder::new(endpoints)
        .run_pipeline(pipeline)
        .await?;
    Ok(())
}

pub async fn check_validators(u: Url) -> ToolResult {
    let client = Client::new(u);
    let indices: Vec<PublicKeyOrIndex> = vec![PublicKeyOrIndex::from(32)];
    let filters: Vec<ValidatorStatus> = vec![];
    let resp = client
        .get_validators(StateId::Head, &indices, &filters)
        .await?;
    println!("{:?}", resp);
    Ok(())
}

fn check_validators_async(u: Url) -> Pin<Box<dyn Future<Output = ToolResult>>> {
    Box::pin(check_validators(u))
}

pub async fn check_balances(u: Url) -> Result<(), Report> {
    let client = Client::new(u);
    let indices: Vec<PublicKeyOrIndex> = vec![PublicKeyOrIndex::from(32)];
    let filters: Vec<ValidatorStatus> = vec![];
    let resp = client.get_balances(StateId::Head, &indices).await?;
    println!("{:?}", resp.len());
    Ok(())
}

fn check_balances_async(u: Url) -> Pin<Box<dyn Future<Output = ToolResult>>> {
    Box::pin(check_balances(u))
}
