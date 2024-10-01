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
        let value = String::from_utf8(value.to_vec())?.to_lowercase();
        match value.as_str() {
            "yes" => Ok(StandaloneValue::Yes),
            "no" => Ok(StandaloneValue::No),
            _ => Err(MalformedReason::InvalidStandAloneValue.into()),
        }
    }
}
impl TryFrom<&str> for StandaloneValue {
    type Error = EditXMLError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.to_lowercase();
        match value.as_str() {
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
        assert_eq!(yes.is_standalone(), true);
        assert_eq!(no.is_standalone(), false);
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
