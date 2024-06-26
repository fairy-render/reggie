#[cfg(feature = "reqwest")]
mod reqw {
    use bytes::Bytes;

    use crate::{Body, DynamicClient, Error, HttpClient, HttpClientFactory};

    #[derive(Default, Clone)]
    pub struct Reqwest {}

    impl HttpClientFactory for Reqwest {
        type Client<B> = reqwest::Client
        where
            B: http_body::Body + Send + 'static,
            B::Data: Into<Bytes> + Send,
            B::Error: Into<Error> + Send;

        fn create<B>(&self) -> Self::Client<B>
        where
            B: http_body::Body + Send + 'static,
            B::Data: Into<Bytes> + Send,
            B::Error: Into<Error> + Send,
        {
            reqwest::Client::new()
        }
    }

    impl From<reqwest::Error> for Error {
        fn from(value: reqwest::Error) -> Self {
            Error::conn(value)
        }
    }

    impl From<reqwest::Body> for Body {
        fn from(value: reqwest::Body) -> Self {
            Body::from_streaming(value)
        }
    }

    impl<B> HttpClient<B> for reqwest::Client
    where
        B: http_body::Body + Send + 'static,
        B::Data: Into<Bytes> + Send,
        B::Error: Into<Error> + Send,
    {
        type Body = reqwest::Body;
        type Future<'a> =
            futures_core::future::BoxFuture<'a, Result<http::Response<Self::Body>, Error>>;
        fn send<'a>(
            &'a self,
            request: http::Request<B>,
        ) -> futures_core::future::BoxFuture<'a, Result<http::Response<Self::Body>, Error>>
        {
            use http_body_util::BodyExt;

            Box::pin(async move {
                let (parts, body) = request.into_parts();

                let output = body
                    .map_frame(|frame| frame.map_data(Into::into))
                    .collect()
                    .await
                    .map_err(Into::into)?
                    .to_bytes();

                let resp = self
                    .request(parts.method, parts.uri.to_string())
                    .headers(parts.headers)
                    .body(output)
                    .send()
                    .await?;

                Ok(resp.into())
            })
        }
    }

    impl<B> DynamicClient<B> for reqwest::Client
    where
        B: http_body::Body + Send + 'static,
        B::Data: Into<Bytes> + Send,
        B::Error: Into<Error> + Send,
    {
        type Body = reqwest::Body;

        fn send<'a>(
            &'a self,
            request: http::Request<B>,
        ) -> futures_core::future::BoxFuture<'a, Result<http::Response<Self::Body>, Error>>
        {
            <reqwest::Client as HttpClient<B>>::send(&self, request)
        }
    }
}

#[cfg(feature = "reqwest")]
pub use reqw::*;
