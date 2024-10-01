use std::borrow::Cow;

use quick_xml::escape::{EscapeError, ParseCharRefError};

#[inline]
fn from_str_radix(src: &str, radix: u32) -> Result<u32, ParseCharRefError> {
    match src.as_bytes().first().copied() {
        // We should not allow sign numbers, but u32::from_str_radix will accept `+`.
        // We also handle `-` to be consistent in returned errors
        Some(b'+') | Some(b'-') => Err(ParseCharRefError::UnexpectedSign),
        _ => u32::from_str_radix(src, radix).map_err(ParseCharRefError::InvalidNumber),
    }
}
fn parse_number(num: &str) -> Result<char, ParseCharRefError> {
    let code = if let Some(hex) = num.strip_prefix('x') {
        from_str_radix(hex, 16)?
    } else {
        from_str_radix(num, 10)?
    };
    if code == 0 {
        return Err(ParseCharRefError::IllegalCharacter(code));
    }
    match std::char::from_u32(code) {
        Some(c) => Ok(c),
        None => Err(ParseCharRefError::InvalidCodepoint(code)),
    }
}
/// Will unescape the given string and ignore any unknown entities
pub fn unescape_with_and_ignore<'input, 'entity, F>(
    raw: &'input str,
    mut resolve_entity: F,
) -> Result<Cow<'input, str>, EscapeError>
where
    // the lifetime of the output comes from a capture or is `'static`
    F: FnMut(&str) -> Option<&'entity str>,
{
    let bytes = raw.as_bytes();
    let mut unescaped = None;
    let mut last_end = 0;
    let mut iter = memchr::Memchr2::new(b'&', b';', bytes);
    while let Some(start) = iter.by_ref().find(|p| bytes[*p] == b'&') {
        match iter.next() {
            Some(end) if bytes[end] == b';' => {
                // append valid data
                if unescaped.is_none() {
                    unescaped = Some(String::with_capacity(raw.len()));
                }
                let unescaped = unescaped.as_mut().expect("initialized");
                unescaped.push_str(&raw[last_end..start]);

                // search for character correctness
                let pat = &raw[start + 1..end];
                if let Some(entity) = pat.strip_prefix('#') {
                    let codepoint = parse_number(entity).map_err(EscapeError::InvalidCharRef)?;
                    unescaped.push_str(codepoint.encode_utf8(&mut [0u8; 4]));
                } else if let Some(value) = resolve_entity(pat) {
                    unescaped.push_str(value);
                } else {
                    tracing::warn!("Unknown entity: {:?}", pat);
                    unescaped.push_str(&raw[start..=end]);
                }

                last_end = end + 1;
            }
            _ => return Err(EscapeError::UnterminatedEntity(start..raw.len())),
        }
    }

    if let Some(mut unescaped) = unescaped {
        if let Some(raw) = raw.get(last_end..) {
            unescaped.push_str(raw);
        }
        Ok(Cow::Owned(unescaped))
    } else {
        Ok(Cow::Borrowed(raw))
    }
}
pub fn unescape_with<'input, 'entity, F>(
    raw: &'input str,
    resolve_entity: F,
) -> Result<Cow<'input, str>, EscapeError>
where
    // the lifetime of the output comes from a capture or is `'static`
    F: FnMut(&str) -> Option<&'entity str>,
{
    #[cfg(feature = "soft-fail-unescape")]
    {
        unescape_with_and_ignore(raw, resolve_entity)
    }
    #[cfg(not(feature = "soft-fail-unescape"))]
    {
        quick_xml::escape::unescape_with(raw, resolve_entity)
    }
}

#[cfg(test)]
mod tests {

    #[cfg(any(feature = "soft-fail-unescape", feature = "escape-html"))]
    #[test]
    fn oslash() -> anyhow::Result<()> {
        use anyhow::Context;

        use crate::{utils::tests, Document, ReadOptions};
        use std::fs::read_to_string;
        tests::setup_logger();
        let file_path = tests::test_dir()
            .join("bugs")
            .join("oslash")
            .join("oslash.xml");
        if !file_path.exists() {
            anyhow::bail!("File not found: {:?}", file_path);
        }
        let file = read_to_string(file_path).context("Failed to read file")?;

        let doc = Document::parse_str_with_opts(&file, ReadOptions::relaxed()).unwrap();
        let root = doc.root_element().context("Root Element not found")?;
        let developers = root
            .find(&doc, "developers")
            .context("Developers Element not found")?;

        for children in developers.children(&doc) {
            println!("{:#?}", children.debug(&doc));
        }
        println!("Parse Successful");
        Ok(())
    }
}
