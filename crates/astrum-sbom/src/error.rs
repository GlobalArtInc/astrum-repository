use crate::{MAX_DEPTH, MAX_INPUT_BYTES, MAX_PACKAGES};

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("the document is larger than {MAX_INPUT_BYTES} bytes")]
    TooLarge,
    #[error("the document nests deeper than {MAX_DEPTH} levels")]
    TooDeep,
    #[error("the document lists more than {MAX_PACKAGES} packages")]
    TooManyPackages,
    #[error("the document is malformed: {0}")]
    Malformed(String),
    #[error("the document is not supported: {0}")]
    Unsupported(String),
}

impl Error {
    pub fn category(&self) -> &'static str {
        match self {
            Error::TooLarge => "too_large",
            Error::TooDeep => "too_deep",
            Error::TooManyPackages => "too_many_packages",
            Error::Malformed(_) => "malformed",
            Error::Unsupported(_) => "unsupported",
        }
    }

    pub(crate) fn malformed(reason: &'static str) -> Self {
        Error::Malformed(reason.to_owned())
    }

    pub(crate) fn unsupported(reason: &'static str) -> Self {
        Error::Unsupported(reason.to_owned())
    }
}
