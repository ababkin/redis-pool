use anyhow::{Error, Result};
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

    #[instrument(name = "get_connection", skip(self))]
    pub async fn get(&self) -> Result<Connection, Error> {
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
