use anyhow::{Result, Error};
use std::env;
use url::Url;
use tokio::sync::Mutex;
use deadpool_redis::{redis::cmd, Config as PoolConfig, Connection, Manager, Pool, Runtime};
use tracing::{error, debug, warn, info};


#[derive(Debug)]
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
            match create(&self.url) {
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
            // tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // Wait before retrying
        }
    }

}

fn create(url: &str) -> Result<Pool, Error> {
    let cfg = deadpool_redis::Config::from_url(url);
    let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
    Ok(pool)
}