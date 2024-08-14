use http::Request;

mod internal {
    use http::Request;

    pub trait Internal {}

    impl<B> Internal for Request<B> {}
}

pub trait RequestExt<B>: internal::Internal {
    fn map_body<T: FnOnce(B) -> U, U>(self, func: T) -> Request<U>;
}

impl<B> RequestExt<B> for Request<B> {
    fn map_body<T: FnOnce(B) -> U, U>(self, func: T) -> Request<U> {
        let (parts, body) = self.into_parts();
        Request::from_parts(parts, func(body))
    }
}
