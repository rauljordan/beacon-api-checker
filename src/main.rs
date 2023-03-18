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

// TODO: Use prom metrics, keep track of different failures
// Try max size, try old epoch requests, try all kinds of state id checks.
// Customize number of parallel requests per client via tokio threads.
// Join handles to check completion.
// Single binary.
// Docker.
#[tokio::main]
async fn main() -> eyre::Result<(), Report> {
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cli = Cli::parse();
    let endpoints: Result<Vec<Url>, _> = cli
        .beacon_api_endpoints
        .into_iter()
        .map(|e| Url::parse(&e))
        .collect();
    let endpoints = endpoints.unwrap();

    // let pipeline: Vec<CheckerFn> = vec![Box::new(check_validators_async)];
    // let _suite = TestSuiteBuilder::new(endpoints)
    //     .run_pipeline(pipeline)
    //     .await?;
    _ = endpoints;

    let mut handles = vec![];
    handles.push(tokio::spawn(async {
        info!("Starting task runner");
        let mut ticker = interval(Duration::from_secs(10));
        loop {
            ticker.tick().await;
            info!("Running tasks");
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

pub async fn check_validators(u: Url) -> ToolResult {
    let client = Client::new(u);
    let indices: Vec<PublicKeyOrIndex> = vec![PublicKeyOrIndex::from(32)];
    let filters: Vec<ValidatorStatus> = vec![];
    let resp = client
        .get_validators(StateId::Head, &indices, &filters)
        .await?;
    println!("{:?}", resp);
    crate::metrics::VALIDATORS_NOT_EQUAL_TOTAL.inc();
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
    crate::metrics::VALIDATORS_NOT_EQUAL_TOTAL.inc();
    Ok(())
}

fn check_balances_async(u: Url) -> Pin<Box<dyn Future<Output = ToolResult>>> {
    Box::pin(check_balances(u))
}
