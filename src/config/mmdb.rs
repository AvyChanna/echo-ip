use std::{borrow::Cow, net::IpAddr, sync::Arc};

use maxminddb::{MaxMindDbError, Reader, geoip2};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeoInfoError {
	#[error("No information available")]
	NoInfo,
	#[error("IP unknown")]
	UnknownIP,
	#[error(transparent)]
	MaxMindDbError(MaxMindDbError),
}

impl From<MaxMindDbError> for GeoInfoError {
	fn from(err: MaxMindDbError) -> Self {
		GeoInfoError::MaxMindDbError(err)
	}
}

pub trait GeoInfoProvider {
	#[must_use]
	fn lookup(&self, ip: IpAddr) -> Result<Cow<'_, str>, GeoInfoError>;
}

#[derive(Debug, Clone)]
pub struct MMDB {
	reader: Arc<Reader<Vec<u8>>>,
}

impl MMDB {
	#[must_use]
	pub fn new<T: AsRef<str>>(path: T) -> Result<Self, GeoInfoError> {
		let reader = Reader::open_readfile(path.as_ref())?;
		Ok(Self {
			reader: Arc::new(reader),
		})
	}
}

impl GeoInfoProvider for MMDB {
	fn lookup(&self, ip: IpAddr) -> Result<Cow<'_, str>, GeoInfoError> {
		match self.reader.lookup::<geoip2::City<'_>>(ip) {
			Ok(Some(city)) => {
				if let Some(city_name) = city
					.city
					.and_then(|c| c.names)
					.and_then(|n| n.get("en").cloned())
				{
					Ok(Cow::Borrowed(city_name))
				} else {
					Err(GeoInfoError::UnknownIP)
				}
			}
			Ok(None) => Err(GeoInfoError::UnknownIP),
			Err(err) => Err(GeoInfoError::from(err)),
		}
	}
}
