use std::net::IpAddr;

use async_trait::async_trait;
pub use hickory_resolver::Resolver as HickoryResolver;
use hickory_resolver::{name_server::ConnectionProvider, proto::rr::rdata::PTR};

#[async_trait]
pub trait ReverseLookupProvider {
	#[must_use]
	async fn rev_lookup(&self, ip: IpAddr) -> Vec<String>;
}

#[async_trait]
impl<P: ConnectionProvider> ReverseLookupProvider for HickoryResolver<P> {
	async fn rev_lookup(&self, ip: IpAddr) -> Vec<String> {
		match self.reverse_lookup(ip).await {
			Ok(res) => res.iter().filter_map(is_ascii_domain_name).collect(),
			Err(_) => Vec::new(),
		}
	}
}

fn is_ascii_domain_name(s: &PTR) -> Option<String> {
	// converts utf8 to xn--*
	// This screws up some cases where the length changes after conversion
	// OKish for now
	let inp = s.to_ascii();
	let l = s.len();
	if l == 0 || l > 254 || l == 254 && !inp.ends_with('.') {
		return None;
	}

	// RULES:
	// - overall length <= 254
	// - each sub-domain-part length <= 63
	// - only [a-zA-Z0-9.-] allowed
	// - sub-domain-parts can not start or end with -
	// - must contain at least one alphabet or underscore
	// - ../.-/-. not allowed
	let mut last_char = '.';
	let mut has_alpha = false;
	let mut sublength = 0;
	for c in inp.chars() {
		match c {
			// do nothing
			'a'..='z' | 'A'..='Z' | '_' => {
				has_alpha = true;
				sublength += 1;
			}
			// do nothing
			'0'..='9' => {
				sublength += 1;
			}
			// Can not be .-
			'-' => {
				if last_char == '.' {
					return None;
				}
				sublength += 1;
				has_alpha = true;
			}
			// can not be .. or .-
			// also check curr part length
			'.' => {
				if last_char == '.' || last_char == '-' || sublength > 63 || sublength == 0 {
					return None;
				}
				sublength = 0;
			}
			// only [a-zA-Z0-9.-] allowed
			_ => return None,
		}
		last_char = c;
	}
	if last_char == '-' || sublength > 63 {
		return None;
	}

	has_alpha.then(|| inp)
}
