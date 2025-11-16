use std::net::SocketAddr;

use axum::{
	Extension, Router, debug_handler,
	routing::{self},
};
use axum_extra::routing::RouterExt;
use hickory_resolver::name_server::ConnectionProvider;
use thiserror::Error;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::config::{cache::Cacher, mmdb::GeoInfoProvider, serverconf::ServerConfig};

#[derive(Error, Debug)]
pub enum ServerError {
	#[error(transparent)]
	Io(#[from] std::io::Error),
	#[error(transparent)]
	Axum(#[from] axum::Error),
}

pub async fn serve<
	C: Cacher + Clone + Send + Sync + 'static,
	L: GeoInfoProvider + Clone + Send + Sync + 'static,
	P: ConnectionProvider,
>(
	bind_addr: SocketAddr,
	ctx: ServerConfig<C, L, P>,
) -> Result<(), ServerError> {
	let service = Router::new()
		.route_with_tsr("/dummy", routing::post(dummy))
		.layer(
			ServiceBuilder::new()
				.layer(Extension(ctx))
				.layer(TraceLayer::new_for_http()),
		)
		.into_make_service();

	let listener = tokio::net::TcpListener::bind(bind_addr).await?;

	axum::serve(listener, service)
		.with_graceful_shutdown(shutdown_signal())
		.await?;
	Ok(())
}

async fn shutdown_signal() {
	let ctrl_c = async {
		signal::ctrl_c()
			.await
			.expect("failed to install Ctrl+C handler");
	};

	#[cfg(unix)]
	let terminate = async {
		signal::unix::signal(signal::unix::SignalKind::terminate())
			.expect("failed to install signal handler")
			.recv()
			.await;
	};

	#[cfg(not(unix))]
	let terminate = std::future::pending::<()>();

	tokio::select! {
		() = ctrl_c => {},
		() = terminate => {},
	}

	info!("shutting down server");
}

#[debug_handler]
async fn dummy() -> &'static str {
	"Hello, World!"
}
