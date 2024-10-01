#![allow(clippy::wrong_self_convention)]
use quick_xml::{
    events::{BytesPI, BytesText},
    name::QName,
};
use tracing::debug;

use crate::EditXMLError;
#[cfg(not(feature = "ahash"))]
pub type HashMap<K, V> = std::collections::HashMap<K, V>;
#[cfg(feature = "ahash")]
pub type HashMap<K, V> = ahash::AHashMap<K, V>;
/// Trait for converting quick-xml types to string
pub trait XMLStringUtils {
    /// Escapes non-ascii characters into their escape sequences
    fn escape_ascii_into_string(&self) -> Result<String, EditXMLError>;
    /// Converts the type into a string

    fn into_string(&self) -> Result<String, EditXMLError>;
    /// Unescapes the content of the type into a string
    fn unescape_to_string(&self) -> Result<String, EditXMLError> {
        let value = self.into_string()?;
        debug!("Unescaping: {}", value);
        let unescape = quick_xml::escape::unescape(&value)?;
        Ok(unescape.into_owned())
    }
}

impl XMLStringUtils for BytesText<'_> {
    fn escape_ascii_into_string(&self) -> Result<String, EditXMLError> {
        Ok(self.escape_ascii().to_string())
    }

    fn into_string(&self) -> Result<String, EditXMLError> {
        String::from_utf8(self.to_vec()).map_err(EditXMLError::from)
    }
}
impl XMLStringUtils for QName<'_> {
    fn escape_ascii_into_string(&self) -> Result<String, EditXMLError> {
        self.into_string()
    }

    fn into_string(&self) -> Result<String, EditXMLError> {
        String::from_utf8(self.0.to_vec()).map_err(EditXMLError::from)
    }
}
impl XMLStringUtils for BytesPI<'_> {
    fn escape_ascii_into_string(&self) -> Result<String, EditXMLError> {
        self.into_string()
    }

    fn into_string(&self) -> Result<String, EditXMLError> {
        Ok(String::from_utf8(self.to_vec())?)
    }
}
pub(crate) fn bytes_to_unescaped_string(cow: &[u8]) -> Result<String, EditXMLError> {
    let value = String::from_utf8(cow.to_vec())?;

    let unescape = quick_xml::escape::unescape(&value)?;
    Ok(unescape.into_owned())
}
