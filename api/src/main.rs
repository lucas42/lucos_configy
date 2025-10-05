use axum::{
	response::IntoResponse,
	routing::get,
	Json, Router,
	http::{HeaderMap, HeaderValue, StatusCode},
};
use serde::Serialize;
use std::{env, net::SocketAddr};
use tokio::signal;

#[derive(Serialize)]
struct InfoCI {
	circle: &'static str,
}

#[derive(Serialize)]
struct InfoResponse {
	system: &'static str,
	title: &'static str,
	network_only: bool,
	show_on_homepage: bool,
	ci: InfoCI,
}

async fn root() -> impl IntoResponse {
	"Hello World"
}

async fn info() -> impl IntoResponse {
	let mut headers = HeaderMap::new();
	headers.insert("X-App-Version", HeaderValue::from_static("1.0"));
	let json = Json(InfoResponse {
		system: "lucos_configy",
		title: "LucOS Configy",
		ci: InfoCI {
			circle: "gh/lucas42/lucos_configy",
		},
		network_only: true,
		show_on_homepage: false,
	});
	(StatusCode::OK, headers, json)
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
	let port: u16 = env::var("PORT")
		.ok()
		.and_then(|p| p.parse().ok())
		.unwrap_or(3000);

	let app = Router::new()
		.route("/", get(root))
		.route("/_info", get(info));

	let addr = SocketAddr::from(([0, 0, 0, 0], port));
	println!("Listening on {}", addr);

	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

	axum::serve(listener, app)
		.with_graceful_shutdown(shutdown_signal())
		.await
		.unwrap();
}
