//! # The Shared Module
//!
//! This module provides shared private and public functions, structs, enum, and types
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::{env, fmt};

/// The address struct holds a address as a tuple with a single value
/// and has methods to reveal the address in different data formats
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Display)]
pub struct Address(String);
impl Address {
    /// This creates a new address from a value that implements [ToString]
    pub fn new<T: ToString>(string: T) -> Self {
        let str = string.to_string();
        if str.starts_with("0x") {
            Self(str)
        } else {
            Self("0x".to_owned() + str.as_str())
        }
    }
}

impl AsRef<str> for Address {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for Address {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Address> for String {
    fn from(address: Address) -> Self {
        address.0
    }
}

impl From<&Address> for String {
    fn from(address: &Address) -> Self {
        address.0.clone()
    }
}

impl PartialEq<str> for Address {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for Address {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for Address {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}

/// This trait provides a standardized way to get data
pub trait HyprData {
    /// This method gets the data
    fn get() -> crate::Result<Self>
    where
        Self: Sized;
    /// This method gets the data (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn get_async() -> crate::Result<Self>
    where
        Self: Sized;
    /// This method gets the data
    fn instance_get(instance: &Instance) -> crate::Result<Self>
    where
        Self: Sized;
    /// This method gets the data (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn instance_get_async(instance: &Instance) -> crate::Result<Self>
    where
        Self: Sized;
}

/// This trait provides a standardized way to get data in a from of a vector
pub trait HyprDataVec<T>: HyprData {
    /// This method returns a vector of data
    fn to_vec(self) -> Vec<T>;
}

/// Trait for helper functions to get the active of the implementor
pub trait HyprDataActive {
    /// This method gets the active data
    fn get_active() -> crate::Result<Self>
    where
        Self: Sized;
    /// This method gets the active data (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn get_active_async() -> crate::Result<Self>
    where
        Self: Sized;
    /// This method gets the active data
    fn instance_get_active(instance: &Instance) -> crate::Result<Self>
    where
        Self: Sized;
    /// This method gets the active data (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn instance_get_active_async(instance: &Instance) -> crate::Result<Self>
    where
        Self: Sized;
}

/// Trait for helper functions to get the active of the implementor, but for optional ones
pub trait HyprDataActiveOptional {
    /// This method gets the active data
    fn get_active() -> crate::Result<Option<Self>>
    where
        Self: Sized;
    /// This method gets the active data (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn get_active_async() -> crate::Result<Option<Self>>
    where
        Self: Sized;
    /// This method gets the active data
    fn instance_get_active(instance: &Instance) -> crate::Result<Option<Self>>
    where
        Self: Sized;
    /// This method gets the active data (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn instance_get_active_async(instance: &Instance) -> crate::Result<Option<Self>>
    where
        Self: Sized;
}

/// This type provides the id used to identify workspaces
/// > its a type because it might change at some point
pub type WorkspaceId = i32;

/// This type provides the id used to identify monitors
/// > its a type because it might change at some point
pub type MonitorId = i128;

#[inline]
fn ser_spec_opt(opt: &Option<String>) -> String {
    match opt {
        Some(name) => "special:".to_owned() + name,
        None => "special".to_owned(),
    }
}

/// This enum holds workspace data
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum WorkspaceType {
    /// A named workspace
    Regular(
        /// The name
        String,
    ),
    /// The special workspace
    Special(
        /// The name, if exists
        Option<String>,
    ),
}

impl fmt::Display for WorkspaceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkspaceType::Regular(name) => f.write_str(name),
            WorkspaceType::Special(opt) => f.write_str(&ser_spec_opt(opt)),
        }
    }
}

impl Serialize for WorkspaceType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for WorkspaceType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;

        // `Regular` must not be tried first: with `#[serde(untagged)]` it matched
        // every string, so `special:magic` came back as `Regular("special:magic")`
        // and `Special` was unreachable. The `special` prefix is the discriminator.
        if s == "special" {
            return Ok(WorkspaceType::Special(None));
        }
        if let Some(name) = s.strip_prefix("special:") {
            return Ok(WorkspaceType::Special(Some(name.to_owned())));
        }
        Ok(WorkspaceType::Regular(s))
    }
}

impl From<&WorkspaceType> for String {
    fn from(value: &WorkspaceType) -> Self {
        value.to_string()
    }
}
macro_rules! from {
    ($($ty:ty),+$(,)?) => {
        $(
            impl TryFrom<$ty> for WorkspaceType {
                type Error = crate::error::HyprError;
                fn try_from(int: $ty) -> Result<Self, Self::Error> {
                    match int {
                        1.. => Ok(WorkspaceType::Regular(int.to_string())),
                        _ => crate::error::hypr_err!("Conversion error: Unrecognised id"),
                    }
                }
            }
        )+
    };
}
from![u8, u16, u32, u64, usize, i8, i16, i32, i64, isize];

impl Hash for WorkspaceType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            WorkspaceType::Regular(name) => name.hash(state),
            WorkspaceType::Special(value) => match value {
                Some(name) => name.hash(state),
                None => "".hash(state),
            },
        }
    }
}

