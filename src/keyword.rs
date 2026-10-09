//! # Keyword module
//!
//! This module is used for setting, getting and discovering keywords
//!
//! ## Usage
//!
//! ```rust, no_run
//! use hyprland::keyword::Keyword;
//! use hyprland::shared::HyprData;
//! fn main() -> hyprland::Result<()> {
//!     Keyword::get("some_keyword")?;
//!     Keyword::set("another_keyword", "the value to set it to")?;
//!     hyprland::keyword::OptionDescriptions::get()?;
//!
//!     Ok(())
//! }
//! ```

use crate::instance::Instance;
use crate::shared::*;
use crate::{default_instance, error::HyprError};
use derive_more::Display;
use serde::{Deserialize, Serialize};

/// A Color made up of rgba values (0-255)
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HyprColor {
    /// Red Channel (0-255)
    pub r: u8,
    /// Green Channel (0-255)
    pub g: u8,
    /// Blue Channel (0-255)
    pub b: u8,
    /// Alpha (0-255)
    pub a: u8,
}

impl std::fmt::Display for HyprColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "rgba({:02x}{:02x}{:02x}{:02x})",
            self.r, self.g, self.b, self.a
        ))
    }
}

/// A Gradient made up of HyprColor(s) and an angle
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HyprGradient {
    /// First gradiant color
    pub color0: HyprColor,
    /// Second gradiant color
    pub color1: Option<HyprColor>,
    /// Angle in degrees
    pub angle: u32,
}

impl std::fmt::Display for HyprGradient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (color1, space) = match &self.color1 {
            Some(s) => (s.to_string(), " "),
            None => (String::from(""), ""),
        };
        f.write_fmt(format_args!(
            "{}{}{} {}deg",
            self.color0, space, color1, self.angle
        ))
    }
}

/// Bounds used for custom Rects
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HyprRect {
    /// Bound Top
    pub top: i64,
    /// Bound Right
    pub right: i64,
    /// Bound Bottom
    pub bottom: i64,
    /// Bound Left
    pub left: i64,
}

impl std::fmt::Display for HyprRect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:0} {:0} {:0} {:0}",
            self.top, self.right, self.bottom, self.left
        ))
    }
}

impl TryFrom<&str> for HyprRect {
    type Error = crate::HyprError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut s = s.trim().split(" ");
        let top = (s.next().ok_or(crate::HyprError::InvalidOptionValue)?)
            .parse::<i64>()
            .map_err(|_| crate::HyprError::InvalidOptionValue)?;
        let right = (s.next().ok_or(crate::HyprError::InvalidOptionValue)?)
            .parse::<i64>()
            .map_err(|_| crate::HyprError::InvalidOptionValue)?;
        let bottom = (s.next().ok_or(crate::HyprError::InvalidOptionValue)?)
            .parse::<i64>()
            .map_err(|_| crate::HyprError::InvalidOptionValue)?;
        let left = (s.next().ok_or(crate::HyprError::InvalidOptionValue)?)
            .parse::<i64>()
            .map_err(|_| crate::HyprError::InvalidOptionValue)?;
        if s.next().is_some() {
            return Err(crate::error::HyprError::InvalidOptionValue);
        }

        Ok(Self {
            top,
            right,
            bottom,
            left,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Display)]
/// A parseable value type for custom options
#[non_exhaustive]
pub enum Custom {
    /// Color Variant for Custom field
    HyprColor(HyprColor),
    /// Gradient Variant for Custom field
    HyprGradient(HyprGradient),
    /// A general rect made of top, right, bottom, left
    HyprRect(HyprRect),
}

impl HyprColor {
    /// Convert an AARRGGBB u32 value to a HyprColor
    pub fn from_argb_u32(i: u32) -> Self {
        Self {
            a: (i >> 24) as u8,             // 0xFF000000
            r: ((i >> 16) & 0x00FF) as u8,  // 0x00FF0000
            g: ((i >> 8) & 0x0000FF) as u8, // 0x0000FF00
            b: (i & 0x000000FF) as u8,      // 0x000000FF
        }
    }

