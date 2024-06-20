mod body;
mod client;
mod error;
mod impls;
mod response_ext;
// mod service;

pub use http::{HeaderMap, HeaderName, HeaderValue, Method, Request, Response};
use std::sync::Arc;

#[allow(unused_imports)]
pub use self::{body::Body, client::*, error::Error, impls::*, response_ext::ResponseExt};
pub use bytes;
pub use http;
pub use http_body;
pub use http_body_util;

#[derive(Clone)]
pub struct Client {
    inner: SharedClient<Body>,
}

impl Client {
    pub fn new<T: HttpClient<Body> + Send + Sync + 'static>(client: T) -> Client
    where
        T::Body: Into<Body>,
    {
        Client {
            inner: client.shared(),
        }
    }
}

impl Client {
    pub async fn request<B: Into<Body>>(&self, req: Request<B>) -> Result<Response<Body>, Error> {
        self.inner.send(req.map(Into::into)).await
    }
}

pub trait ClientFactory {
    fn create(&self) -> Client;
}

impl ClientFactory for Box<dyn ClientFactory + Send + Sync> {
    fn create(&self) -> Client {
        (**self).create()
    }
}

impl ClientFactory for Arc<dyn ClientFactory + Send + Sync> {
    fn create(&self) -> Client {
        (**self).create()
    }
}

struct BoxedClientFactory<T>(T);

impl<T> ClientFactory for BoxedClientFactory<T>
where
    T: HttpClientFactory,
    T::Client<Body>: Send + Sync + 'static,
    <T::Client<Body> as HttpClient<Body>>::Body: Into<Body>,
{
    fn create(&self) -> Client {
        Client::new(self.0.create())
    }
}

pub fn factory_box<T>(factory: T) -> BoxClientFactory
where
    T: HttpClientFactory + Send + Sync + 'static,
    T::Client<Body>: Send + Sync + 'static,
    <T::Client<Body> as HttpClient<Body>>::Body: Into<Body>,
{
    Box::new(BoxedClientFactory(factory))
}

pub fn factory_arc<T>(factory: T) -> SharedClientFactory
where
    T: HttpClientFactory + Send + Sync + 'static,
    T::Client<Body>: Send + Sync + 'static,
    <T::Client<Body> as HttpClient<Body>>::Body: Into<Body>,
{
    Arc::new(BoxedClientFactory(factory))
}

pub type BoxClientFactory = Box<dyn ClientFactory + Send + Sync>;

pub type SharedClientFactory = Arc<dyn ClientFactory + Send + Sync>;
