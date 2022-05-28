use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

pub enum ImageError {
    InvalidImage,
    InvalidFormat,
    FailedWrite,
}

impl ImageError {
    fn message(&self) -> &str {
        match self {
            Self::InvalidImage => "Invalid Image",
            Self::InvalidFormat => "Invalid Format For Image",
            Self::FailedWrite => "Failed To Write Image",
        }
    }
}

impl Display for ImageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Debug for ImageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}
