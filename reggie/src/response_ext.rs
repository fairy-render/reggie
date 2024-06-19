use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures_core::future::BoxFuture;
use http::Response;

use crate::Error;

mod internal {
    use http::Response;

    pub trait Internal {}

    impl<B> Internal for Response<B> {}
}

// TODO: No boxing
pub trait ResponseExt<B>: internal::Internal {
    fn text(self) -> BoxFuture<'static, Result<String, Error>>;
    fn bytes(self) -> BoxFuture<'static, Result<Bytes, Error>>;
    #[cfg(feature = "json")]
    fn json<T: serde::de::DeserializeOwned>(self) -> BoxFuture<'static, Result<T, Error>>;
    fn bytes_stream(self) -> DataStream<B>;
}

impl<B> ResponseExt<B> for Response<B>
where
    B: http_body::Body + Send + 'static,
    B::Error: Into<Error>,
    B::Data: Send,
{
    fn text(self) -> BoxFuture<'static, Result<String, Error>> {
        Box::pin(async move {
            use http_body_util::BodyExt;

            let bytes = BodyExt::collect(self.into_body())
                .await
                .map(|buf| buf.to_bytes())
                .map_err(Into::into)?;

            String::from_utf8(bytes.to_vec()).map_err(|err| Error::Body(Box::new(err)))
        })
    }

    fn bytes(self) -> BoxFuture<'static, Result<Bytes, Error>> {
        Box::pin(async move {
            use http_body_util::BodyExt;

            BodyExt::collect(self.into_body())
                .await
                .map(|buf| buf.to_bytes())
                .map_err(Into::into)
        })
    }

    #[cfg(feature = "json")]
    fn json<T: serde::de::DeserializeOwned>(self) -> BoxFuture<'static, Result<T, Error>> {
        Box::pin(async move {
            use http_body_util::BodyExt;

            let bytes = BodyExt::collect(self.into_body())
                .await
                .map(|buf| buf.to_bytes())
                .map_err(Into::into)?;

            serde_json::from_slice::<T>(&bytes).map_err(|err| Error::Body(Box::new(err)))
        })
    }

    fn bytes_stream(self) -> DataStream<B> {
        DataStream(self.into_body())
    }
}

pub struct DataStream<T>(T);

impl<B> futures_core::Stream for DataStream<B>
where
    B: http_body::Body<Data = Bytes> + Unpin,
{
    type Item = Result<Bytes, B::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        loop {
            return match futures_core::ready!(Pin::new(&mut self.0).poll_frame(cx)) {
                Some(Ok(frame)) => {
                    // skip non-data frames
                    if let Ok(buf) = frame.into_data() {
                        Poll::Ready(Some(Ok(buf)))
                    } else {
                        continue;
                    }
                }
                Some(Err(err)) => Poll::Ready(Some(Err(err))),
                None => Poll::Ready(None),
            };
        }
    }
}
