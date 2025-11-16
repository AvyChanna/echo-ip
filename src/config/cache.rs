use std::net::IpAddr;

use async_trait::async_trait;
pub use moka::future::Cache as MokaCache;
use thiserror::Error;

use super::IpInfo;

#[async_trait]
pub trait Cacher {
	#[must_use]
	async fn lookup(&self, ip: IpAddr) -> Result<IpInfo, CacheError>;
	#[must_use]
	async fn store(&self, ip: IpAddr, lookup_result: IpInfo) -> Result<(), CacheError>;
}

#[derive(Error, Debug)]
pub enum CacheError {
	#[error("IP not found")]
	NotFound,
}

#[async_trait]
impl Cacher for MokaCache<IpAddr, IpInfo> {
	async fn lookup(&self, ip: IpAddr) -> Result<IpInfo, CacheError> {
		match self.get(&ip).await {
			Some(value) => Ok(value),
			None => Err(CacheError::NotFound),
		}
	}

	async fn store(&self, ip: IpAddr, lookup_result: IpInfo) -> Result<(), CacheError> {
		self.insert(ip, lookup_result).await;
		Ok(())
	}
}