pub(crate) fn get_hypr_path() -> crate::Result<PathBuf> {
    let mut buf = if let Some(runtime_path) = env::var_os("XDG_RUNTIME_DIR") {
        std::path::PathBuf::from(runtime_path)
    } else if let Ok(uid) = env::var("UID") {
        std::path::PathBuf::from("/run/user/".to_owned() + &uid)
    } else {
        hypr_err!("Could not find XDG_RUNTIME_DIR or UID");
    };
    buf.push("hypr");
    Ok(buf)
}

/// This enum defines the possible command flags that can be used.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CommandFlag {
    /// The JSON flag.
    #[default]
    JSON,
    /// An empty flag.
    Empty,
}

/// This struct defines the content of a command, which consists of a flag and a data string.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandContent {
    /// The flag for the command.
    pub flag: CommandFlag,
    /// The data string for the command.
    pub data: String,
}

impl CommandContent {
    /// Converts the command content to a byte vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyprland::shared::*;
    ///
    /// let content = CommandContent { flag: CommandFlag::JSON, data: "foo".to_string() };
    /// let bytes = content.as_bytes();
    /// assert_eq!(bytes, b"j/foo");
    /// ```
    pub fn as_bytes(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl fmt::Display for CommandContent {
    /// Formats the command content as a string for display.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyprland::shared::*;
    ///
    /// let content = CommandContent { flag: CommandFlag::JSON, data: "foo".to_string() };
    /// let s = format!("{}", content);
    /// assert_eq!(s, "j/foo");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.flag {
            CommandFlag::JSON => write!(f, "j/{}", self.data),
            CommandFlag::Empty => write!(f, "/{}", self.data),
        }
    }
}

/// Creates a `CommandContent` instance with the given flag and formatted data.
///
/// # Arguments
///
/// * `$flag` - A `CommandFlag` variant (`JSON` or `Empty`) that represents the flag for the command.
/// * `$($k:tt)*` - A format string and its arguments to be used as the data in the `CommandContent` instance.
#[macro_export]
macro_rules! command {
    ($flag:ident, $($k:tt)*) => {{
        $crate::shared::CommandContent {
            flag: $crate::shared::CommandFlag::$flag,
            data: format!($($k)*),
        }
    }};
}
use crate::error::hypr_err;
use crate::instance::Instance;
pub use command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
#[allow(missing_docs)]
/// Enum for mod keys used in bind combinations
#[non_exhaustive]
pub enum Mod {
    #[display("SUPER")]
    SUPER,
    #[display("SHIFT")]
    SHIFT,
    #[display("ALT")]
    ALT,
    #[display("CTRL")]
    CTRL,
    #[display("")]
    NONE,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn address_normalises_the_prefix() {
        assert_eq!(AsRef::<str>::as_ref(&Address::new("55a")), "0x55a");
        assert_eq!(AsRef::<str>::as_ref(&Address::new("0x55a")), "0x55a");
    }

    /// The traits added so consumers are not forced through `.as_str()`.
    #[test]
    fn address_supports_the_obvious_conversions() {
        let a = Address::new("55a");
        let s: &str = a.as_ref();
        assert_eq!(s, "0x55a");
        assert_eq!(String::from(a.clone()), "0x55a");
        assert_eq!(String::from(&a), "0x55a");
    }

    #[test]
    fn address_derefs_to_str() {
        let a = Address::new("55a");
        assert_eq!(a.len(), 5);
        assert!(a.starts_with("0x"));
    }

    #[test]
    fn address_compares_against_str_and_string() {
        let a = Address::new("55a");
        assert!(a == *"0x55a");
        assert!(a != *"0x55b");
        assert_eq!(a, "0x55a");
        assert_eq!(a, String::from("0x55a"));
    }
}

#[cfg(test)]
mod workspace_type_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn round_trip(s: &str) -> WorkspaceType {
        let json = format!("\"{s}\"");
        let parsed: WorkspaceType = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&parsed).unwrap(), json);
        parsed
    }

    /// `#[serde(untagged)]` tried `Regular` first and it matches every string,
    /// so `special:magic` deserialized as `Regular("special:magic")` and the
    /// `Special` variant was unreachable.
    #[test]
    fn special_with_a_name_is_not_regular() {
        assert_eq!(
            round_trip("special:magic"),
            WorkspaceType::Special(Some("magic".to_owned()))
        );
    }

    #[test]
    fn bare_special_is_special_with_no_name() {
        assert_eq!(round_trip("special"), WorkspaceType::Special(None));
    }

    #[test]
    fn a_named_special_workspace_round_trips() {
        assert_eq!(
            round_trip("special:audit-sp"),
            WorkspaceType::Special(Some("audit-sp".to_owned()))
        );
    }

    #[test]
    fn ordinary_workspaces_stay_regular() {
        for name in ["1", "example", "specialist", "my-special-thing", "spec"] {
            assert_eq!(
                round_trip(name),
                WorkspaceType::Regular(name.to_owned()),
                "{name} should be regular"
            );
        }
    }

    /// The prefix match must be exact, not a `contains`.
    #[test]
    fn only_the_exact_prefix_is_special() {
        assert_eq!(
            round_trip("not-special:x"),
            WorkspaceType::Regular("not-special:x".to_owned())
        );
    }

    #[test]
    fn display_matches_the_wire_form() {
        assert_eq!(WorkspaceType::Regular("1".into()).to_string(), "1");
        assert_eq!(WorkspaceType::Special(None).to_string(), "special");
        assert_eq!(
            WorkspaceType::Special(Some("m".into())).to_string(),
            "special:m"
        );
    }
}
