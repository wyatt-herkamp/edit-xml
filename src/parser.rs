use crate::document::{Document, Node};
use crate::element::Element;
use crate::error::{DecodeError, EditXMLError, MalformedReason, Result};
use crate::types::StandaloneValue;
use crate::utils::{attributes, bytes_owned_to_unescaped_string, HashMap};
use crate::utils::{bytes_to_unescaped_string, XMLStringUtils};
use encoding_rs::Decoder;
use encoding_rs::{Encoding, UTF_16BE, UTF_16LE, UTF_8};
use quick_xml::events::{BytesDecl, BytesStart, Event};
use quick_xml::Reader;
use std::io::{BufRead, Read};

pub(crate) struct DecodeReader<R: Read> {
    decoder: Option<Decoder>,
    inner: R,
    undecoded: Box<[u8]>,
    undecoded_pos: usize,
    undecoded_cap: usize,
    remaining: [u8; 32], // Is there an encoding with > 32 bytes for a char?
    decoded: Box<[u8]>,
    decoded_pos: usize,
    decoded_cap: usize,
    done: bool,
}

impl<R: Read> DecodeReader<R> {
    // If Decoder is not set, don't decode.
    pub(crate) fn new(reader: R, decoder: Option<Decoder>) -> DecodeReader<R> {
        DecodeReader {
            decoder,
            inner: reader,
            undecoded: vec![0; 4096].into_boxed_slice(),
            undecoded_pos: 0,
            undecoded_cap: 0,
            remaining: [0; 32],
            decoded: vec![0; 12288].into_boxed_slice(),
            decoded_pos: 0,
            decoded_cap: 0,
            done: false,
        }
    }

    pub(crate) fn set_encoding(&mut self, encoding: Option<&'static Encoding>) {
        self.decoder = encoding.map(|e| e.new_decoder_without_bom_handling());
        self.done = false;
    }

    // Call this only when decoder is Some
    fn fill_buf_decode(&mut self) -> std::io::Result<&[u8]> {
        if self.decoded_pos >= self.decoded_cap {
            debug_assert!(self.decoded_pos == self.decoded_cap);
            if self.done {
                return Ok(&[]);
            }
            let remaining = self.undecoded_cap - self.undecoded_pos;
            if remaining <= 32 {
                // Move remaining undecoded bytes at the end to start
                self.remaining[..remaining]
                    .copy_from_slice(&self.undecoded[self.undecoded_pos..self.undecoded_cap]);
                self.undecoded[..remaining].copy_from_slice(&self.remaining[..remaining]);
                // Fill undecoded buffer
                let read = self.inner.read(&mut self.undecoded[remaining..])?;
                self.done = read == 0;
                self.undecoded_pos = 0;
                self.undecoded_cap = remaining + read;
            }

            // Fill decoded buffer
            let (_res, read, written, _replaced) = self.decoder.as_mut().unwrap().decode_to_utf8(
                &self.undecoded[self.undecoded_pos..self.undecoded_cap],
                &mut self.decoded,
                self.done,
            );
            self.undecoded_pos += read;
            self.decoded_cap = written;
            self.decoded_pos = 0;
        }
        Ok(&self.decoded[self.decoded_pos..self.decoded_cap])
    }

    fn fill_buf_without_decode(&mut self) -> std::io::Result<&[u8]> {
        if self.undecoded_pos >= self.undecoded_cap {
            debug_assert!(self.undecoded_pos == self.undecoded_cap);
            self.undecoded_cap = self.inner.read(&mut self.undecoded)?;
            self.undecoded_pos = 0;
        }
        Ok(&self.undecoded[self.undecoded_pos..self.undecoded_cap])
    }
}

impl<R: Read> Read for DecodeReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        (&self.decoded[..]).read(buf)
    }
}

