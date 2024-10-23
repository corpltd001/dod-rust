use crate::fetcher::FetcherService;
use crate::threads::ThreadsManager;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

pub static THREADS: Lazy<Mutex<ThreadsManager>> =
    Lazy::new(|| Mutex::new(ThreadsManager::default()));

pub static MINER: Lazy<Mutex<FetcherService>> = Lazy::new(|| Mutex::new(FetcherService::default()));
pub static LATEST_BLOCK: Lazy<Mutex<Option<u64>>> = Lazy::new(|| Mutex::new(None));

pub static RUNNING: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
