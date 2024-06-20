use std::{sync::Arc, task::Poll};

use crate::{Error, HttpClient};
use futures_core::future::BoxFuture;
use http::{Request, Response};
use tower_service::Service;

pub struct ReggieService<T> {
    client: Arc<T>,
}

impl<T> ReggieService<T> {
    pub fn new(client: T) -> ReggieService<T> {
        ReggieService {
            client: Arc::new(client),
        }
    }
}

impl<T, B> Service<Request<B>> for ReggieService<T>
where
    T: HttpClient<B> + Send + Sync + 'static,
    for<'a> T::Future<'a>: Send,
    B: Send + 'static,
{
    type Response = Response<T::Body>;

    type Error = Error;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let client = self.client.clone();
        Box::pin(async move { client.send(req).await })
    }
}
