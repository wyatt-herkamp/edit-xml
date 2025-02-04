use crate::{error::MalformedReason, EditXMLError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StandaloneValue {
    Yes,
    #[default]
    No,
}

impl StandaloneValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            StandaloneValue::Yes => "yes",
            StandaloneValue::No => "no",
        }
    }
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            StandaloneValue::Yes => b"yes",
            StandaloneValue::No => b"no",
        }
    }
    pub fn is_standalone(&self) -> bool {
        match self {
            StandaloneValue::Yes => true,
            StandaloneValue::No => false,
        }
    }
}
impl TryFrom<&[u8]> for StandaloneValue {
    type Error = EditXMLError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"yes" => Ok(StandaloneValue::Yes),
            b"no" => Ok(StandaloneValue::No),
            _ => Err(MalformedReason::InvalidStandAloneValue.into()),
        }
    }
}
impl TryFrom<&str> for StandaloneValue {
    type Error = EditXMLError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "yes" => Ok(StandaloneValue::Yes),
            "no" => Ok(StandaloneValue::No),
            _ => Err(MalformedReason::InvalidStandAloneValue.into()),
        }
    }
}
impl std::fmt::Display for StandaloneValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
#[cfg(feature = "serde")]
mod serde_impl {
    use serde::{de::Visitor, Deserialize, Serialize};

    use super::StandaloneValue;
    impl Serialize for StandaloneValue {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(self.as_str())
        }
    }
    struct StandaloneValueVisitor;
    impl<'de> Visitor<'de> for StandaloneValueVisitor {
        type Value = StandaloneValue;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string representing a standalone value")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            StandaloneValue::try_from(value).map_err(serde::de::Error::custom)
        }
        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            StandaloneValue::try_from(v.as_str()).map_err(serde::de::Error::custom)
        }
        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            StandaloneValue::try_from(v).map_err(serde::de::Error::custom)
        }
        fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            StandaloneValue::try_from(v).map_err(serde::de::Error::custom)
        }
    }

    impl<'de> Deserialize<'de> for StandaloneValue {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_str(StandaloneValueVisitor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_standalone_value() {
        let yes = StandaloneValue::Yes;
        let no = StandaloneValue::No;
        assert_eq!(yes.as_str(), "yes");
        assert_eq!(no.as_str(), "no");
        assert_eq!(yes.as_bytes(), b"yes");
        assert_eq!(no.as_bytes(), b"no");
        assert!(yes.is_standalone());
        assert!(!no.is_standalone());
        let yes_str = "yes";
        let no_str = "no";
        assert_eq!(
            StandaloneValue::try_from(yes_str).unwrap(),
            StandaloneValue::Yes
        );
        assert_eq!(
            StandaloneValue::try_from(no_str).unwrap(),
            StandaloneValue::No
        );
    }
}
