pub mod cache;
pub mod mmdb;
pub mod revlookup;
pub mod serverconf;

#[derive(Debug, Clone)]
pub struct IpInfo {
	_ip: String,
	// hostname: Option<String>,
}