    /// Try to convert from &str in the argb legacy format to HyprColor,
    /// e.g. 0xeeb3ff1a
    pub fn try_from_argb_str(s: &str) -> Option<Self> {
        let s = s.trim();
        let s = s.strip_prefix("0x").unwrap_or(s);
        // Hyprland writes colours as 6 (rrggbb) or 8 (rrggbbaa) hex digits.
        // Any length was accepted, so `u32::from_str_radix("1", 16)` succeeded and
        // `Keyword::set("k", "1")` silently became a nearly-transparent black.
        if s.len() != 6 && s.len() != 8 {
            return None;
        }
        u32::from_str_radix(s, 16).ok().map(Self::from_argb_u32)
    }

    /// Try to convert from &str in the rgba format to HyprColor,
    /// e.g. rgba(b3ff1aee), or the decimal equivalent rgba(179,255,26,0.933)
    pub fn try_from_rgba_str(s: &str) -> Option<Self> {
        s.trim()
            .strip_prefix("rgba(")?
            .strip_suffix(")")
            .and_then(|s| match s.contains(",") {
                // b10 parse (e.g., "rgba(255, 0, 170, 0.5)")
                true => {
                    let mut parts = s.split(",").enumerate().map(|(i, t)| {
                        if i < 3 {
                            t.trim().parse::<u8>().ok()
                        } else {
                            let f: f32 = t.trim().parse().ok()?;
                            Some((f.clamp(0.0, 1.0) * 255.0).round() as u8)
                        }
                    });

                    let r = parts.next()?? as u32;
                    let g = parts.next()?? as u32;
                    let b = parts.next()?? as u32;
                    let a = parts.next()?? as u32;
                    if parts.next().is_some() {
                        return None;
                    }

                    let u: u32 = (a << 24) | (r << 16) | (g << 8) | b;
                    Some(Self::from_argb_u32(u))
                }
                // b16 parse — Hyprland writes 8 hex digits, e.g. "rgba(FF00AA7F)".
                // 6 digits is accepted too, and means opaque.
                false => {
                    let s = s.trim();
                    if s.len() != 6 && s.len() != 8 {
                        return None;
                    }
                    let i = u32::from_str_radix(s, 16).ok()?;
                    // 6 digits are rrggbb; Hyprland's own form is rrggbbaa.
                    let u = if s.len() == 6 {
                        (0xFFu32 << 24) | i
                    } else {
                        let a = (i & 0xFF) << 24;
                        (i >> 8) | a
                    };
                    Some(Self::from_argb_u32(u))
                }
            })
    }

    /// Try to convert from &str in the rgb format to HyprColor,
    /// e.g. rgb(b3ff1a), or the decimal equivalent rgb(179,255,26)
    pub fn try_from_rgb_str(s: &str) -> Option<Self> {
        s.trim()
            .strip_prefix("rgb(")?
            .strip_suffix(")")
            .and_then(|s| match s.contains(",") {
                // b10 parse (e.g., "rgb(255, 0, 170)")
                true => {
                    let mut parts = s.split(",").map(|t| t.trim().parse::<u8>().ok());
                    let r = parts.next()?? as u32;
                    let g = parts.next()?? as u32;
                    let b = parts.next()?? as u32;
                    if parts.next().is_some() {
                        return None;
                    }
                    let a = 0xFFu32;

                    let u: u32 = (a << 24) | (r << 16) | (g << 8) | b;
                    Some(Self::from_argb_u32(u))
                }
                // b16 parse (e.g., "rgb(ff00aa)")
                false => {
                    let s = s.trim();
                    if s.len() != 6 {
                        return None;
                    }
                    let i = u32::from_str_radix(s, 16).ok()?;
                    let a = 0xFF000000;
                    let i = i | a;
                    Some(Self::from_argb_u32(i))
                }
            })
    }
}

impl TryFrom<&str> for HyprColor {
    type Error = crate::HyprError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let s = s.trim().to_lowercase();
        if s.starts_with("rgba") {
            Self::try_from_rgba_str(&s).ok_or(crate::HyprError::InvalidHyprColorFormat)
        } else if s.starts_with("rgb") {
            Self::try_from_rgb_str(&s).ok_or(crate::HyprError::InvalidHyprColorFormat)
        } else {
            Self::try_from_argb_str(&s).ok_or(crate::HyprError::InvalidHyprColorFormat)
        }
    }
}

