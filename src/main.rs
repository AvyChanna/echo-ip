use std::net::SocketAddr;

use clap::Parser;
use echo_ip::{
	config::{
		mmdb::{GeoInfoError, MMDB},
		serverconf::ServerConfigBuilder,
	},
	server::{ServerError, serve},
};
use hickory_resolver::{
	ResolveError, name_server::TokioConnectionProvider, system_conf::read_system_conf,
};
use moka::future::Cache;
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(about, version, long_version=include_str!(concat!(env!("OUT_DIR"), "/long-help.txt")))]
struct Cli {
	#[arg(short, long)]
	mmdb_path: Option<String>,
	#[arg(short, long, default_value = "127.0.0.1:8080")]
	bind_addr: SocketAddr,
	#[arg(short, long, num_args = 0..=1, default_missing_value = "true")]
	reverse_lookup: bool,
	// #[arg(short, long, num_args = 0..=1, default_missing_value = "true")]
	// port_lookup: bool,
	#[arg(short, long, default_value_t = 1024)]
	cache_size: u64,
	#[arg(short, long, num_args = 1, value_delimiter = ',')]
	ip_headers: Vec<String>,
}

#[derive(Debug, Error)]
enum AppError {
	#[error("Axum Error: {0}")]
	AxumError(#[from] ServerError),
	#[error("Hickory Error: {0}")]
	ResolveError(#[from] ResolveError),
	#[error("MMDB Error: {0}")]
	MMDBError(#[from] GeoInfoError),
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), AppError> {
	let cli = Cli::parse();

	let mut cfg_builder = ServerConfigBuilder::new();

	cfg_builder = cfg_builder.ip_headers(cli.ip_headers);

	if cli.cache_size != 0 {
		cfg_builder = cfg_builder.cache(Cache::new(cli.cache_size));
	}

	if let Some(path) = cli.mmdb_path {
		cfg_builder = cfg_builder.ip_lookup(MMDB::new(path)?);
	}

	if cli.reverse_lookup {
		let (cfg, _) = read_system_conf()?;
		cfg_builder = cfg_builder.rev_lookup(
			hickory_resolver::Resolver::builder_with_config(
				cfg,
				TokioConnectionProvider::default(),
			)
			.build(),
		);
	}

	let cfg = cfg_builder.build();
	serve(cli.bind_addr, cfg).await?;

	Ok(())
}
