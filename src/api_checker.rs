use eyre::Result;
use std::future::Future;
use std::pin::Pin;
use tokio::time::Duration;
use url::Url;

/// Turns the eyre Result type into an boxed future.
type AsyncResult = Pin<Box<dyn Future<Output = Result<()>> + Send + Sync>>;

/// Defines a function that can make API checks to series of endpoints
/// and verify the equality of the responses.
pub type CheckerFn = Box<dyn Fn(Vec<Url>) -> AsyncResult + Send + Sync>;

/// Forces a value a future to be boxed and send+sync for use
/// across threads in tokio. Used to convert our simple API checker functions
/// defined in endpoints.rs into the CheckerFn trait defined above for usage
/// in tokio::spawn calls.
pub fn force_boxed<T>(f: fn(Vec<Url>) -> T) -> CheckerFn
where
    T: Future<Output = Result<()>> + 'static + Send + Sync,
{
    Box::new(move |n| Box::pin(f(n)))
}

/// ApiChecker defines a struct which can perform a series of stress tests
/// and conformity checks against a series of beacon API endpoints
pub struct ApiChecker {
    /// How often to run the API checks against all endpoints.
    pub run_every: Duration,
    /// The beacon api endpoints to request.
    endpoints: Vec<Url>,
    /// The HTTP timeout when making requests.
    timeout: Duration,
    /// A pipeline of functions that the API checker will run
    /// against the endpoints to check for conformity.
    fns: Vec<CheckerFn>,
}

impl Default for ApiChecker {
    fn default() -> Self {
        ApiChecker {
            run_every: Duration::from_secs(60),
            endpoints: vec![],
            fns: vec![],
            timeout: Duration::from_secs(10),
        }
    }
}

impl ApiChecker {
    pub fn new() -> ApiChecker {
        ApiChecker::default()
    }
    pub fn run_every(mut self, duration: Duration) -> ApiChecker {
        self.run_every = duration;
        self
    }
    pub fn endpoints(mut self, items: Vec<Url>) -> ApiChecker {
        self.endpoints = items;
        self
    }
    pub fn timeout(mut self, timeout: Duration) -> ApiChecker {
        self.timeout = timeout;
        self
    }
    pub fn pipeline(mut self, fns: Vec<CheckerFn>) -> ApiChecker {
        self.fns = fns;
        self
    }
    pub fn build(self) -> ApiChecker {
        self
    }
    pub async fn run_pipeline(&self) -> Result<()> {
        for f in self.fns.iter() {
            f(self.endpoints.clone()).await?;
        }
        Ok(())
    }
}
