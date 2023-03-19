use eyre::Result;
use std::future::Future;
use std::pin::Pin;
use tokio::time::Duration;
use url::Url;

pub type AsyncResult = Pin<Box<dyn Future<Output = Result<()>> + Send + Sync>>;
pub type CheckerFn = Box<dyn Fn(Url) -> AsyncResult + Send + Sync>;

pub fn force_boxed<T>(f: fn(Url) -> T) -> CheckerFn
where
    T: Future<Output = Result<()>> + 'static + Send + Sync,
{
    Box::new(move |n| Box::pin(f(n)))
}

pub struct ApiChecker {
    pub run_every: Duration,
    endpoints: Vec<Url>,
    timeout: Duration,
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
            for e in self.endpoints.iter() {
                f(e.clone()).await?;
            }
        }
        Ok(())
    }
}
