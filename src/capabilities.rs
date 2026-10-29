//! Runtime capabilities of the connected Hyprland instance.

use crate::command;
use crate::instance::Instance;
use serde::Deserialize;

/// The config grammar the compositor selected at startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum ConfigProvider {
    /// The legacy `hyprlang` (`.conf`) provider.
    #[default]
    Legacy,
    /// The Lua provider.
    Lua,
}

impl ConfigProvider {
    /// Parse a `j/status` response. Unknown or malformed input means [`Self::Legacy`].
    #[must_use]
    pub fn from_status_json(data: &str) -> Self {
        let Ok(status) = serde_json::from_str::<StatusPayload>(data) else {
            return Self::Legacy;
        };

        if status.config_provider.eq_ignore_ascii_case("lua") {
            Self::Lua
        } else {
            Self::Legacy
        }
    }
}

/// Capabilities discovered from the running compositor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct Capabilities {
    /// The config grammar the compositor is using.
    pub config_provider: ConfigProvider,
}

impl Capabilities {
    /// Build capabilities from a `j/status` response.
    #[must_use]
    pub fn from_status_json(data: &str) -> Self {
        Self {
            config_provider: ConfigProvider::from_status_json(data),
        }
    }

    fn probe(instance: &Instance) -> Self {
        let Ok(data) = instance.write_to_socket(command!(JSON, "status")) else {
            return Self::default();
        };
        Self::from_status_json(&data)
    }

    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn probe_async(instance: &Instance) -> Self {
        let Ok(data) = instance
            .write_to_socket_async(command!(JSON, "status"))
            .await
        else {
            return Self::default();
        };
        Self::from_status_json(&data)
    }
}

impl Instance {
    /// Return the cached capabilities of this instance.
    pub fn capabilities(&self) -> Capabilities {
        *self.capabilities.get_or_init(|| Capabilities::probe(self))
    }

    /// Return the cached capabilities of this instance, asynchronously.
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn capabilities_async(&self) -> Capabilities {
        if let Some(capabilities) = self.capabilities.get() {
            return *capabilities;
        }

        let capabilities = Capabilities::probe_async(self).await;
        let _ = self.capabilities.set(capabilities);
        capabilities
    }

    /// Whether the compositor is using the Lua config provider.
    #[must_use]
    pub fn uses_lua_config(&self) -> bool {
        self.capabilities().config_provider == ConfigProvider::Lua
    }

    /// Whether the compositor is using the Lua config provider, asynchronously.
    #[must_use]
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn uses_lua_config_async(&self) -> bool {
        self.capabilities_async().await.config_provider == ConfigProvider::Lua
    }
}

#[derive(Deserialize)]
struct StatusPayload {
    #[serde(rename = "configProvider")]
    config_provider: String,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn lua_is_detected() {
        let caps = Capabilities::from_status_json(r#"{"configProvider":"lua","backend":"drm"}"#);
        assert_eq!(caps.config_provider, ConfigProvider::Lua);
    }

    #[test]
    fn hyprlang_is_the_legacy_provider() {
        let caps =
            Capabilities::from_status_json(r#"{"configProvider":"hyprlang","backend":"wayland"}"#);
        assert_eq!(caps.config_provider, ConfigProvider::Legacy);
    }

    #[test]
    fn an_unknown_provider_falls_back_to_legacy() {
        let caps = Capabilities::from_status_json(r#"{"configProvider":"something-new"}"#);
        assert_eq!(caps.config_provider, ConfigProvider::Legacy);
    }

    #[test]
    fn an_unknown_request_falls_back_to_legacy() {
        assert_eq!(
            ConfigProvider::from_status_json("unknown request"),
            ConfigProvider::Legacy
        );
    }

    #[test]
    fn malformed_json_falls_back_to_legacy() {
        assert_eq!(
            ConfigProvider::from_status_json(r#"{"configProvider":"#),
            ConfigProvider::Legacy
        );
    }

    #[test]
    fn a_missing_field_falls_back_to_legacy() {
        assert_eq!(
            ConfigProvider::from_status_json(r#"{"backend":"drm"}"#),
            ConfigProvider::Legacy
        );
    }

    #[test]
    fn the_provider_match_is_case_insensitive() {
        assert_eq!(
            ConfigProvider::from_status_json(r#"{"configProvider":"LUA"}"#),
            ConfigProvider::Lua
        );
    }
}
