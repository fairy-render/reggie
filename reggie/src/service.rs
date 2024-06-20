use http::{Error, Request, Response};
use tower_service::Service;

use crate::{Body, HttpClient};

pub struct ReggieService<T> {
    client: T,
}

impl<T, B> Service<Request<B>> for ReggieService<T>
where
    T: HttpClient<B>,
{
    type Response = Response<Body>;

    type Error = Error;

    type Future = ();

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        todo!()
    }
}
