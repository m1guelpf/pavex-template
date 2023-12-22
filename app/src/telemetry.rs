use pavex::{
	http::Version,
	middleware::Next,
	request::{route::MatchedRouteTemplate, RequestHead},
	response::Response,
};
use std::{borrow::Cow, future::IntoFuture};
use tracing::Instrument;

/// A logging middleware that wraps the request pipeline in the root span.
/// It takes care to record key information about the request and the response.
pub async fn logger<T>(next: Next<T>, root_span: RootSpan) -> Response
where
	T: Send + IntoFuture<Output = Response>,
	T::IntoFuture: Send,
{
	let response = next
		.into_future()
		.instrument(root_span.clone().into_inner())
		.await;

	root_span.record_response_data(&response);

	response
}

/// A root span is the top-level *logical* span for an incoming request.
///
/// It is not necessarily the top-level *physical* span, as it may be a child of
/// another span (e.g. a span representing the underlying HTTP connection).
///
/// We use the root span to attach as much information as possible about the
/// incoming request, and to record the final outcome of the request (success or
/// failure).
#[derive(Debug, Clone)]
pub struct RootSpan(tracing::Span);

impl RootSpan {
	/// Create a new root span for the given request.
	///
	/// We follow `OpenTelemetry`'s HTTP semantic conventions as closely as
	/// possible for field naming.
	pub fn new(request_head: &RequestHead, matched_route: MatchedRouteTemplate) -> Self {
		let user_agent = request_head
			.headers
			.get("User-Agent")
			.map(|h| h.to_str().unwrap_or_default())
			.unwrap_or_default();

		let span = tracing::info_span!(
			"HTTP request",
			http.method = %request_head.method,
			http.flavor = %http_flavor(request_head.version),
			user_agent.original = %user_agent,
			http.response.status_code = tracing::field::Empty,
			http.route = %matched_route,
			http.target = %request_head.uri.path_and_query().map_or("", |p| p.as_str()),
		);
		Self(span)
	}

	pub fn record_response_data(&self, response: &Response) {
		self.0
			.record("http.response.status_code", response.status().as_u16());
	}

	/// Get a reference to the underlying [`tracing::Span`].
	#[must_use]
	pub const fn inner(&self) -> &tracing::Span {
		&self.0
	}

	/// Deconstruct the root span into its underlying [`tracing::Span`].
	#[must_use]
	pub fn into_inner(self) -> tracing::Span {
		self.0
	}
}

/// Return the HTTP version as a string.
fn http_flavor(version: Version) -> Cow<'static, str> {
	match version {
		Version::HTTP_09 => "0.9".into(),
		Version::HTTP_10 => "1.0".into(),
		Version::HTTP_11 => "1.1".into(),
		Version::HTTP_2 => "2.0".into(),
		Version::HTTP_3 => "3.0".into(),
		other => format!("{other:?}").into(),
	}
}
