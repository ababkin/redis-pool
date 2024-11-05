use anyhow::{Error, Result};
use tracing::{debug, error, info};
// use redis::{Cmd, Pipeline};
use deadpool_redis::{Connection, Pool, Config, PoolConfig, Runtime};
use std::sync::Arc;

/// Struct representing a safe Redis connection pool for non-cluster Redis with SSL support.
pub struct SafePool {
    pool: Pool,
}

impl SafePool {
    /// Creates a new `SafePool` for non-cluster Redis with SSL support.
    ///
    /// # Arguments
    ///
    /// * `urls` - A vector of Redis server URLs. Only the first URL is used. Supports both `redis://` and `rediss://`.
    /// * `pool_size` - The maximum number of connections in the pool.
    ///
    /// # Panics
    ///
    /// Panics if the pool cannot be created.
    pub fn new(url: String, pool_size: usize) -> SafePool {
        let pool = match create(url, pool_size) {
            Ok(pool) => {
                debug!("Connected to Redis Non-Cluster with SSL support.");
                pool
            }
            Err(e) => {
                let e = format!("Failed to connect to Redis Non-Cluster with SSL support: {:?}", e);
                error!("{}", e);
                panic!("{}", e);
            }
        };

        SafePool { pool }
    }

    /// Retrieves a connection from the pool.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if a connection cannot be obtained.
    // #[instrument(name = "get_connection", skip(self))]
    pub async fn get(&self) -> Result<Connection, Error> {
        let conn = self.pool.get().await?;
        Ok(conn)
    }
}

/// Creates a non-cluster Redis connection pool with SSL support.
///
/// # Arguments
///
/// * `urls` - A vector of Redis server URLs. Only the first URL is used. Supports both `redis://` and `rediss://`.
/// * `pool_size` - The maximum number of connections in the pool.
///
/// # Errors
///
/// Returns an `Error` if no URL is provided or if the pool cannot be created.
fn create(url: String, pool_size: usize) -> Result<Pool, Error> {

    // let crypto_provider = CryptoProvider::new(); // or another constructor if available
    // Arc::new(crypto_provider)
    //     .install_default()
    //     .expect("Failed to install default CryptoProvider");

    info!("connecting to redis: {}", url);

    let mut config = Config::from_url(url);

    let pool = config
        .create_pool(Some(Runtime::Tokio1))
        .map_err(|e| Error::msg(format!("Failed to create Redis pool: {}", e)))?;

    Ok(pool)
}