impl TryFrom<&str> for HyprGradient {
    type Error = crate::HyprError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut a = None;
        let s = s.trim().replace(", ", ",");
        let mut s = s.split(" ");
        let color0 = HyprColor::try_from(
            s.next()
                .ok_or(crate::HyprError::InvalidHyprGradientFormat)?,
        )?;
        let mut color1 = None;
        let c1_angle = s
            .next()
            .ok_or(crate::HyprError::InvalidHyprGradientFormat)?;
        if c1_angle.contains("deg") {
            let tmp = c1_angle
                .strip_suffix("deg")
                .ok_or(crate::HyprError::InvalidHyprGradientFormat)?;
            a = Some(
                tmp.parse::<u32>()
                    .map_err(|_| crate::HyprError::InvalidHyprGradientFormat)?,
            );
        } else {
            color1 = Some(HyprColor::try_from(c1_angle)?)
        }

        let angle = match a {
            Some(a) => a,
            None => {
                let c1_angle = s
                    .next()
                    .ok_or(crate::HyprError::InvalidHyprGradientFormat)?;
                let tmp = c1_angle
                    .strip_suffix("deg")
                    .ok_or(crate::HyprError::InvalidHyprGradientFormat)?;
                match tmp.parse::<u32>() {
                    Ok(i) => i,
                    Err(_) => return Err(crate::HyprError::InvalidHyprGradientFormat),
                }
            }
        };

        if s.next().is_some() {
            return Err(crate::HyprError::InvalidHyprGradientFormat);
        }

        Ok(HyprGradient {
            color0,
            color1,
            angle,
        })
    }
}

impl TryFrom<&str> for Custom {
    type Error = crate::HyprError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let c = s.trim().replace(", ", ",");
        let c = c.split(" ");
        let c = c.count();
        if c == 4 {
            Ok(Self::HyprRect(HyprRect::try_from(s)?))
        } else if c > 1 {
            Ok(Self::HyprGradient(HyprGradient::try_from(s)?))
        } else {
            Ok(Self::HyprColor(HyprColor::try_from(s)?))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct OptionRaw {
    pub option: String,
    pub set: bool,

    #[serde(flatten)]
    pub value: std::collections::HashMap<String, serde_json::Value>,

    #[serde(skip)]
    pub json: String,
}

/// This enum holds the possible values of a keyword/option
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub enum OptionValue {
    /// A integer (64-bit)
    Int(i64),
    /// A floating point (64-point)
    Float(f64),
    /// A string
    String(String),
    /// A hyprland Color or Gradient
    Custom(Custom),
    /// A Vector of 2 ints
    Vec2([i64; 2]),
    /// A boolean
    Bool(bool),
    /// Could not parse value
    Unknown(String),
}

impl std::fmt::Display for OptionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionValue::Int(i) => f.write_fmt(format_args!("{}", i)),
            OptionValue::Float(fl) => f.write_fmt(format_args!("{}", fl)),
            OptionValue::String(s) => f.write_fmt(format_args!("{}", s)),
            OptionValue::Custom(custom) => f.write_fmt(format_args!("{}", custom)),
            OptionValue::Vec2(v) => f.write_fmt(format_args!("{} {}", v[0], v[1])),
            OptionValue::Bool(b) => f.write_fmt(format_args!("{b}")),
            OptionValue::Unknown(s) => f.write_fmt(format_args!("Unknown Value type ({})", s)),
        }
    }
}

impl From<OptionValue> for String {
    fn from(opt: OptionValue) -> Self {
        opt.to_string()
    }
}

trait IsString {}
impl IsString for String {}
impl IsString for &str {}

impl<Str: ToString + IsString> From<Str> for OptionValue {
    /// Bare text becomes a [OptionValue::String]; it becomes a
    /// [OptionValue::Custom] only when it is unambiguously a colour or geometry.
    ///
    /// This used to try [Custom::try_from] **first**, and because
    /// [HyprColor::try_from_argb_str] accepted any hex length, `Keyword::set("k", "1")`
    /// set a nearly-transparent black and `set("k", "deadbeef")` set a colour.
    ///
    /// Bare 6/8-digit hex is **not** auto-detected here, because
    /// [std::fmt::Display] for [HyprColor] only ever emits `rgba(...)` — accepting
    /// bare hex on input would be a form the crate cannot produce on output. Write
    /// `rgba(deadbeef)` or `0xdeadbeef` for a colour. This is the *write* path only:
    /// parsing Hyprland's replies goes through [Custom::try_from] directly and still
    /// accepts the bare `ee1a1a1a` form Hyprland sends.
    fn from(str: Str) -> Self {
        let s = str.to_string();
        let t = s.trim();
        let looks_custom = t.starts_with("rgb") || t.starts_with("0x") || {
            // HyprRect: `Custom::try_from` only builds one from exactly four
            // whitespace-separated integers, e.g. the css gap string "3 3 3 3".
            // Two or three go down the gradient path and fail there.
            let parts: Vec<&str> = t.split_whitespace().collect();
            parts.len() == 4 && parts.iter().all(|p| p.parse::<i64>().is_ok())
        };

        if looks_custom && let Ok(c) = Custom::try_from(t) {
            OptionValue::Custom(c)
        } else {
            OptionValue::String(s)
        }
    }
}

