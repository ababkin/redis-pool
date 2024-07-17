use anyhow::{Error, Result};
use tokio::sync::RwLock;
use tracing::{ debug, error, warn, instrument };
use deadpool_redis::cluster::{Connection, Pool, Config, Runtime};

pub struct SafePool {
    pool: Option<Pool>,
}

impl SafePool {
    pub fn new(urls: Vec<String>) -> SafePool {


        match create(urls) {
            Ok(pool) => {

                debug!("Connected to Redis.");

                SafePool {
                    pool: Some(pool),
                }
            },
            Err(e) => {
                let e = format!("Failed to connect to Redis: {:?}", e);
                error!(e);
                panic!("{}", e);
            }
        }

    }

    pub async fn invalidate(&self) { //TODO
        // let mut lock = self.pool.write().await;
        // *lock = None;
    }

    #[instrument(skip(self))]
    pub async fn ensure(&self) { // TODO
        // loop {
        //     debug!("Trying to connect to Redis...");
        //     match create(&self.url) {
        //         Ok(pool) => {
        //             let mut locked = self.pool.write().await;
        //             *locked = Some(pool);
        //             debug!("Connected to Redis.");
        //             break;
        //         },
        //         Err(e) => {
        //             error!("Failed to connect to Redis: {:?}", e);
        //             tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // Wait before retrying
        //         }
        //     }
        // }
    }

    #[instrument(name = "get_connection", skip(self))]
    pub async fn get(&self) -> Result<Connection, Error> {
        // loop {
        //     {
        //         let lock = self.pool.read().await;
        //         if let Some(pool) = &*lock {
        //             return Ok(pool.get().await?);
        //         }
        //     }
        //     warn!("Redis connection lost, attempting to reconnect...");
        //     self.ensure().await;
        // }

        Ok(self.pool.as_ref().unwrap().get().await?)
    }
}

fn create(urls: Vec<String>) -> Result<Pool, Error> {
    // let cfg = Config::from_urls(vec![url.to_string()]);
    // let pool = cfg.create_pool(Some(Runtime::Tokio1))?;

    let manager = deadpool_redis::cluster::Manager::new(urls)?;
    let pool = deadpool_redis::cluster::Pool::builder(manager)
        .max_size(64)  // Adjust pool size according to your needs
        .build()
        .unwrap();

    Ok(pool)
}
