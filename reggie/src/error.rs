use core::fmt;

#[derive(Debug)]
pub enum Error {
    Connection(Box<dyn std::error::Error + Send + Sync>),
    Body(Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    pub fn conn<E: Into<Box<dyn std::error::Error + Send + Sync>>>(error: E) -> Error {
        Error::Connection(error.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection(err) => write!(f, "connection: {err}"),
            Self::Body(err) => write!(f, "body: {err}"),
        }
    }
}

impl std::error::Error for Error {}
