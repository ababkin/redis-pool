use anyhow::{Error, Result};
use tokio::sync::Mutex;
use tracing::{debug, error, warn};
use deadpool_redis::cluster::{Connection, Pool, Config, Runtime};


pub struct SafePool {
    pool: Mutex<Option<Pool>>,
    url: String,
}

impl SafePool {
    pub fn new(url: String) -> SafePool {
        SafePool {
            pool: Mutex::new(None),
            url,
        }
    }

    pub async fn invalidate(&self) {
        let mut lock = self.pool.lock().await;
        *lock = None;
    }

    pub async fn ensure(&self) {
        loop {
            debug!("Trying to connect to Redis...");
            match create(&self.url).await {
                Ok(pool) => {
                    let mut locked = self.pool.lock().await;
                    *locked = Some(pool);
                    debug!("Connected to Redis.");
                    break;
                },
                Err(e) => {
                    error!("Failed to connect to Redis: {:?}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // Wait before retrying
                }
            }
        }
    }

    pub async fn get(&self) -> Result<Connection, Error> {
        loop {
            {
                let lock = self.pool.lock().await;
                if let Some(pool) = &*lock {
                    return Ok(pool.get().await?)
                }
            }
            warn!("Redis connection lost, attempting to reconnect...");
            self.ensure().await;
        }
    }
}

async fn create(url: &str) -> Result<Pool, Error> {
    let cfg = Config::from_urls(vec![url.to_string()]);
    let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
    Ok(pool)
}



// fn create_cluster_pool(urls: &[String]) -> Result<Pool, Box<dyn std::error::Error>> {
//     let mut cfg = Config::from_urls(redis_urls);
//     let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();

// }
