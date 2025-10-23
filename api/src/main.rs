mod data;
mod info;
mod systems;
mod volumes;
mod hosts;

use axum::{
	response::Redirect,
	routing::get,
	Router,
};
use std::{env, net::SocketAddr};
use tokio::signal;
use std::sync::Arc;

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
	let arc_data;
	match crate::data::Data::from_file("config.yaml") {
		Ok(data) => {
			println!("Loaded {} systems; {} volumes; {} hosts", data.system_count(), data.volume_count(), data.host_count());
			arc_data = Arc::new(data);
		}
		Err(err) => {
			panic!("Failed to load config, ({:?})", err);
		}
	}

	let port: u16 = env::var("PORT")
		.ok()
		.and_then(|p| p.parse().ok())
		.unwrap_or(3000);

	let app = Router::new()
		.route("/", get(Redirect::temporary("/systems")))
		.route("/_info", get(crate::info::controller))
		.route("/systems", get(crate::systems::all))
		.route("/systems/subdomain/{root_domain}", get(crate::systems::subdomain))
		.route("/systems/http", get(crate::systems::http))
		.route("/systems/host/{host}", get(crate::systems::host))
		.route("/systems{*_subpath}", get(Redirect::temporary("/systems")))
		.route("/volumes", get(crate::volumes::all))
		.route("/volumes{*_subpath}", get(Redirect::temporary("/volumes")))
		.route("/hosts", get(crate::hosts::all))
		.route("/hosts{*_subpath}", get(Redirect::temporary("/hosts")))
		.with_state(arc_data);

	let addr = SocketAddr::from(([0, 0, 0, 0], port));
	println!("Listening on {}", addr);

	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

	axum::serve(listener, app)
		.with_graceful_shutdown(shutdown_signal())
		.await
		.unwrap();
}
