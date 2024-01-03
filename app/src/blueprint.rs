use pavex::{
	blueprint::{
		constructor::{CloningStrategy, Lifecycle},
		Blueprint,
	},
	f,
	request::{
		body::{BodySizeLimit, BufferedBody, JsonBody},
		path::PathParams,
		query::QueryParams,
	},
};

use crate::routes;

/// The main blueprint, containing all the routes, constructors and error handlers required by our API.
#[must_use]
pub fn blueprint() -> Blueprint {
	let mut bp = Blueprint::new();

	register_common_constructors(&mut bp);
	add_telemetry_middleware(&mut bp);
	routes::handler(&mut bp);

	bp
}

/// Common constructors used by all routes.
fn register_common_constructors(bp: &mut Blueprint) {
	JsonBody::register(bp);
	PathParams::register(bp);
	QueryParams::register(bp);
	BufferedBody::register(bp);
	BodySizeLimit::register(bp);
}

/// Add the telemetry middleware, as well as the constructors of its dependencies.
fn add_telemetry_middleware(bp: &mut Blueprint) {
	bp.constructor(
		f!(crate::telemetry::RootSpan::new),
		Lifecycle::RequestScoped,
	)
	.cloning(CloningStrategy::CloneIfNecessary);

	bp.wrap(f!(crate::telemetry::logger));
}
