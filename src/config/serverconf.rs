use super::cache::Cacher;
use super::mmdb::GeoInfoProvider;
use hickory_resolver::{Resolver, name_server::ConnectionProvider};

#[derive(Debug, Clone)]
pub struct ServerConfig<C: Cacher, L: GeoInfoProvider, P: ConnectionProvider> {
	_ip_headers: Vec<String>,
	_ip_lookup: Option<L>,
	_rev_lookup: Option<Resolver<P>>,
	_cache: Option<C>,
}

impl<C: Cacher, L: GeoInfoProvider, P: ConnectionProvider> ServerConfig<C, L, P> {
	#[must_use]
	fn new(
		ip_headers: Vec<String>,
		ip_lookup: Option<L>,
		rev_lookup: Option<Resolver<P>>,
		cache: Option<C>,
	) -> Self {
		Self {
			_ip_headers: ip_headers,
			_ip_lookup: ip_lookup,
			_rev_lookup: rev_lookup,
			_cache: cache,
		}
	}
}

#[derive(Debug)]
pub struct ServerConfigBuilder<C: Cacher, L: GeoInfoProvider, P: ConnectionProvider> {
	ip_headers: Vec<String>,
	ip_lookup: Option<L>,
	rev_lookup: Option<Resolver<P>>,
	cache: Option<C>,
}

impl<C: Cacher, L: GeoInfoProvider, P: ConnectionProvider> Default
	for ServerConfigBuilder<C, L, P>
{
	fn default() -> Self {
		Self::new()
	}
}

impl<C: Cacher, L: GeoInfoProvider, P: ConnectionProvider> ServerConfigBuilder<C, L, P> {
	#[must_use]
	pub fn new() -> Self {
		Self {
			ip_headers: vec![],
			ip_lookup: None,
			rev_lookup: None,
			cache: None,
		}
	}

	#[must_use]
	pub fn ip_headers(mut self, headers: Vec<String>) -> Self {
		self.ip_headers = headers;
		self
	}

	#[must_use]
	pub fn ip_lookup(mut self, lookup: L) -> Self {
		self.ip_lookup = Some(lookup);
		self
	}

	#[must_use]
	pub fn rev_lookup(mut self, lookup: Resolver<P>) -> Self {
		self.rev_lookup = Some(lookup);
		self
	}

	#[must_use]
	pub fn cache(mut self, cache: C) -> Self {
		self.cache = Some(cache);
		self
	}

	#[must_use]
	pub fn build(self) -> ServerConfig<C, L, P> {
		ServerConfig::new(self.ip_headers, self.ip_lookup, self.rev_lookup, self.cache)
	}
}
