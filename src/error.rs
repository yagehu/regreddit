use std::error;
use std::fmt;
use std::intrinsics::write_bytes;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    repr: Repr,
}

enum Repr {
    Simple(ErrorKind),
    Custom(Box<Custom>),
}

#[derive(Debug)]
struct Custom {
    kind: ErrorKind,
    error: Box<dyn error::Error + Send + Sync>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
    Authentication,
    InvalidInput,
    Network,
    Reddit,
}

impl Error {
    pub fn new<E>(kind: ErrorKind, err: E) -> Error
    where
        E: Into<Box<dyn error::Error + Send + Sync>>,
    {
        Error {
            repr: Repr::Custom(Box::new(Custom {
                kind,
                error: err.into(),
            })),
        }
    }

    // pub fn kind(&self) -> ErrorKind {
    //     match self.repr {
    //         Repr::Simple(kind) => kind,
    //         Repr::Custom(ref c) => c.kind,
    //     }
    // }
}

impl ErrorKind {
    pub(crate) fn as_str(&self) -> &'static str {
        match *self {
            ErrorKind::Authentication => "could not authenticate",
            ErrorKind::InvalidInput => "invalid input",
            ErrorKind::Network => "network error",
            ErrorKind::Reddit => "Reddit error",
        }
    }
}

impl From<ErrorKind> for Error {
    /// Converts an [`ErrorKind`] into an [`Error`].
    ///
    /// This conversion allocates a new error with a simple representation of
    /// error kind.
    #[inline]
    fn from(kind: ErrorKind) -> Error {
        Error {
            repr: Repr::Simple(kind),
        }
    }
}

impl fmt::Debug for Repr {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Repr::Simple(kind) => fmt.debug_tuple("Kind").field(&kind).finish(),
            Repr::Custom(ref c) => c.error.fmt(fmt),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.repr {
            Repr::Custom(ref c) => c.error.fmt(fmt),
            Repr::Simple(kind) => write!(fmt, "{}", kind.as_str()),
        }
    }
}

// impl error::Error for Error {
//     fn description(&self) -> &str {
//         match self.repr {
//             Repr::Simple(..) => self.kind().as_str(),
//             Repr::Custom(ref c) => c.error.description(),
//         }
//     }
// }