macro_rules! match_unknown {
    ($k:expr, $v:expr, $opt:ident) => {
        match $v {
            Some(vi) => OptionValue::$opt(vi),
            None => OptionValue::Unknown($k.to_string()),
        }
    };
}

impl TryFrom<&OptionRaw> for OptionValue {
    type Error = HyprError;
    fn try_from(raw: &OptionRaw) -> crate::Result<Self> {
        Ok(match raw.value.iter().next() {
            Some((k, v)) => match k.as_str() {
                "int" => match_unknown!(raw.json, v.as_i64(), Int),
                "float" => match_unknown!(raw.json, v.as_f64(), Float),
                "str" => match_unknown!(raw.json, v.as_str().map(|v| v.to_string()), String),
                "custom" => match_unknown!(
                    raw.json,
                    v.as_str().and_then(|v| Custom::try_from(v).ok()),
                    Custom
                ),
                "vec2" => {
                    if let Some(a) = v.as_array()
                        && a.len() == 2
                        && a[0].is_i64()
                        && a[1].is_i64()
                    {
                        return Ok(OptionValue::Vec2([
                            a[0].as_i64().ok_or(HyprError::InvalidOptionValue)?,
                            a[1].as_i64().ok_or(HyprError::InvalidOptionValue)?,
                        ]));
                    }
                    OptionValue::Unknown(raw.json.to_string())
                }
                "bool" => match_unknown!(raw.json, v.as_bool(), Bool),
                // `css` is a whitespace-separated gap string of 1, 2 or 4 ints
                // (e.g. "3 3 3 3"), which is the same shape as `custom`
                "css" => match_unknown!(
                    raw.json,
                    v.as_str().and_then(|s| Custom::try_from(s).ok()),
                    Custom
                ),
                // `gradient` is "<color> <color> <angle>deg"
                "gradient" => match_unknown!(
                    raw.json,
                    v.as_str()
                        .and_then(|g| Custom::try_from(g).ok())
                        .filter(|c| matches!(c, Custom::HyprGradient(_))),
                    Custom
                ),
                // `font_weight` arrives as an int (100, 200, ...)
                "font_weight" => match_unknown!(raw.json, v.as_i64(), Int),
                _ => OptionValue::Unknown(raw.json.to_string()),
            },
            None => OptionValue::Unknown(raw.json.to_string()),
        })
    }
}

/// This struct holds a keyword
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Keyword {
    /// The identifier (or name) of the keyword
    pub option: String,
    /// The value of the keyword/option
    pub value: OptionValue,
    /// Is value overriden or not
    pub set: bool,
}

impl Keyword {
    /// This function sets a keyword's value
    pub fn set<Str: ToString, Opt: Into<OptionValue>>(key: Str, value: Opt) -> crate::Result<()> {
        Self::instance_set(default_instance()?, key, value)
    }

    /// This function sets a keyword's value
    pub fn instance_set<Str: ToString, Opt: Into<OptionValue>>(
        instance: &Instance,
        key: Str,
        value: Opt,
    ) -> crate::Result<()> {
        let value = value.into();

        let value = match value {
            OptionValue::Unknown(_) => {
                return Err(crate::HyprError::InvalidOptionValue);
            }
            x => x,
        };

        instance.write_to_socket(command!(
            Empty,
            "keyword {} {}",
            key.to_string(),
            value.to_string()
        ))?;
        Ok(())
    }

