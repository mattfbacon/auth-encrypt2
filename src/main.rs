#![deny(
	absolute_paths_not_starting_with_crate,
	keyword_idents,
	macro_use_extern_crate,
	meta_variable_misuse,
	missing_abi,
	missing_copy_implementations,
	non_ascii_idents,
	nonstandard_style,
	noop_method_call,
	pointer_structural_match,
	private_in_public,
	rust_2018_idioms,
	unused_qualifications
)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use std::convert::Infallible;

use hyper::service::{make_service_fn, service_fn};
use tokio::net::{UnixListener, UnixStream};

use crate::executor::Executor;
use crate::handler::handler;
use crate::incoming::Incoming;

mod decrypt;
mod executor;
mod handler;
mod incoming;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	let path = std::env::var_os("LISTEN_ON").expect("missing `LISTEN_ON` env var");
	_ = std::fs::remove_file(&path);
	let socket = UnixListener::bind(path).expect("IO error binding to `LISTEN_ON`");
	let server = hyper::Server::builder(Incoming(socket))
		.executor(Executor)
		.serve(make_service_fn(|_: &UnixStream| {
			std::future::ready(Ok::<_, Infallible>(service_fn(handler)))
		}));
	server.await.unwrap();
}
