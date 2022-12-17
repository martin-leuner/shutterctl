pub mod motor;
pub mod rpc;
pub mod transport;

mod shuttercomm;
use shuttercomm::shutterheader;
use shuttercomm::shuttermsg;

use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    FB(flatbuffers::InvalidFlatbuffer),
    HeaderSize,
    BadMagic,
    PayloadSize,
    UnknownVersion,
    UnknownCrypto,
    CommandMissing,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => {
                write!(f, "{}", e)
            }
            Error::FB(e) => {
                write!(f, "{}", e)
            }
            Error::HeaderSize => {
                write!(f, "Message too short to contain protocol header")
            }
            Error::BadMagic => {
                write!(f, "Message does not start with magic number")
            }
            Error::PayloadSize => {
                write!(f, "Message too short to contain full payload")
            }
            Error::UnknownVersion => {
                write!(f, "Unknown protocol version")
            }
            Error::UnknownCrypto => {
                write!(f, "Unknown crypto mechanism")
            }
            Error::CommandMissing => {
                write!(f, "No command message enveloped in header")
            }
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<flatbuffers::InvalidFlatbuffer> for Error {
    fn from(e: flatbuffers::InvalidFlatbuffer) -> Self {
        Error::FB(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
