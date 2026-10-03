#[derive(Debug, derive_more::Display)]
/// Error that unifies different error types used by Hyprland-rs
#[non_exhaustive]
pub enum HyprError {
    /// Error coming from serde
    SerdeError(serde_json::Error),
    /// Error coming from std::io
    IoError(io::Error),
    /// Error from failing to parse HyprColor
    InvalidHyprColorFormat,
    /// Error from failing to parse HyprGradient
    InvalidHyprGradientFormat,
    /// Error that occurs when parsing UTF-8 string
    FromUtf8Error(std::string::FromUtf8Error),
    /// Dispatcher returned non `ok` value
    #[display("A dispatcher returned a non-`ok`, value which is probably an error: {_0}")]
    NotOkDispatch(String),
    /// Error when interacting with Hyprpaper.
    #[cfg(feature = "hyprpaper")]
    Hyprpaper(crate::hyprpaper::Error),
    /// Keyword does not exist
    InvalidOptionKey(String),
    /// Unparsable Option
    InvalidOptionValue,
    /// Internal Hyprland error
    Internal(String),
    /// Error that occurs for other reasons. Avoid using this.
    Other(String),
}
impl HyprError {
    /// Try to get an owned version of the internal error.
    ///
    /// Some dependencies of hyprland do not impl Clone in their error types. This is a partial workaround.
    ///
    /// If it succeeds, it returns the owned version of HyprError in Ok(). Otherwise, it returns a reference to the error type.
    pub fn try_as_cloned(&self) -> Result<Self, &Self> {
        match self {
            Self::SerdeError(_) => Err(self),
            Self::IoError(_) => Err(self),
            Self::FromUtf8Error(e) => Ok(Self::FromUtf8Error(e.clone())),
            Self::NotOkDispatch(s) => Ok(Self::NotOkDispatch(s.clone())),
            Self::InvalidHyprColorFormat => Ok(Self::InvalidHyprColorFormat),
            Self::InvalidHyprGradientFormat => Ok(Self::InvalidHyprGradientFormat),
            #[cfg(feature = "hyprpaper")]
            Self::Hyprpaper(_) => Err(self),
            Self::InvalidOptionKey(key) => Ok(Self::InvalidOptionKey(key.clone())),
            Self::InvalidOptionValue => Ok(Self::InvalidOptionValue),
            Self::Internal(s) => Ok(Self::Internal(s.clone())),
            Self::Other(s) => Ok(Self::Other(s.clone())),
        }
    }
    /// Create a Hyprland error with dynamic data.
    #[inline(always)]
    pub fn other<S: Into<String>>(other: S) -> Self {
        Self::Other(other.into())
    }
}

impl From<io::Error> for HyprError {
    fn from(error: io::Error) -> Self {
        HyprError::IoError(error)
    }
}

impl From<serde_json::Error> for HyprError {
    fn from(error: serde_json::Error) -> Self {
        HyprError::SerdeError(error)
    }
}

impl From<std::string::FromUtf8Error> for HyprError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        HyprError::FromUtf8Error(error)
    }
}

impl error::Error for HyprError {
    /// Was left as the default `None`, so `serde_json::Error` and `io::Error`
    /// lost their cause chain and `anyhow`-style reporting printed only the
    /// wrapper. The unit variants have no source, so they return `None`.
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            HyprError::SerdeError(e) => Some(e),
            HyprError::IoError(e) => Some(e),
            HyprError::FromUtf8Error(e) => Some(e),
            #[cfg(feature = "hyprpaper")]
            HyprError::Hyprpaper(e) => Some(e),
            _ => None,
        }
    }
}

/// Hand-written because two of the wrapped types are not `PartialEq`:
/// `serde_json::Error` and `std::io::Error` both impl `Error` but not `Eq`.
/// This was written by hand rather than derived for exactly that reason.
impl PartialEq for HyprError {
    fn eq(&self, other: &Self) -> bool {
        use HyprError::*;
        match (self, other) {
            (SerdeError(a), SerdeError(b)) => a.to_string() == b.to_string(),
            (IoError(a), IoError(b)) => a.kind() == b.kind() && a.to_string() == b.to_string(),
            (FromUtf8Error(a), FromUtf8Error(b)) => a == b,
            #[cfg(feature = "hyprpaper")]
            (Hyprpaper(a), Hyprpaper(b)) => a == b,

            (InvalidHyprColorFormat, InvalidHyprColorFormat)
            | (InvalidHyprGradientFormat, InvalidHyprGradientFormat)
            | (InvalidOptionValue, InvalidOptionValue) => true,

            (NotOkDispatch(a), NotOkDispatch(b))
            | (InvalidOptionKey(a), InvalidOptionKey(b))
            | (Internal(a), Internal(b))
            | (Other(a), Other(b)) => a == b,

            _ => false,
        }
    }
}

