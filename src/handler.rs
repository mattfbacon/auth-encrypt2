use std::convert::Infallible;
use std::io::ErrorKind;
use std::path::{Component, Path};

use bytes::Bytes;
use http::header::{AUTHORIZATION, WWW_AUTHENTICATE};
use http::{HeaderValue, StatusCode};
use hyper::{Body, Request, Response};

use crate::decrypt::decrypt;

async fn handler_inner(req: Request<Body>) -> Result<Body, StatusCode> {
	const MAX_AUTH_LEN: usize = 2048;

	let path = req.uri().path().trim_start_matches('/');
	let path = Path::new(path);

	if path
		.components()
		.any(|component| !matches!(component, Component::Normal(..)))
	{
		return Err(StatusCode::NOT_FOUND);
	}

	let auth = req
		.headers()
		.get(AUTHORIZATION)
		.ok_or(StatusCode::UNAUTHORIZED)?;
	// `auth.strip_prefix_ignore_ascii_case("Basic ")`
	let [b'b' | b'B', b'a' | b'A', b's' | b'S', b'i' | b'I', b'c' | b'C', b' ', auth @ ..] = auth.as_bytes() else { return Err(StatusCode::BAD_REQUEST); };
	if auth.len() > MAX_AUTH_LEN {
		return Err(StatusCode::BAD_REQUEST);
	}

	let file = tokio::fs::File::open(path)
		.await
		.map_err(|error| match error.kind() {
			ErrorKind::NotFound => StatusCode::NOT_FOUND,
			_ => StatusCode::INTERNAL_SERVER_ERROR,
		})?;
	let file = tokio::io::BufReader::new(file);

	decrypt(file, Bytes::copy_from_slice(auth))
		.await
		.map(Body::wrap_stream)
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
	let response = match handler_inner(req).await {
		Ok(body) => Response::new(body),
		Err(code) => {
			let mut response = Response::new(code.canonical_reason().unwrap_or("").into());
			*response.status_mut() = code;
			if code == StatusCode::UNAUTHORIZED {
				response
					.headers_mut()
					.insert(WWW_AUTHENTICATE, HeaderValue::from_static("Basic"));
			}
			response
		}
	};
	Ok(response)
}
