use axum::{extract::Request, response::Response};
use log::{log, Level};
use std::{
	future::Future,
	pin::Pin,
	task::{Context, Poll},
};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct LogLayer;

impl LogLayer {
	pub fn new() -> Self {
		Self {}
	}
}

impl<S> Layer<S> for LogLayer {
	type Service = LogMiddleware<S>;

	fn layer(&self, inner: S) -> Self::Service {
		LogMiddleware { inner }
	}
}

#[derive(Clone)]
pub struct LogMiddleware<S> {
	inner: S,
}

impl<S> Service<Request> for LogMiddleware<S>
where
	S: Service<Request, Response = Response> + Send + 'static,
	S::Future: Send + 'static,
{
	type Response = S::Response;
	type Error = S::Error;
	type Future =
		Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx)
	}

	fn call(&mut self, request: Request) -> Self::Future {
		let uri = request.uri().clone();
		let future = self.inner.call(request);
		Box::pin(async move {
			let response: Response = future.await?;
			let level = if response.status().is_success() {
				Level::Info
			} else {
				Level::Error
			};
			log!(level, "[{}] {}", response.status(), uri);
			Ok(response)
		})
	}
}
