use pavex::http::StatusCode;

/// Respond with a `200 OK` status code to indicate that the server is alive and ready to accept new requests.
pub fn health_check() -> StatusCode {
	StatusCode::OK
}
