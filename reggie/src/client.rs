use crate::{body::Body, error::Error};
use bytes::Bytes;
use core::marker::PhantomData;
use futures_core::{future::BoxFuture, Future};
use std::sync::Arc;

pub trait HttpClientFactory {
    type Client<B>: HttpClient<B>
    where
        B: http_body::Body + Send + 'static,
        B::Data: Into<Bytes> + Send,
        B::Error: Into<Error> + Send;
    fn create<B>(&self) -> Self::Client<B>
    where
        B: http_body::Body + Send + 'static,
        B::Data: Into<Bytes> + Send,
        B::Error: Into<Error> + Send;
}

pub trait HttpClient<B> {
    type Body;
    type Future<'a>: Future<Output = Result<http::Response<Self::Body>, Error>>
    where
        Self: 'a;
    fn send<'a>(&'a self, request: http::Request<B>) -> Self::Future<'a>;

    fn boxed(self) -> BoxClient<B>
    where
        Self: Sized + Send + Sync + 'static,
        B: http_body::Body + Send + 'static,
        Self::Body: Into<Body>,
        for<'a> Self::Future<'a>: Send,
    {
        Box::new(BoxedClient {
            client: self,
            body: PhantomData,
        })
    }

    fn shared(self) -> SharedClient<B>
    where
        Self: Sized + Send + Sync + 'static,
        B: http_body::Body + Send + 'static,
        Self::Body: Into<Body>,
        for<'a> Self::Future<'a>: Send,
    {
        Arc::new(BoxedClient {
            client: self,
            body: PhantomData,
        })
    }
}

pub trait DynamicClient<B> {
    type Body;
    fn send<'a>(
        &'a self,
        request: http::Request<B>,
    ) -> BoxFuture<'a, Result<http::Response<Self::Body>, Error>>;
}

impl<B> HttpClient<B> for Box<dyn DynamicClient<B, Body = Body> + Send + Sync>
where
    B: 'static,
{
    type Body = Body;
    type Future<'a> = BoxFuture<'a, Result<http::Response<Self::Body>, Error>>;
    fn send<'a>(&'a self, request: http::Request<B>) -> Self::Future<'a> {
        (**self).send(request)
    }
}

impl<B> HttpClient<B> for Arc<dyn DynamicClient<B, Body = Body> + Send + Sync>
where
    B: 'static,
{
    type Body = Body;

    type Future<'a> = BoxFuture<'a, Result<http::Response<Self::Body>, Error>>;

    fn send<'a>(
        &'a self,
        request: http::Request<B>,
    ) -> BoxFuture<'a, Result<http::Response<Self::Body>, Error>> {
        (**self).send(request)
    }
}

// pub trait HttpClientExt<B>: HttpClient<B> {
//     fn boxed(self) -> BoxClient<B>
//     where
//         Self: Sized + Send + Sync + 'static,
//         B: http_body::Body + Send + 'static,
//         Self::Body: Into<Body>,
//         for<'a> Self::Future<'a>: Send,
//     {
//         Box::new(BoxedClient {
//             client: self,
//             body: PhantomData,
//         })
//     }

//     fn shared(self) -> SharedClient<B>
//     where
//         Self: Sized + Send + Sync + 'static,
//         B: http_body::Body + Send + 'static,
//         Self::Body: Into<Body>,
//         for<'a> Self::Future<'a>: Send,
//     {
//         Arc::new(BoxedClient {
//             client: self,
//             body: PhantomData,
//         })
//     }
// }

// impl<T, B> HttpClientExt<B> for T where T: HttpClient<B> {}

pub type BoxClient<B> = Box<dyn DynamicClient<B, Body = Body> + Send + Sync>;
pub type SharedClient<B> = Arc<dyn DynamicClient<B, Body = Body> + Send + Sync>;

struct BoxedClient<T, B> {
    client: T,
    body: PhantomData<B>,
}

unsafe impl<T: Send, B> Send for BoxedClient<T, B> {}

unsafe impl<T: Sync, B> Sync for BoxedClient<T, B> {}

impl<T, B> DynamicClient<B> for BoxedClient<T, B>
where
    B: http_body::Body + Send,
    T: HttpClient<B> + Send + Sync,
    for<'a> T::Future<'a>: Send,
    T::Body: Into<Body>,
{
    type Body = Body;
    fn send<'a>(
        &'a self,
        request: http::Request<B>,
    ) -> BoxFuture<'a, Result<http::Response<Self::Body>, Error>> {
        Box::pin(async move { Ok(self.client.send(request).await?.map(Into::into)) })
    }
}
