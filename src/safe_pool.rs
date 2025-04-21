use tracing::{debug, error, info};
use deadpool_redis::{Connection, Pool, Config, Runtime};
use anyhow::Error;

/// Struct representing a safe Redis connection pool for non-cluster Redis with SSL support.
pub struct SafePool {
    pool: Pool,
}

impl SafePool {
    /// Creates a new `SafePool` for non-cluster Redis with SSL support.
    ///
    /// # Arguments
    ///
    /// * `url` - Redis server URL. Supports both `redis://` and `rediss://`.
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
/// * `url` - Redis server URL. Supports both `redis://` and `rediss://`.
/// * `pool_size` - The maximum number of connections in the pool.
///
/// # Errors
///
/// Returns an `Error` if no URL is provided or if the pool cannot be created.
fn create(url: String, pool_size: usize) -> Result<Pool, Error> {
    info!("connecting to redis: {}", url);

    let config = Config::from_url(url);
    let mut config = config;
    config.pool = Some(deadpool_redis::PoolConfig::new(pool_size));

    let pool = config.create_pool(Some(Runtime::Tokio1))?;

    Ok(pool)
}
