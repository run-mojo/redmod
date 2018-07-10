use std;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum RedError {
    Generic(GenericError),
    FromUtf8(std::string::FromUtf8Error),
    ParseInt(std::num::ParseIntError),
}

impl RedError {
    pub fn generic(message: &str) -> RedError {
        RedError::Generic(GenericError::new(message))
    }
}

impl From<std::string::FromUtf8Error> for RedError {
    fn from(err: std::string::FromUtf8Error) -> RedError {
        RedError::FromUtf8(err)
    }
}

impl From<std::num::ParseIntError> for RedError {
    fn from(err: std::num::ParseIntError) -> RedError {
        RedError::ParseInt(err)
    }
}

impl fmt::Display for RedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // Both underlying errors already impl `Display`, so we defer to
            // their implementations.
            RedError::Generic(ref err) => write!(f, "{}", err),
            RedError::FromUtf8(ref err) => write!(f, "{}", err),
            RedError::ParseInt(ref err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for RedError {
    fn description(&self) -> &str {
        // Both underlying errors already impl `Error`, so we defer to their
        // implementations.
        match *self {
            RedError::Generic(ref err) => err.description(),
            RedError::FromUtf8(ref err) => err.description(),
            RedError::ParseInt(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            // N.B. Both of these implicitly cast `err` from their concrete
            // types (either `&io::Error` or `&num::ParseIntError`)
            // to a trait object `&Error`. This works because both error types
            // implement `Error`.
            RedError::Generic(ref err) => Some(err),
            RedError::FromUtf8(ref err) => Some(err),
            RedError::ParseInt(ref err) => Some(err),
        }
    }
}

#[derive(Debug)]
pub struct GenericError {
    message: String,
}

impl GenericError {
    pub fn new(message: &str) -> GenericError {
        GenericError {
            message: String::from(message),
        }
    }
}

impl<'a> fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Store error: {}", self.message)
    }
}

impl<'a> error::Error for GenericError {
    fn description(&self) -> &str {
        self.message.as_str()
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