    /// This function sets a keyword's value (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn set_async<Str: ToString, Opt: Into<OptionValue>>(
        key: Str,
        value: Opt,
    ) -> crate::Result<()> {
        Self::instance_set_async(default_instance()?, key, value).await
    }

    /// This function sets a keyword's value (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_set_async<Str: ToString, Opt: Into<OptionValue>>(
        instance: &Instance,
        key: Str,
        value: Opt,
    ) -> crate::Result<()> {
        instance
            .write_to_socket_async(command!(
                Empty,
                "keyword {} {}",
                key.to_string(),
                value.into().to_string()
            ))
            .await?;
        Ok(())
    }

    /// This function returns the value of a keyword
    pub fn get<Str: ToString>(key: Str) -> crate::Result<Self> {
        Self::instance_get(default_instance()?, key)
    }

    /// This function returns the value of a keyword
    pub fn instance_get<Str: ToString>(instance: &Instance, key: Str) -> crate::Result<Self> {
        let data = instance.write_to_socket(command!(JSON, "getoption {}", key.to_string()))?;
        if data == "no such option" {
            return Err(crate::error::HyprError::InvalidOptionKey(key.to_string()));
        }
        let mut deserialized: OptionRaw = serde_json::from_str(&data)?;
        deserialized.json = data;
        let value = OptionValue::try_from(&deserialized)?;

        let keyword = Keyword {
            option: deserialized.option,
            value,
            set: deserialized.set,
        };
        Ok(keyword)
    }

    /// This function returns the value of a keyword (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn get_async<Str: ToString>(key: Str) -> crate::Result<Self> {
        Self::instance_get_async(default_instance()?, key).await
    }

    /// This function returns the value of a keyword (async)
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_get_async<Str: ToString>(
        instance: &Instance,
        key: Str,
    ) -> crate::Result<Self> {
        let data = instance
            .write_to_socket_async(command!(JSON, "getoption {}", key.to_string()))
            .await?;
        if data == "no such option" {
            return Err(crate::error::HyprError::InvalidOptionKey(key.to_string()));
        }
        let mut deserialized: OptionRaw = serde_json::from_str(&data)?;
        deserialized.json = data;

        let value = OptionValue::try_from(&deserialized)?;

        let keyword = Keyword {
            option: deserialized.option,
            value,
            set: deserialized.set,
        };
        Ok(keyword)
    }
}

/// One entry of the `descriptions` command: an option's name, its description,
/// and its default and current values.
///
/// `default` and `current` are [serde_json::Value] because Hyprland's shape is
/// type-dependent (`ConfigValues.cpp:30-160`): Int options emit bare numbers,
/// `css`/`gradient`/`vec2`/colour options emit strings or `[x, y]` arrays, and
/// Bool options emit real booleans. `min`/`max` are only emitted at all for the
/// Int, Float and `css` types, and Int options also carry a `map` field that
/// this struct ignores.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OptionDescription {
    /// The option name in the colon form Hyprland emits, e.g. `general:border_size`
    pub name: String,
    /// The human-readable description of the option
    pub description: String,
    /// The default value, whose JSON shape depends on the option's type
    pub default: serde_json::Value,
    /// The current value, in the same shape as [Self::default]
    pub current: serde_json::Value,
    /// The minimum, when the option has one
    pub min: Option<serde_json::Value>,
    /// The maximum, when the option has one
    pub max: Option<serde_json::Value>,
}

impl OptionDescription {
    /// The name in the dotted form [Keyword::get] accepts, using Hyprland's own
    /// translation: every `:` becomes `.` and every `-` becomes `_`
    /// (`lua/ConfigManager.cpp:1136-1141`).
    ///
    /// `descriptions` is the one command that emits colon-form names, so this is
    /// what turns a discovered entry into an option that can be read.
    pub fn dotted_name(&self) -> String {
        self.name.replace(':', ".").replace('-', "_")
    }
}

/// This struct holds a vector of every option description
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OptionDescriptions(Vec<OptionDescription>);

impl HyprData for OptionDescriptions {
    fn get() -> crate::Result<Self> {
        Self::instance_get(default_instance()?)
    }
    fn instance_get(instance: &Instance) -> crate::Result<Self> {
        Ok(Self(serde_json::from_str(
            &instance.write_to_socket(command!(JSON, "descriptions"))?,
        )?))
    }
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn get_async() -> crate::Result<Self> {
        Self::instance_get_async(default_instance()?).await
    }
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn instance_get_async(instance: &Instance) -> crate::Result<Self> {
        Ok(Self(serde_json::from_str(
            &instance
                .write_to_socket_async(command!(JSON, "descriptions"))
                .await?,
        )?))
    }
}

