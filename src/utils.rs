#![allow(clippy::wrong_self_convention)]
pub mod encoding;
use core::str;
use std::{borrow::Cow, ops::Deref};

use quick_xml::{
    escape::{resolve_predefined_entity, resolve_xml_entity},
    events::{BytesPI, BytesRef, BytesText},
    name::{LocalName, QName},
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
        #[cfg(feature = "tracing")]
        tracing::debug!("Unescaping: {}", value);
        let unescape =
            crate::utils::encoding::unescape_with(value.as_str(), resolve_predefined_entity)?;
        #[cfg(feature = "tracing")]
        tracing::debug!("Unescaped: {}", unescape);
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
    fn unescape_to_string(&self) -> Result<String, EditXMLError> {
        bytes_to_unescaped_string(self.deref())
    }
}
impl XMLStringUtils for BytesRef<'_>{
    fn escape_ascii_into_string(&self) -> Result<String, EditXMLError> {
        Ok(self.escape_ascii().to_string())
    }

    fn into_string(&self) -> Result<String, EditXMLError> {
        String::from_utf8(self.to_vec()).map_err(EditXMLError::from)
    }
    fn unescape_to_string(&self) -> Result<String, EditXMLError> {
        let decoded =self.decode()?;
     Ok(resolve_xml_entity(decoded.as_ref()).unwrap().to_owned())
    }
}
impl XMLStringUtils for QName<'_> {
    fn escape_ascii_into_string(&self) -> Result<String, EditXMLError> {
        self.into_string()
    }
    #[inline(always)]
    fn into_string(&self) -> Result<String, EditXMLError> {
        String::from_utf8(self.0.to_vec()).map_err(EditXMLError::from)
    }

    fn unescape_to_string(&self) -> Result<String, EditXMLError> {
        bytes_to_unescaped_string(self.0)
    }
}
impl XMLStringUtils for LocalName<'_> {
    fn escape_ascii_into_string(&self) -> Result<String, EditXMLError> {
        self.into_string()
    }

    fn into_string(&self) -> Result<String, EditXMLError> {
        let bytes = self.into_inner().to_vec();
        String::from_utf8(bytes).map_err(EditXMLError::from)
    }

    fn unescape_to_string(&self) -> Result<String, EditXMLError> {
        bytes_to_unescaped_string(self.into_inner())
    }
}
impl XMLStringUtils for BytesPI<'_> {
    fn escape_ascii_into_string(&self) -> Result<String, EditXMLError> {
        self.into_string()
    }

    fn into_string(&self) -> Result<String, EditXMLError> {
        Ok(String::from_utf8(self.to_vec())?)
    }

    fn unescape_to_string(&self) -> Result<String, EditXMLError> {
        bytes_to_unescaped_string(self.content())
    }
}
pub(crate) fn resolve_ref(cow: &[u8]) -> Result<Cow<str>, EditXMLError> {
    let value = str::from_utf8(cow).map_err(EditXMLError::from)?;

    let unescape = crate::utils::encoding::unescape_with(value, resolve_predefined_entity)?;
    Ok(unescape)
}

pub(crate) fn bytes_to_unescaped_string(cow: &[u8]) -> Result<String, EditXMLError> {
    let value = str::from_utf8(cow).map_err(EditXMLError::from)?;

    let unescape = crate::utils::encoding::unescape_with(value, resolve_predefined_entity)?;
    Ok(unescape.into_owned())
}

pub(crate) fn bytes_owned_to_unescaped_string(bytes: Vec<u8>) -> Result<String, EditXMLError> {
    let value = String::from_utf8(bytes).map_err(EditXMLError::from)?;

    let unescape = crate::utils::encoding::unescape_with(&value, resolve_predefined_entity)?;
    // Check if the unescaped string is borrowed
    if let Cow::Borrowed(_) = unescape {
        return Ok(value);
    }
    Ok(unescape.into_owned())
}

pub mod attributes {
    pub static XMLNS: &str = "xmlns";
    pub static XMLNS_BYTES: &[u8] = b"xmlns";
    #[inline(always)]
    pub fn is_xlmns(bytes: impl AsRef<[u8]>) -> bool {
        let bytes = bytes.as_ref();
        bytes == XMLNS_BYTES || bytes.starts_with(XMLNS_BYTES)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_is_xmlns() {
            assert!(is_xlmns(b"xmlns"));
            assert!(is_xlmns(b"xmlns:"));
            assert!(!is_xlmns(b"xmln"));
            assert!(!is_xlmns(b"xml"));
        }
    }
}
#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use std::sync::Once;
    use tracing::{debug, info};
    use tracing_subscriber::fmt;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    pub fn setup_logger() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            let stdout_log = fmt::layer().pretty();
            tracing_subscriber::registry().with(stdout_log).init();
        });
        info!("Logger initialized");
        debug!("Logger initialized");
    }
    pub fn test_dir() -> std::path::PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")
    }
    pub fn documents_dir() -> std::path::PathBuf {
        test_dir().join("documents")
    }
}
