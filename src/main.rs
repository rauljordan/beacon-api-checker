use axum::{routing::get, Router};
use clap::Parser;
use eyre::Result;
use futures::future::join_all;
use tokio::time::{interval, Duration};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use url::Url;

mod api_checker;
mod endpoints;
mod metrics;
mod types;

use crate::api_checker::{force_boxed, ApiChecker, CheckerFn};
use crate::endpoints::{
    check_balances, check_block, check_finality_checkpoints, check_state_root, check_validators,
};

#[derive(Parser, Debug)]
#[command(name = "beacon-api-checker")]
#[command(author = "rauljordan")]
#[command(version = "0.0.1")]
#[command(
    about = "CLI tool for cross-checking beacon API responses across clients",
    long_about = None,
)]
struct Cli {
    #[arg(long)]
    endpoint: Vec<String>,
    #[arg(default_value = "127.0.0.1")]
    metrics_host: String,
    #[arg(default_value_t = 8080)]
    metrics_port: u32,
    #[arg(value_parser = parse_duration)]
    interval_seconds: Option<Duration>,
    #[arg(value_parser = parse_duration)]
    http_timeout: Option<Duration>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Sets up our logging to capture INFO objects to stdout.
    let subscriber = FmtSubscriber::builder()
        // TODO: Configurable verbosity.
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();

    // Parses the list of specified beacon API endpoints.
    let endpoints: Result<Vec<Url>, _> = cli.endpoint.into_iter().map(|e| Url::parse(&e)).collect();

    // Defines a pipeline of functions to run through our API checker.
    // Each function will call a respective API endpoint across all specified
    // beacon node URLs and cross-check their responses.
    // At this time, the pipeline is executed sequentially.
    let pipeline: Vec<CheckerFn> = vec![
        force_boxed(check_validators),
        force_boxed(check_balances),
        force_boxed(check_block),
        force_boxed(check_finality_checkpoints),
        force_boxed(check_state_root),
    ];

    // Builds an API checker from our specified CLI flags
    // and the pipeline defined above.
    let mut api_checker = ApiChecker::new()
        .endpoints(endpoints.unwrap())
        .pipeline(pipeline);

    if cli.http_timeout.is_some() {
        api_checker = api_checker.timeout(cli.http_timeout.unwrap());
    }
    if cli.interval_seconds.is_some() {
        api_checker = api_checker.run_every(cli.interval_seconds.unwrap());
    }
    api_checker = api_checker.build();

    // Rnus our API checker and metrics server in the background.
    let mut handles = vec![];
    handles.push(tokio::spawn(run_api_checker(api_checker)));

    let metrics_server = setup_metrics_server(cli.metrics_host, cli.metrics_port);
    handles.push(tokio::spawn(metrics_server));

    join_all(handles).await;
    Ok(())
}

pub async fn setup_metrics_server(host: String, port: u32) {
    info!("Starting prometheus metrics server");
    let router = Router::new().route("/metrics", get(crate::metrics::handler));
    let addr = format!("{}:{}", host, port);
    let server = axum::Server::bind(&addr.parse().unwrap()).serve(router.into_make_service());
    server.await.unwrap();
}

pub async fn run_api_checker(checker: ApiChecker) {
    info!("Starting API checker");
    let mut ticker = interval(checker.run_every);
    loop {
        ticker.tick().await;
        info!("Running API checker pipeline");
        checker.run_pipeline().await.unwrap();
    }
}

fn parse_duration(arg: &str) -> Result<Duration, std::num::ParseIntError> {
    let seconds = arg.parse()?;
    Ok(Duration::from_secs(seconds))
}
