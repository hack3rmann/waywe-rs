use crate::xml::Description;

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct DocDescription<'s> {
    pub inner_summary: Option<&'s str>,
    pub outer_summary: Option<&'s str>,
    pub outer_description: Option<&'s str>,
}

impl<'s> DocDescription<'s> {
    pub fn from_inner_and_outer(
        inner: Option<&'s str>,
        outer: Option<&'s Description<'s>>,
    ) -> Self {
        Self {
            inner_summary: inner.filter(|s| !s.is_empty()),
            outer_summary: outer
                .and_then(|desc| desc.summary.as_deref())
                .filter(|s| !s.is_empty()),
            outer_description: outer
                .and_then(|desc| desc.body.as_deref())
                .filter(|s| !s.is_empty()),
        }
    }

    pub fn from_outer(value: Option<&'s Description<'s>>) -> Self {
        Self::from_inner_and_outer(None, value)
    }

    pub fn from_inner(value: Option<&'s str>) -> Self {
        Self::from_inner_and_outer(value, None)
    }
}

pub fn make_first_letter_uppercase(source: &mut str) {
    // Safety: `source` is still a valid UTF-8 string after making first byte
    // an ASCII uppercase character if it was a valid ASCII lowercase letter.
    if let Some(first_byte) = unsafe { source.as_bytes_mut() }.get_mut(0) {
        first_byte.make_ascii_uppercase();
    }
}

pub fn format_doc_string(desc: DocDescription<'_>) -> String {
    let mut summary = match (desc.inner_summary, desc.outer_summary) {
        (Some(inner), Some(outer)) => {
            let mut inner = remove_offsets(inner);
            let outer = remove_offsets(outer);

            Some(if inner == outer {
                inner
            } else {
                inner.reserve(1 + outer.len());
                inner.push('\n');
                inner.push_str(&outer);
                inner
            })
        }
        (Some(summary), None) | (None, Some(summary)) => Some(remove_offsets(summary)),
        (None, None) => None,
    };

    if let Some(summary) = &mut summary {
        make_first_letter_uppercase(summary);
    }

    let mut body = desc.outer_description.map(remove_offsets);

    if let Some(body) = &mut body {
        make_first_letter_uppercase(body);
    }

    match (summary, body) {
        (Some(mut summary), Some(body)) => {
            summary.reserve(2 + body.len());
            summary.push_str("\n\n");
            summary.push_str(&body);
            summary
        }
        (Some(body), None) | (None, Some(body)) => body,
        (None, None) => String::new(),
    }
}

pub fn remove_offsets(source: &str) -> String {
    let source = source.trim();

    let mut buf = Vec::<u8>::with_capacity(source.len());
    let mut removing_offset = false;

    for byte in source.bytes() {
        match (byte, removing_offset) {
            (b'\n', ..) => {
                removing_offset = true;
                buf.push(b'\n');
            }
            (b' ' | b'\t', false) => buf.push(b' '),
            (b' ' | b'\t', true) => continue,
            (byte, ..) => {
                removing_offset = false;
                buf.push(byte);
            }
        }
    }

    // Safety: removing ASCII characters from UTF-8 string produces another valid UTF-8 string
    unsafe { String::from_utf8_unchecked(buf) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_remove_offsets() {
        const SOURCE: &str = r"     This is test
            for removing offsets
            in strings
            like this
            ";

        const EXPECTATION: &str = r"This is test
for removing offsets
in strings
like this";

        assert_eq!(EXPECTATION, remove_offsets(SOURCE));
    }

    #[test]
    fn try_remove_offsets_in_empty_string() {
        assert_eq!("", remove_offsets(""));
    }
}
