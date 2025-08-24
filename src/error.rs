use quick_xml::{encoding::EncodingError, escape::EscapeError, events::attributes::AttrError, Error as XMLError};
use std::{str::Utf8Error, string::FromUtf8Error, sync::Arc};
use thiserror::Error;

/// Wrapper around `std::Result`
pub type Result<T> = std::result::Result<T, EditXMLError>;
#[derive(Debug, Error)]
pub enum MalformedReason {
    #[error("Missing XML Declaration. Expected `<?xml version=\"1.0\" encoding=\"UTF-8\"?>`")]
    MissingDeclaration,
    #[error("Unexpected Item {0}")]
    UnexpectedItem(&'static str),
    #[error("Malformed Element Tree")]
    GenericMalformedTree,
    #[error("Standalone Document should be yes or no")]
    InvalidStandAloneValue,
    #[error("Missing closing tag")]
    MissingClosingTag,
}
/// Error types
#[derive(Debug, Error)]
pub enum EditXMLError {
    /// [`std::io`] related error.
    #[error("IO Error: {0}")]
    Io(#[from] Arc<std::io::Error>),
    /// Decoding related error.
    /// Maybe the XML declaration has an encoding value that it doesn't recognize,
    /// or it doesn't match its actual encoding,
    #[error(transparent)]
    CannotDecode(#[from] DecodeError),
    #[error("Cannot find entity for {0}")]
    EncodingError(#[from] EncodingError),
    /// Assorted errors while parsing XML.
    #[error("Malformed XML: {0}")]
    MalformedXML(#[from] MalformedReason),
    /// The container element cannot have a parent.
    /// Use `element.is_container()` to check if it is a container before
    /// assigning it to another parent.
    #[error("Container element cannot move")]
    ContainerCannotMove,
    /// You need to call `element.detach()` before assigning another parent.
    #[error("Element already has a parent. Call detach() before changing parent.")]
    HasAParent,
    #[error("Attribute Error {0}")]
    AttrError(#[from] AttrError),
    #[error("{0}")]
    OtherXML(#[source] XMLError),
}

impl From<XMLError> for EditXMLError {
    fn from(err: XMLError) -> EditXMLError {
        match err {
            //XMLError::EndEventMismatch { expected, found } => Error::MalformedXML(format!(
            //    "Closing tag mismatch. Expected {}, found {}",
            //    expected, found,
            //)),
            XMLError::Io(err) => EditXMLError::Io(err),
            // TODO XMLError::(_) => Error::CannotDecode,
            err => EditXMLError::OtherXML(err),
        }
    }
}
macro_rules! decode_error_proxy {
    (
        $($ty:ty),*
    ) => {
        $(
            impl From<$ty> for EditXMLError {
                fn from(error: $ty) -> EditXMLError {
                    DecodeError::from(error).into()
                }
            }
        )*
    };
}
decode_error_proxy!(Utf8Error, FromUtf8Error, EscapeError);

impl From<std::io::Error> for EditXMLError {
    fn from(err: std::io::Error) -> EditXMLError {
        EditXMLError::Io(Arc::new(err))
    }
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("Cannot decode String from UTF-8 {0}")]
    UTF8(#[from] Utf8Error),
    #[error("Cannot decode String from UTF-8 {0}")]
    FromUTF8(#[from] FromUtf8Error),
    #[error("Cannot decode XML")]
    Other,
    #[error("Cannot decode XML")]
    MissingEncoding,
    #[error(transparent)]
    EscapeError(#[from] EscapeError),
}
