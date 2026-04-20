use axum::{
	response::Redirect,
	routing::get,
	Router,
};
use std::sync::Arc;

pub fn app(arc_data: Arc<crate::data::Data>) -> Router {
	Router::new()
		.route("/all", get(crate::all::all))
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
		.route("/hosts/http", get(crate::hosts::http))
		.route("/hosts{*_subpath}", get(Redirect::temporary("/hosts")))
		.route("/components", get(crate::components::all))
		.route("/components{*_subpath}", get(Redirect::temporary("/components")))
		.route("/scripts", get(crate::scripts::all))
		.route("/scripts{*_subpath}", get(Redirect::temporary("/scripts")))
		.route("/repositories/{id}", get(crate::repositories::get))
		.with_state(arc_data)
}