impl HyprDataVec<OptionDescription> for OptionDescriptions {
    fn to_vec(self) -> Vec<OptionDescription> {
        self.0
    }
}

impl std::ops::Deref for OptionDescriptions {
    type Target = Vec<OptionDescription>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<OptionDescription>> for OptionDescriptions {
    fn from(value: Vec<OptionDescription>) -> Self {
        Self(value)
    }
}

impl From<OptionDescriptions> for Vec<OptionDescription> {
    fn from(value: OptionDescriptions) -> Self {
        value.0
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    /// Hyprland writes colours as `rgba(FF00AA7F)` — 8 hex digits. The old
    /// parser only accepted 6, so it rejected its own documented example.
    #[test]
    fn hypr_color_parses_eight_digit_rgba() {
        // round-trips: Hyprland writes rrggbbaa and Display emits the same 8 digits
        let c = HyprColor::try_from_rgba_str("rgba(ff00aa7f)").expect("8-digit rgba");
        assert_eq!(c.to_string().to_lowercase(), "rgba(ff00aa7f)");
        // the crate's own documented example
        assert!(HyprColor::try_from_rgba_str("rgba(b3ff1aee)").is_some());
    }

    #[test]
    fn hypr_color_still_parses_six_digit_rgba_as_opaque() {
        assert!(HyprColor::try_from_rgba_str("rgba(ff00aa)").is_some());
    }

    #[test]
    fn hypr_color_parses_decimal_rgba() {
        assert!(HyprColor::try_from_rgba_str("rgba(255,0,170,0.5)").is_some());
    }

    fn raw(payload: &str) -> OptionRaw {
        serde_json::from_str(payload).unwrap()
    }

    /// `misc:disable_hyprland_logo` and most other bool options used to come
    /// back `Unknown`. This is a semver break (a new variant on a public enum
    /// with no `#[non_exhaustive]`), which is why it did not go into 0.4.0.
    #[test]
    fn option_value_parses_bool() {
        for (payload, expected) in [
            (
                r#"{"option":"misc:disable_hyprland_logo","set":true,"bool":true}"#,
                true,
            ),
            (
                r#"{"option":"decoration:blur:enabled","set":true,"bool":false}"#,
                false,
            ),
            (
                r#"{"option":"input:touchpad:natural_scroll","set":true,"bool":true}"#,
                true,
            ),
        ] {
            let v = OptionValue::try_from(&raw(payload)).unwrap();
            assert!(
                matches!(v, OptionValue::Bool(b) if b == expected),
                "payload {payload} gave {v:?}"
            );
        }
    }

    /// `bool` must round-trip through Display, since `Keyword::set` writes
    /// the value back as a string.
    #[test]
    fn option_value_bool_displays_as_a_bare_word() {
        assert_eq!(OptionValue::Bool(true).to_string(), "true");
        assert_eq!(OptionValue::Bool(false).to_string(), "false");
    }

    /// `general:gaps_in` used to come back `Unknown`.
    #[test]
    fn option_value_parses_css() {
        let v = OptionValue::try_from(&raw(
            r#"{"option":"general:gaps_in","set":true,"css":"3 3 3 3"}"#,
        ))
        .unwrap();
        assert!(
            matches!(v, OptionValue::Custom(Custom::HyprRect(_))),
            "got {v:?}"
        );
    }

    /// `decoration:shadow:color` used to come back `Unknown`.
    #[test]
    fn option_value_parses_gradient() {
        let v = OptionValue::try_from(&raw(
            r#"{"option":"decoration:shadow:color","set":true,"gradient":"rgba(ee1a1a1a) rgba(00ff00ff) 90deg"}"#,
        ))
        .unwrap();
        assert!(
            matches!(v, OptionValue::Custom(Custom::HyprGradient(_))),
            "got {v:?}"
        );
    }

    #[test]
    fn option_value_still_parses_the_original_types() {
        assert!(matches!(
            OptionValue::try_from(&raw(
                r#"{"option":"general:border_size","set":true,"int":2}"#
            ))
            .unwrap(),
            OptionValue::Int(2)
        ));
    }

    #[test]
    fn unknown_type_stays_unknown() {
        assert!(matches!(
            OptionValue::try_from(&raw(r#"{"option":"x","set":true,"nonsense":1}"#)).unwrap(),
            OptionValue::Unknown(_)
        ));
    }
}

#[cfg(test)]
mod description_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    /// A slice of real upstream output, one entry per value type, in the shapes
    /// `ConfigValues.cpp:30-160` actually emits. `general:border_size` is an Int
    /// (min/max present, `map` always emitted and always ignored here),
    /// `general:gaps_in` is a `css` gap (min/max present but null),
    /// `general:col.inactive_border` is a Gradient and
    /// `decoration:blur:enabled` a Bool (no min/max at all).
    const PAYLOAD: &str = r#"[
            {"name":"general:border_size","description":"size of the border around windows","default":1,"current":2,"min":0,"max":20,"map":null},
            {"name":"general:gaps_in","description":"gaps between windows","default":"5","current":"3 3 3 3","min":null,"max":null},
            {"name":"general:col.inactive_border","description":"border color for inactive windows","default":"ff444444","current":"ff444444"},
            {"name":"decoration:blur:enabled","description":"enable kawase window background blur","default":true,"current":false}
        ]"#;

    fn descriptions() -> Vec<OptionDescription> {
        serde_json::from_str(PAYLOAD).unwrap()
    }

    /// Every upstream type shape must deserialize,including the Int `map`
    /// field this struct deliberately drops.
    #[test]
    fn every_upstream_shape_deserializes() {
        let all = descriptions();
        assert_eq!(all.len(), 4);

        let int = &all[0];
        assert_eq!(int.name, "general:border_size");
        assert_eq!(int.description, "size of the border around windows");
        assert_eq!(int.default, serde_json::json!(1));
        assert_eq!(int.current, serde_json::json!(2));
        assert_eq!(int.min, Some(serde_json::json!(0)));
        assert_eq!(int.max, Some(serde_json::json!(20)));

        // css gaps are strings, and their null min/max are None, not errors
        let css = &all[1];
        assert_eq!(css.default, serde_json::json!("5"));
        assert_eq!(css.current, serde_json::json!("3 3 3 3"));
        assert!(css.min.is_none() && css.max.is_none());

        // a gradient arrives pre-rendered as a hex string
        assert_eq!(all[2].default, serde_json::json!("ff444444"));

        // a bool is a real JSON bool, and the type emits no min/max keys at all
        assert_eq!(all[3].default, serde_json::json!(true));
        assert!(all[3].min.is_none() && all[3].max.is_none());
    }

    /// Hyprland's own translation, `lua/ConfigManager.cpp:1136-1141`: every
    /// `:` becomes `.` and every `-` becomes `_`. `descriptions` emits colon
    /// names, so this is what makes a discovered entry readable by
    /// `Keyword::get`.
    #[test]
    fn dotted_name_uses_hyprlands_own_translation() {
        assert_eq!(
            descriptions()[0].dotted_name(),
            "general.border_size",
            "one colon"
        );
        // a name with a colon *and* a dot in the key
        assert_eq!(
            descriptions()[2].dotted_name(),
            "general.col.inactive_border",
            "colon and dot"
        );
        // nested sections: two colons, both translated
        assert_eq!(
            OptionDescription {
                name: "decoration:blur:enabled".to_string(),
                description: String::new(),
                default: serde_json::json!(true),
                current: serde_json::json!(true),
                min: None,
                max: None,
            }
            .dotted_name(),
            "decoration.blur.enabled",
            "two colons"
        );
        // dashes become underscores, per the same two lines of upstream
        assert_eq!(
            OptionDescription {
                name: "device:epic-mouse-v1:sensitivity".to_string(),
                description: String::new(),
                default: serde_json::json!(0),
                current: serde_json::json!(0),
                min: None,
                max: None,
            }
            .dotted_name(),
            "device.epic_mouse_v1.sensitivity",
            "dash"
        );
    }

    /// The reader type must behave like the vector it wraps, because
    /// discovering an option means iterating and filtering the whole set.
    #[test]
    fn the_reader_behaves_like_a_vector() {
        let all: OptionDescriptions = descriptions().into();
        assert_eq!(all.len(), 4);
        assert_eq!(all[0].name, "general:border_size");
        assert_eq!(all.iter().filter(|o| o.max.is_some()).count(), 1);

        let by_name = |n: &str| all.iter().any(|o| o.name == n);
        assert!(by_name("decoration:blur:enabled"));
        assert!(!by_name("decoration:blur:nonsense"));

        let owned: Vec<OptionDescription> = all.clone().to_vec();
        assert_eq!(owned.len(), 4);
        assert_eq!(Vec::from(all), owned);
    }
}

#[cfg(test)]
mod conversion_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    /// `try_from_argb_str` accepted any hex length, so a one-digit string parsed
    /// as a colour.
    #[test]
    fn short_hex_is_not_a_colour() {
        assert!(HyprColor::try_from_argb_str("1").is_none());
        assert!(HyprColor::try_from_argb_str("a").is_none());
        assert!(HyprColor::try_from_argb_str("0").is_none());
    }