impl Eq for HyprError {}

/// Internal macro to return a Hyprland error
macro_rules! hypr_err {
    ($fmt:literal) => {
        return Err($crate::error::HyprError::Internal(format!($fmt)))
    };
    (other $fmt:literal) => {
        return Err($crate::error::HyprError::Other(format!($fmt)))
    };
    ($fmt:literal $(, $value:expr)+) => {
        return Err($crate::error::HyprError::Internal(format!($fmt $(, $value)+)))
    };
    (other $fmt:literal $(, $value:expr)+) => {
        return Err($crate::error::HyprError::Other(format!($fmt $(, $value)+)))
    };
}

pub(crate) use hypr_err;
use std::{error, io};

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    /// The point of `PartialEq`: assert equality directly instead of `matches!`.
    #[test]
    fn errors_compare_by_value() {
        assert_eq!(HyprError::other("boom"), HyprError::other("boom"));
        assert_ne!(HyprError::other("boom"), HyprError::other("bang"));
        assert_eq!(
            HyprError::NotOkDispatch("no".into()),
            HyprError::NotOkDispatch("no".into())
        );
        assert_eq!(HyprError::InvalidOptionValue, HyprError::InvalidOptionValue);
    }

    /// Different variants are never equal, even with identical payloads.
    #[test]
    fn different_variants_are_unequal() {
        assert_ne!(HyprError::other("x"), HyprError::Internal("x".into()));
        assert_ne!(
            HyprError::InvalidOptionKey("k".into()),
            HyprError::Other("k".into())
        );
    }

    #[test]
    fn io_errors_compare_by_kind_and_message() {
        let a = HyprError::IoError(io::Error::new(io::ErrorKind::NotFound, "gone"));
        let b = HyprError::IoError(io::Error::new(io::ErrorKind::NotFound, "gone"));
        let c = HyprError::IoError(io::Error::new(io::ErrorKind::PermissionDenied, "gone"));
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    /// `Error::source()` used the default `None`, so the wrapped
    /// `serde_json::Error` / `io::Error` / `FromUtf8Error` never surfaced.
    #[test]
    fn wrapped_errors_expose_their_source() {
        fn source_of(e: &HyprError) -> Option<String> {
            use std::error::Error;
            e.source().map(|s| s.to_string())
        }

        let io = HyprError::from(io::Error::new(io::ErrorKind::NotFound, "no such file"));
        assert_eq!(source_of(&io).as_deref(), Some("no such file"));

        let serde: HyprError = serde_json::from_str::<i32>("not json").unwrap_err().into();
        assert!(source_of(&serde).is_some(), "serde error lost its source");

        let utf8: HyprError = String::from_utf8(vec![0xff]).unwrap_err().into();
        assert!(source_of(&utf8).is_some(), "utf8 error lost its source");
    }

    /// Unit and string-only variants have nothing to point at.
    #[test]
    fn variants_without_a_wrapped_error_have_no_source() {
        use std::error::Error;
        for e in [
            HyprError::InvalidOptionValue,
            HyprError::InvalidHyprGradientFormat,
            HyprError::other("boom"),
            HyprError::Internal("i".into()),
        ] {
            assert!(e.source().is_none(), "unexpected source for {e:?}");
        }
    }

    /// The rename from `InvalidHyprGradiantFormat` must not leave stragglers.
    #[test]
    fn the_gradient_variant_is_spelled_correctly() {
        let e = HyprError::InvalidHyprGradientFormat;
        assert!(e.to_string().contains("Gradient"), "got {e}");
    }

    #[test]
    fn utf8_errors_compare() {
        let bad = || std::string::String::from_utf8(vec![0xff]).unwrap_err();
        assert_eq!(HyprError::from(bad()), HyprError::from(bad()));
    }
}
