use std::future::Future;

#[derive(Clone)]
pub struct Executor;

impl<Fut> hyper::rt::Executor<Fut> for Executor
where
	Fut: Future<Output = ()> + Send + 'static,
{
	fn execute(&self, fut: Fut) {
		tokio::spawn(fut);
	}
}
