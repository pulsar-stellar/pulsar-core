//! The error returned when raw bytes are not decodable as an `ScVal`.
//!
//! Decoding separates two failures that the wire taxonomy deliberately keeps
//! apart. Bytes that are not parseable XDR are a [`DecodeError`]: nothing can be
//! said about them, not even their type. A value that parses but names a variant
//! this decoder cannot represent is *not* an error; it degrades to
//! [`DecodedValue::Unknown`](crate::DecodedValue::Unknown), carrying its XDR
//! intact. So the only way to reach this type is unparseable input.
//!
//! The error is hand-written rather than derived through a macro crate: it has
//! two shapes, both wrapping the underlying `stellar_xdr` error as their
//! [`source`](std::error::Error::source), and that is small enough to spell out.

use std::fmt;

use stellar_xdr::curr::Error as XdrError;

/// A failure to decode base64 XDR into an `ScVal`.
///
/// Both variants wrap the underlying [`stellar_xdr`] parse error as their
/// [`source`](std::error::Error::source), so a caller can inspect the cause
/// while the [`Display`](fmt::Display) message stays self-contained.
#[derive(Debug)]
pub enum DecodeError {
    /// A single value's bytes were not valid `ScVal` XDR.
    InvalidXdr(XdrError),
    /// A topic in an event's topic list was not valid `ScVal` XDR. Carries the
    /// zero-based index of the offending topic, so a caller decoding a list
    /// learns which entry failed rather than only that one did.
    InvalidTopic {
        /// The zero-based position of the topic that failed to parse.
        index: usize,
        /// The underlying `stellar_xdr` parse error.
        source: XdrError,
    },
}

impl DecodeError {
    /// Rewrites a single-value error as a topic error at `index`.
    ///
    /// Used by [`Decoder::decode_topics`](crate::Decoder::decode_topics) so a
    /// failure raised while decoding one topic points at its position in the
    /// list. A value already carrying a topic index is left unchanged.
    #[must_use]
    pub(crate) fn in_topic(self, index: usize) -> Self {
        match self {
            DecodeError::InvalidXdr(source) => DecodeError::InvalidTopic { index, source },
            already @ DecodeError::InvalidTopic { .. } => already,
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::InvalidXdr(source) => {
                write!(f, "value is not valid ScVal XDR: {source}")
            }
            DecodeError::InvalidTopic { index, source } => {
                write!(f, "topic {index} is not valid ScVal XDR: {source}")
            }
        }
    }
}

impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DecodeError::InvalidXdr(source) | DecodeError::InvalidTopic { source, .. } => {
                Some(source)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DecodeError;
    use std::error::Error as _;

    /// Produces a real `stellar_xdr` parse error to wrap, rather than
    /// constructing one by hand, so the source is exactly what decoding yields.
    fn xdr_error() -> stellar_xdr::curr::Error {
        use stellar_xdr::curr::{Limits, ReadXdr, ScVal};
        ScVal::from_xdr_base64("!!!! not base64 !!!!", Limits::none())
            .expect_err("garbage is not valid base64 XDR")
    }

    #[test]
    fn invalid_xdr_message_is_self_contained() {
        let err = DecodeError::InvalidXdr(xdr_error());
        let message = err.to_string();
        assert!(
            message.starts_with("value is not valid ScVal XDR: "),
            "message was {message:?}"
        );
    }

    #[test]
    fn invalid_topic_names_its_index() {
        let err = DecodeError::InvalidTopic {
            index: 3,
            source: xdr_error(),
        };
        assert!(
            err.to_string()
                .starts_with("topic 3 is not valid ScVal XDR: "),
            "message was {:?}",
            err.to_string()
        );
    }

    #[test]
    fn in_topic_relabels_a_single_value_error() {
        let relabeled = DecodeError::InvalidXdr(xdr_error()).in_topic(2);
        match relabeled {
            DecodeError::InvalidTopic { index, .. } => assert_eq!(index, 2),
            DecodeError::InvalidXdr(_) => panic!("in_topic must relabel a single-value error"),
        }
    }

    #[test]
    fn in_topic_leaves_an_existing_topic_index_alone() {
        let already = DecodeError::InvalidTopic {
            index: 5,
            source: xdr_error(),
        };
        match already.in_topic(9) {
            DecodeError::InvalidTopic { index, .. } => assert_eq!(index, 5, "index must be kept"),
            DecodeError::InvalidXdr(_) => panic!("a topic error must stay a topic error"),
        }
    }

    #[test]
    fn source_exposes_the_underlying_xdr_error() {
        let err = DecodeError::InvalidXdr(xdr_error());
        assert!(
            err.source().is_some(),
            "the xdr parse error must be reachable"
        );
    }
}
