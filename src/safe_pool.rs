use anyhow::{Error, Result};
use tracing::{debug, error, instrument};
use deadpool_redis::{Connection as NonClusterConnection, Pool as NonClusterPool, Manager as NonClusterManager};
use deadpool_redis::cluster::{Connection as ClusterConnection, Pool as ClusterPool, Manager as ClusterManager};
use redis::aio::ConnectionLike;
use redis::{RedisFuture, Cmd};
use redis::Pipeline;


// Enum to handle different connection types
pub enum RedisConnection {
    Cluster(ClusterConnection),
    NonCluster(NonClusterConnection),
}

impl ConnectionLike for RedisConnection {
    fn req_packed_command<'a>(&'a mut self, cmd: &'a Cmd) -> RedisFuture<'a, redis::Value> {
        match self {
            RedisConnection::Cluster(conn) => conn.req_packed_command(cmd),
            RedisConnection::NonCluster(conn) => conn.req_packed_command(cmd),
        }
    }

    fn req_packed_commands<'a>(&'a mut self, cmd: &'a Pipeline, offset: usize, count: usize) -> RedisFuture<'a, Vec<redis::Value>> {
        match self {
            RedisConnection::Cluster(conn) => conn.req_packed_commands(cmd, offset, count),
            RedisConnection::NonCluster(conn) => conn.req_packed_commands(cmd, offset, count),
        }
    }

    fn get_db(&self) -> i64 {
        match self {
            RedisConnection::Cluster(conn) => conn.get_db(),
            RedisConnection::NonCluster(conn) => conn.get_db(),
        }
    }
}

pub enum RedisMode {
    Cluster,
    NonCluster,
}

pub enum RedisPool {
    Cluster(ClusterPool),
    NonCluster(NonClusterPool),
}

pub struct SafePool {
    pool: RedisPool,
}

impl SafePool {
    pub fn new(urls: Vec<String>, mode: RedisMode, pool_size: usize) -> SafePool {
        let pool = match mode {
            RedisMode::Cluster => {
                match create_cluster_pool(urls, pool_size) {
                    Ok(pool) => {
                        debug!("Connected to Redis Cluster.");
                        RedisPool::Cluster(pool)
                    }
                    Err(e) => {
                        let e = format!("Failed to connect to Redis Cluster: {:?}", e);
                        error!("{}", e);
                        panic!("{}", e);
                    }
                }
            }
            RedisMode::NonCluster => {
                match create_non_cluster_pool(urls, pool_size) {
                    Ok(pool) => {
                        debug!("Connected to Redis Non-Cluster.");
                        RedisPool::NonCluster(pool)
                    }
                    Err(e) => {
                        let e = format!("Failed to connect to Redis Non-Cluster: {:?}", e);
                        error!("{}", e);
                        panic!("{}", e);
                    }
                }
            }
        };

        SafePool { pool }
    }

    #[instrument(name = "get_connection", skip(self))]
    pub async fn get(&self) -> Result<RedisConnection, Error> {
        match &self.pool {
            RedisPool::Cluster(pool) => {
                Ok(RedisConnection::Cluster(pool.get().await?))
            }
            RedisPool::NonCluster(pool) => {
                Ok(RedisConnection::NonCluster(pool.get().await?))
            }
        }
    }
}

fn create_cluster_pool(urls: Vec<String>, pool_size: usize) -> Result<ClusterPool, Error> {
    let manager = ClusterManager::new(urls)?;
    let pool = ClusterPool::builder(manager)
        .max_size(pool_size)  // Adjust pool size according to your needs
        .build()
        .expect("must be able to create cluster pool");
    Ok(pool)
}

fn create_non_cluster_pool(urls: Vec<String>, pool_size: usize) -> Result<NonClusterPool, Error> {
    let url = urls.first().ok_or_else(|| Error::msg("No URL provided"))?;
    let manager = NonClusterManager::new(url.to_string())?;
    let pool = NonClusterPool::builder(manager)
        .max_size(pool_size)  // Adjust pool size according to your needs
        .build()
        .expect("must be able to create non-cluster pool");
    Ok(pool)
}