    #[test]
    fn six_and_eight_digit_hex_are_colours() {
        assert!(HyprColor::try_from_argb_str("ff00aa").is_some());
        assert!(HyprColor::try_from_argb_str("ff00aa7f").is_some());
        assert!(HyprColor::try_from_argb_str("0xff00aa").is_some());
    }

    /// The bug this fixes: `Keyword::set("k", "1")` silently set a colour.
    #[test]
    fn bare_numbers_stay_strings() {
        assert!(matches!(
            OptionValue::from("1".to_owned()),
            OptionValue::String(_)
        ));
        assert!(matches!(
            OptionValue::from("0.5".to_owned()),
            OptionValue::String(_)
        ));
        assert!(matches!(
            OptionValue::from("true".to_owned()),
            OptionValue::String(_)
        ));
        assert!(matches!(
            OptionValue::from(String::new()),
            OptionValue::String(_)
        ));
    }

    /// `"deadbeef"` is 8 valid hex digits but is far more likely to be a string
    /// than a colour, and `Display` only ever emits `rgba(...)`.
    #[test]
    fn bare_hex_stays_a_string_unless_prefixed() {
        assert!(matches!(
            OptionValue::from("deadbeef".to_owned()),
            OptionValue::String(_)
        ));
        assert!(matches!(
            OptionValue::from("ff00aa7f".to_owned()),
            OptionValue::String(_)
        ));
        assert!(matches!(
            OptionValue::from("rgba(ff00aa7f)".to_owned()),
            OptionValue::Custom(_)
        ));
        assert!(matches!(
            OptionValue::from("0xff00aa".to_owned()),
            OptionValue::Custom(_)
        ));
    }

