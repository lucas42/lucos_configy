mod data;
mod info;

use axum::{
	response::IntoResponse,
	routing::get,
	Router,
};
use std::{env, net::SocketAddr};
use tokio::signal;


async fn root() -> impl IntoResponse {
	"Hello World"
}

// Wait for SIGINT or SIGTERM
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
		_ = ctrl_c => {},
		_ = terminate => {},
	}

	println!("Signal received, starting graceful shutdown...");
}

#[tokio::main]
async fn main() {
	match crate::data::Data::from_file("config.yaml") {
		Ok(data) => {
			println!("Loaded {} systems; {} volumes; {} hosts", data.systemCount(), data.volumeCount(), data.hostCount());
		}
		Err(err) => {
			println!("Failed to load config, ({:?})", err);
		}
	}

	let port: u16 = env::var("PORT")
		.ok()
		.and_then(|p| p.parse().ok())
		.unwrap_or(3000);

	let app = Router::new()
		.route("/", get(root))
		.route("/_info", get(crate::info::controller));

	let addr = SocketAddr::from(([0, 0, 0, 0], port));
	println!("Listening on {}", addr);

	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

	axum::serve(listener, app)
		.with_graceful_shutdown(shutdown_signal())
		.await
		.unwrap();
}