impl<R: Read> BufRead for DecodeReader<R> {
    // Decoder may change from None to Some.
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        match &self.decoder {
            Some(_) => self.fill_buf_decode(),
            None => self.fill_buf_without_decode(),
        }
    }
    fn consume(&mut self, amt: usize) {
        match &self.decoder {
            Some(_) => {
                self.decoded_pos = std::cmp::min(self.decoded_pos + amt, self.decoded_cap);
            }
            None => {
                self.undecoded_pos = std::cmp::min(self.undecoded_pos + amt, self.undecoded_cap);
            }
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadOptionsOptimizations {
    pub reader_stack_initial_capacity: usize,
    pub document_initial_capacity: usize,
    pub attribute_initial_capacity: usize,
    pub namespace_initial_capacity: usize,
    pub children_initial_capacity: usize,
    pub parse_content_buffer_initial_capacity: usize,
}
impl Default for ReadOptionsOptimizations {
    fn default() -> Self {
        ReadOptionsOptimizations {
            reader_stack_initial_capacity: 10,
            document_initial_capacity: 100,
            attribute_initial_capacity: 20,
            namespace_initial_capacity: 20,
            children_initial_capacity: 1,
            parse_content_buffer_initial_capacity: 512,
        }
    }
}
/// Options when parsing xml.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadOptions {
    /// `<tag></tag>` will have a Node::Text("") as its children, while `<tag />` won't.
    /// Default: `true`
    pub empty_text_node: bool,
    /// Trims leading and ending whitespaces in `Node::Text`, and ignore node if it is empty.
    /// Default: `true`
    pub trim_text: bool,
    /// Ignore Node::Text that only has whitespaces.
    /// Only makes sense if `trim_text` is `false`. (If both are `true`, performance takes a hit for no gain)
    /// Default: `false`
    pub ignore_whitespace_only: bool,
    /// Returns error if document doesn't start with XML declaration.
    /// If there is no XML declaration, the parser won't be able to decode encodings other than UTF-8, unless `encoding` below is set.
    /// Default: `true`
    pub require_decl: bool,
    /// If this is set, the parser will start reading with this encoding.
    /// But it will switch to XML declaration's encoding value if it has a different value.
    /// See [`encoding_rs::Encoding::for_label`] for valid values.
    /// Default: `None`
    pub encoding: Option<String>,

    pub normalize_attribute_value_space: bool,

    pub optimizations: ReadOptionsOptimizations,
}
impl ReadOptions {
    /// New ReadOptions that is relaxed by not requiring XML declaration.
    pub fn relaxed() -> Self {
        ReadOptions {
            empty_text_node: true,
            trim_text: true,
            ignore_whitespace_only: true,
            require_decl: false,
            encoding: None,
            normalize_attribute_value_space: false,
            optimizations: ReadOptionsOptimizations::default(),
        }
    }
}
impl Default for ReadOptions {
    fn default() -> Self {
        ReadOptions {
            empty_text_node: true,
            trim_text: true,
            ignore_whitespace_only: false,
            require_decl: true,
            encoding: None,
            normalize_attribute_value_space: false,
            optimizations: ReadOptionsOptimizations::default(),
        }
    }
}

//TODO: don't unwrap element_stack.last() or pop(). Invalid XML file can crash the software.
pub(crate) struct DocumentParser {
    doc: Document,
    read_opts: ReadOptions,
    encoding: Option<&'static Encoding>,
    element_stack: Vec<Element>,
}

impl DocumentParser {
    pub(crate) fn parse_reader<R: Read>(reader: R, opts: ReadOptions) -> Result<Document> {
        let doc = Document::new_with_store_size(opts.optimizations.document_initial_capacity);
        let mut element_stack =
            Vec::with_capacity(opts.optimizations.reader_stack_initial_capacity);
        element_stack.push(Element::container().0);
        let mut parser = DocumentParser {
            doc,
            read_opts: opts,
            encoding: None,
            element_stack,
        };
        parser.parse_start(reader)?;
        Ok(parser.doc)
    }

    fn handle_decl(&mut self, ev: &BytesDecl) -> Result<()> {
        self.doc.version = String::from_utf8(ev.version()?.to_vec())?;
        self.encoding = match ev.encoding() {
            Some(res) => {
                let encoding = Encoding::for_label(&res?).ok_or(DecodeError::MissingEncoding)?;
                if encoding == UTF_8 {
                    None
                } else {
                    Some(encoding)
                }
            }
            None => None,
        };
        self.doc.standalone = match ev.standalone() {
            Some(res) => {
                let standalone_value = res?;
                Some(StandaloneValue::try_from(standalone_value.as_ref())?)
            }
            None => None,
        };
        Ok(())
    }
    #[inline(always)]
    fn element_attributes(
        &self,
        ev: &BytesStart,
    ) -> Result<(HashMap<String, String>, HashMap<String, String>)> {
        let mut attributes =
            HashMap::with_capacity(self.read_opts.optimizations.attribute_initial_capacity);
        let mut namespace_decls =
            HashMap::with_capacity(self.read_opts.optimizations.namespace_initial_capacity);

        for attr in ev.attributes() {
            let attr = attr?;
            // Key is converted to string.
            let (key, prefix) = attr.key.decompose();
            let value = if self.read_opts.normalize_attribute_value_space {
                let value = normalize_space(&attr.value);
                bytes_owned_to_unescaped_string(value)?
            } else {
                bytes_to_unescaped_string(&attr.value)?
            };

            if prefix.map(attributes::is_xlmns).unwrap_or(false) {
                // Has a prefix of `xmlns` so it is going in
                let key = key.into_string()?;
                namespace_decls.insert(key, value);
            } else if attributes::is_xlmns(key) {
                // The attribute is just `xmlns` meaning it is empty string
                namespace_decls.insert(String::default(), value);
            } else {
                let key = attr.key.into_string()?;
                attributes.insert(key, value);
            }
        }
        Ok((attributes, namespace_decls))
    }
    /// Create a new element and push it to the parent element.
    fn create_element(&mut self, parent: Element, ev: &BytesStart) -> Result<Element> {
        let full_name = ev.name().into_string()?;
        let (attributes, namespace_decls) = self.element_attributes(ev)?;
        let elem = Element::with_data(&mut self.doc, full_name, attributes, namespace_decls);
        parent
            .push_child(&mut self.doc, Node::Element(elem))
            .unwrap();
        Ok(elem)
    }

    // Returns true if document parsing is finished.
    fn handle_event(&mut self, event: Event) -> Result<bool> {
        match event {
            Event::Start(ref ev) => {
                let parent = *self.element_stack.last().ok_or({
                    EditXMLError::MalformedXML(MalformedReason::GenericMalformedTree)
                })?;
                let element = self.create_element(parent, ev)?;
                self.element_stack.push(element);
                Ok(false)
            }
            Event::End(_) => {
                let elem = self.element_stack.pop().ok_or({
                    EditXMLError::MalformedXML(MalformedReason::GenericMalformedTree)
                })?;
                // quick-xml checks if tag names match for us
                if self.read_opts.empty_text_node {
                    // distinguish <tag></tag> and <tag />
                    if !elem.has_children(&self.doc) {
                        elem.push_child(&mut self.doc, Node::Text(String::new()))
                            .unwrap();
                    }
                }
                Ok(false)
            }
            Event::Empty(ref ev) => {
                let parent = *self.element_stack.last().ok_or({
                    EditXMLError::MalformedXML(MalformedReason::GenericMalformedTree)
                })?;
                self.create_element(parent, ev)?;
                Ok(false)
            }
            // Comment, CData, and PI content should not be escaped,
            // but quick-xml assumes only CDATA is not escaped.
            Event::Text(ev) => {
                if self.read_opts.ignore_whitespace_only && only_has_whitespace(&ev) {
                    return Ok(false);
                }
                // when trim_text, ignore_whitespace_only, empty_text_node are all false
                if ev.is_empty() {
                    return Ok(false);
                }
                // NOTE: Was Unescaped
                let content = ev.unescape_to_string()?;
                let node = Node::Text(content);
                let parent = *self.element_stack.last().ok_or({
                    EditXMLError::MalformedXML(MalformedReason::GenericMalformedTree)
                })?;
                parent.push_child(&mut self.doc, node).unwrap();
                Ok(false)
            }
            Event::DocType(ev) => {
                // Event::DocType comes with one leading whitespace. Strip the whitespace.
                let raw = ev.unescape_to_string()?.into_bytes();
                let content = if !raw.is_empty() && raw[0] == b' ' {
                    String::from_utf8(raw[1..].to_vec())?
                } else {
                    String::from_utf8(raw.to_vec())?
                };
                let node = Node::DocType(content);
                let parent = *self.element_stack.last().ok_or({
                    EditXMLError::MalformedXML(MalformedReason::GenericMalformedTree)
                })?;
                parent.push_child(&mut self.doc, node).unwrap();
                Ok(false)
            }
            Event::Comment(ev) => {
                let content = String::from_utf8(ev.escape_ascii().collect())?;
                let node = Node::Comment(content);
                let parent = *self.element_stack.last().ok_or({
                    EditXMLError::MalformedXML(MalformedReason::GenericMalformedTree)
                })?;
                parent.push_child(&mut self.doc, node).unwrap();
                Ok(false)
            }
            Event::CData(ev) => {
                let content = String::from_utf8(ev.to_vec())?;
                let node = Node::CData(content);
                let parent = *self.element_stack.last().ok_or(EditXMLError::MalformedXML(
                    MalformedReason::GenericMalformedTree,
                ))?;
                parent.push_child(&mut self.doc, node).unwrap();
                Ok(false)
            }
            Event::PI(ev) => {
                let content = ev.into_string()?;
                let node = Node::PI(content);
                let parent = *self.element_stack.last().ok_or(EditXMLError::MalformedXML(
                    MalformedReason::GenericMalformedTree,
                ))?;
                parent.push_child(&mut self.doc, node).unwrap();
                Ok(false)
            }
            Event::Decl(_) => Err(EditXMLError::MalformedXML(MalformedReason::UnexpectedItem(
                "XML Declaration",
            ))),
            Event::Eof => Ok(true),
        }
    }

    // Sniff encoding and consume BOM
    fn sniff_encoding<R: Read>(
        &mut self,
        decodereader: &mut DecodeReader<R>,
    ) -> Result<Option<&'static Encoding>> {
        let bytes = decodereader.fill_buf()?;
        let encoding = match bytes {
            [0x3c, 0x3f, ..] => None, // UTF-8 '<?'
            [0xfe, 0xff, ..] => {
                // UTF-16 BE BOM
                decodereader.consume(2);
                Some(UTF_16BE)
            }
            [0xff, 0xfe, ..] => {
                // UTF-16 LE BOM
                decodereader.consume(2);
                Some(UTF_16LE)
            }
            [0xef, 0xbb, 0xbf, ..] => {
                // UTF-8 BOM
                decodereader.consume(3);
                None
            }
            [0x00, 0x3c, 0x00, 0x3f, ..] => Some(UTF_16BE),
            [0x3c, 0x00, 0x3f, 0x00, ..] => Some(UTF_16LE),
            _ => None, // Try decoding it with UTF-8
        };
        Ok(encoding)
    }

    // Look at the document decl and figure out the document encoding
    fn parse_start<R: Read>(&mut self, reader: R) -> Result<()> {
        #[cfg(feature = "tracing")]
        tracing::debug!(?self.read_opts, "Parsing Start");
        let mut decodereader = DecodeReader::new(reader, None);
        let mut init_encoding = self.sniff_encoding(&mut decodereader)?;
        if let Some(enc) = &self.read_opts.encoding {
            init_encoding =
                Some(Encoding::for_label(enc.as_bytes()).ok_or(DecodeError::MissingEncoding)?)
        }
        #[cfg(feature = "tracing")]
        tracing::debug!(?init_encoding, "Initial Encoding");
        decodereader.set_encoding(init_encoding);
        let mut xmlreader = Reader::from_reader(decodereader);
        xmlreader.config_mut().trim_text(self.read_opts.trim_text);

        let mut buf = Vec::with_capacity(200);

        // Skip first event if it only has whitespace
        let event = match xmlreader.read_event_into(&mut buf)? {
            Event::Text(ev) => {
                if ev.len() == 0 {
                    #[cfg(feature = "tracing")]
                    tracing::trace!("Skipping empty text event");
                    xmlreader.read_event_into(&mut buf)?
                } else if self.read_opts.ignore_whitespace_only && only_has_whitespace(&ev) {
                    #[cfg(feature = "tracing")]
                    tracing::trace!("Skipping whitespace only text event");
                    xmlreader.read_event_into(&mut buf)?
                } else {
                    #[cfg(feature = "tracing")]

                    tracing::trace!("First Event is Text");
                    Event::Text(ev)
                }
            }
            ev => ev,
        };
        #[cfg(feature = "tracing")]
        tracing::debug!(?event, "First Event");
        if let Event::Decl(ev) = event {
            self.handle_decl(&ev)?;
            // Encoding::for_label("UTF-16") defaults to UTF-16 LE, even though it could be UTF-16 BE
            if self.encoding != init_encoding
                && !(self.encoding == Some(UTF_16LE) && init_encoding == Some(UTF_16BE))
            {
                let mut decode_reader = xmlreader.into_inner();
                decode_reader.set_encoding(self.encoding);
                xmlreader = Reader::from_reader(decode_reader);
                xmlreader.config_mut().trim_text(self.read_opts.trim_text);
            }
        } else if self.read_opts.require_decl {
            #[cfg(feature = "tracing")]
            tracing::debug!(?self.read_opts, ?event, "XML Declaration is required");
            return Err(MalformedReason::MissingDeclaration.into());
        } else if self.handle_event(event)? {
            return Ok(());
        }
        // Handle rest of the events
        self.parse_content(xmlreader)
    }

    fn parse_content<B: BufRead>(&mut self, mut reader: Reader<B>) -> Result<()> {
        let mut buf = Vec::with_capacity(
            self.read_opts
                .optimizations
                .parse_content_buffer_initial_capacity,
        ); // reduce time increasing capacity at start.

        loop {
            let ev = reader.read_event_into(&mut buf)?;

            if self.handle_event(ev)? {
                if self.element_stack.len() == 1 {
                    // Should only have container remaining in element_stack
                    return Ok(());
                } else {
                    return Err(MalformedReason::MissingClosingTag.into());
                }
            }
        }
    }
}

/// Returns true if byte is an XML whitespace character
#[allow(clippy::match_like_matches_macro)]
#[inline(always)]
fn is_whitespace(byte: u8) -> bool {
    match byte {
        b'\r' | b'\n' | b'\t' | b' ' => true,
        _ => false,
    }
}

/// Returns true if bytes.len() == 0 or bytes only has a whitespace-like character.
fn only_has_whitespace(bytes: &[u8]) -> bool {
    bytes.iter().all(|b| is_whitespace(*b))
}

/// #xD(\r), #xA(\n), #x9(\t) is normalized into #x20.
/// Leading and trailing spaces(#x20) are discarded
/// and sequence of spaces are replaced by a single space.
pub fn normalize_space(bytes: &[u8]) -> Vec<u8> {
    let mut normalized = Vec::with_capacity(bytes.len());
    let mut char_found = false;
    let mut last_space = false;
    for &byte in bytes {
        if is_whitespace(byte) {
            if char_found && !last_space {
                normalized.push(b' ');
                last_space = true;
            }
        } else {
            char_found = true;
            last_space = false;
            normalized.push(byte);
        }
    }
    // There can't be multiple whitespaces
    if normalized.last() == Some(&b' ') {
        normalized.pop();
    }
    normalized
}