    #[test]
    fn ordinary_words_stay_strings() {
        for s in ["hello", "special", "dwindle", "Hyprland", "main"] {
            assert!(
                matches!(OptionValue::from(s.to_owned()), OptionValue::String(_)),
                "{s} should stay a string"
            );
        }
    }

    /// A gap string is structured, so it stays auto-detected.
    #[test]
    fn gap_strings_are_rects() {
        assert!(matches!(
            OptionValue::from("3 3 3 3".to_owned()),
            OptionValue::Custom(Custom::HyprRect(_))
        ));
        // Two or three integers are not a rect — `Custom::try_from` routes them
        // to HyprGradient, which rejects them, so they stay strings.
        assert!(matches!(
            OptionValue::from("3 5".to_owned()),
            OptionValue::String(_)
        ));
    }

    /// `{:0}` is a zero-pad flag with no width — it does nothing. Removed.
    /// `Float(7.0)` renders `"7"` and that is **correct**: hyprlang parses the
    /// value with `std::stof` (`src/config.cpp:479`, CONFIGDATATYPE_FLOAT),
    /// which accepts an integer literal for a float option. The original claim
    /// that Hyprland "may reject" it was never verified and is contradicted by
    /// the source.
    #[test]
    fn floats_render_without_a_trailing_zero() {
        assert_eq!(OptionValue::Float(7.0).to_string(), "7");
        assert_eq!(OptionValue::Float(0.5).to_string(), "0.5");
        assert_eq!(OptionValue::Float(-3.5).to_string(), "-3.5");
        assert_eq!(OptionValue::Int(7).to_string(), "7");
    }

    /// Reading Hyprland's replies is a different path and must still accept the
    /// bare `ee1a1a1a` form Hyprland sends.
    #[test]
    fn the_read_path_still_accepts_bare_hex() {
        let raw: OptionRaw = serde_json::from_str(
            r#"{"option":"decoration:shadow:color","set":true,"custom":"ee1a1a1a 0deg"}"#,
        )
        .unwrap();
        let v = OptionValue::try_from(&raw).unwrap();
        assert!(
            matches!(v, OptionValue::Custom(Custom::HyprGradient(_))),
            "got {v:?}"
        );
    }
}
