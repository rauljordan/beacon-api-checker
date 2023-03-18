use axum::{routing::get, Router};
use beacon_api_client::{Client, PublicKeyOrIndex, StateId, ValidatorStatus};
use clap::Parser;
use eyre::Report;
use futures::future::join_all;
use std::future::Future;
use std::pin::Pin;
use tokio::time::{interval, Duration};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use url::Url;

mod metrics;

#[derive(Parser, Debug)]
#[command(name = "api-checker")]
#[command(author = "rauljordan")]
#[command(version = "0.0.1")]
#[command(
    about = "CLI tool for cross-checking beacon API responses across clients",
    long_about = None,
)]
struct Cli {
    #[arg(long)]
    beacon_api_endpoints: Vec<String>,
    #[arg(default_value = "127.0.0.1")]
    metrics_host: String,
    #[arg(default_value_t = 8080)]
    metrics_port: u32,
}

type ToolResult = Result<(), Report>;
type AsyncResult = Pin<Box<dyn Future<Output = ToolResult> + Send + Sync>>;
type ApiChecker = Box<dyn Fn(Url) -> AsyncResult + Send + Sync>;

fn force_boxed<T>(f: fn(Url) -> T) -> ApiChecker
where
    T: Future<Output = ToolResult> + 'static + Send + Sync,
{
    Box::new(move |n| Box::pin(f(n)))
}

// TODO: Use prom metrics, keep track of different failures
// Try max size, try old epoch requests, try all kinds of state id checks.
// Customize number of parallel requests per client via tokio threads.
// Join handles to check completion.
// Single binary.
// Docker.
#[tokio::main]
async fn main() -> eyre::Result<(), Report> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();
    let endpoints: Result<Vec<Url>, _> = cli
        .beacon_api_endpoints
        .into_iter()
        .map(|e| Url::parse(&e))
        .collect();
    let endpoints = endpoints.unwrap();
    info!("Starting task runner");

    let pipeline: Vec<ApiChecker> =
        vec![force_boxed(check_validators), force_boxed(check_balances)];
    let suite = TestSuiteBuilder::new(endpoints)
        .timeout(Duration::from_secs(10))
        .pipeline(pipeline)
        .build();

    let mut handles = vec![];
    handles.push(tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(10));
        loop {
            ticker.tick().await;
            info!("Running API checker pipeline");
            suite.run_pipeline().await.unwrap();
        }
    }));

    info!("Starting metrics server");

    let app = Router::new().route("/metrics", get(crate::metrics::handler));
    let addr = format!("{}:{}", cli.metrics_host, cli.metrics_port);
    let server = axum::Server::bind(&addr.parse().unwrap()).serve(app.into_make_service());
    handles.push(tokio::spawn(server));

    join_all(handles).await;
    Ok(())
}

// TODO: Need to compare endpoints.
// TODO: Attestations, blocks, block headers, state root, sync committees, committees, checkpoint
pub async fn check_validators(u: Url) -> ToolResult {
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

pub async fn check_balances(u: Url) -> Result<(), Report> {
    let client = Client::new(u);
    let indices: Vec<PublicKeyOrIndex> = vec![PublicKeyOrIndex::from(32)];
    let resp = client.get_balances(StateId::Head, &indices).await?;
    let balances: Vec<u64> = resp.into_iter().map(|b| b.balance as u64).collect();
    info!("Got balances result {:?}", balances);
    crate::metrics::VALIDATORS_NOT_EQUAL_TOTAL.inc();
    Ok(())
}

pub async fn check_blocks(u: Url) -> Result<(), Report> {
    let client = Client::new(u);
    let indices: Vec<PublicKeyOrIndex> = vec![PublicKeyOrIndex::from(32)];
    let resp = client.get_balances(StateId::Head, &indices).await?;
    let balances: Vec<u64> = resp.into_iter().map(|b| b.balance as u64).collect();
    info!("Got balances result {:?}", balances);
    crate::metrics::VALIDATORS_NOT_EQUAL_TOTAL.inc();
    Ok(())
}

#[derive(Default)]
pub struct TestSuiteBuilder {
    pub endpoints: Vec<Url>,
    timeout: Duration,
    pub fns: Vec<ApiChecker>,
}

impl TestSuiteBuilder {
    pub fn new(endpoints: Vec<Url>) -> TestSuiteBuilder {
        TestSuiteBuilder {
            endpoints,
            fns: vec![],
            timeout: Duration::from_secs(10),
        }
    }
    pub fn timeout(mut self, timeout: Duration) -> TestSuiteBuilder {
        self.timeout = timeout;
        self
    }
    pub fn pipeline(mut self, fns: Vec<ApiChecker>) -> TestSuiteBuilder {
        self.fns = fns;
        self
    }
    pub fn build(self) -> TestSuiteBuilder {
        self
    }
    pub async fn run_pipeline(&self) -> ToolResult {
        for f in self.fns.iter() {
            let url = Url::parse("http://localhost:3500").unwrap();
            f(url).await?;
        }
        Ok(())
    }
}
