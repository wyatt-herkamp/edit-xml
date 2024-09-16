use quick_xml::{
    events::{BytesPI, BytesText},
    name::QName,
};

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
        let unescape = quick_xml::escape::unescape(&value)?;
        Ok(unescape.into_owned())
    }
}

impl XMLStringUtils for BytesText<'_> {
    fn escape_ascii_into_string(&self) -> Result<String, EditXMLError> {
        Ok(self.escape_ascii().to_string())
    }

    fn into_string(&self) -> Result<String, EditXMLError> {
        String::from_utf8(self.to_vec()).map_err(|err| EditXMLError::from(err))
    }
}
impl XMLStringUtils for QName<'_> {
    fn escape_ascii_into_string(&self) -> Result<String, EditXMLError> {
        self.into_string()
    }

    fn into_string(&self) -> Result<String, EditXMLError> {
        String::from_utf8(self.0.to_vec()).map_err(|err| EditXMLError::from(err))
    }
}
impl XMLStringUtils for BytesPI<'_> {
    fn escape_ascii_into_string(&self) -> Result<String, EditXMLError> {
        return self.into_string();
    }

    fn into_string(&self) -> Result<String, EditXMLError> {
        return Ok(String::from_utf8(self.to_vec())?);
    }
}
pub(crate) fn from_cow_bytes_to_string(
    cow: &std::borrow::Cow<'_, [u8]>,
) -> Result<String, EditXMLError> {
    let value = String::from_utf8(cow.to_vec())?;
    let unescape = quick_xml::escape::unescape(&value)?;
    Ok(unescape.into_owned())
}
