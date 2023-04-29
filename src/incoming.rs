use std::pin::Pin;
use std::task::{Context, Poll};

use hyper::server::accept::Accept;
use tokio::net::{UnixListener, UnixStream};

pub struct Incoming(pub UnixListener);

impl Accept for Incoming {
	type Conn = UnixStream;

	type Error = std::io::Error;

	fn poll_accept(
		self: Pin<&mut Self>,
		ctx: &mut Context<'_>,
	) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
		self
			.0
			.poll_accept(ctx)
			.map(|res| Some(res.map(|(socket, _addr)| socket)))
	}
}
