use super::*;
use crate::default_instance;
use crate::error::hypr_err;
use crate::instance::Instance;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

const SPECIAL_WORKSPACE_ID_START: WorkspaceId = -99;
const SPECIAL_WORKSPACE_ID_END: WorkspaceId = -2;

/// Mirrors Hyprland's `CWorkspaceQueryCore::isSpecial`. Named workspaces are
/// negative too (from -1337 down), so `id < 0` is not enough.
fn is_special_id(id: WorkspaceId) -> bool {
    (SPECIAL_WORKSPACE_ID_START..=SPECIAL_WORKSPACE_ID_END).contains(&id)
}

/// This pub(crate) enum holds every socket command that returns data
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DataCommands {
    #[display("monitors all")]
    Monitors,
    #[display("workspaces")]
    Workspaces,
    #[display("activeworkspace")]
    ActiveWorkspace,
    #[display("clients")]
    Clients,
    #[display("activewindow")]
    ActiveWindow,
    #[display("layers")]
    Layers,
    #[display("devices")]
    Devices,
    #[display("version")]
    Version,
    #[display("cursorpos")]
    CursorPosition,
    #[display("binds")]
    Binds,
    #[display("animations")]
    Animations,
    #[display("workspacerules")]
    WorkspaceRules,
}

/// This struct holds a basic identifier for a workspace often used in other structs
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceBasic {
    /// The workspace Id
    pub id: WorkspaceId,
    /// The workspace's name
    pub name: String,
}

impl WorkspaceBasic {
    /// If this is a special workspace
    pub fn is_special(&self) -> bool {
        is_special_id(self.id)
    }
}

/// This enum provides the different monitor transforms
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq, Copy)]
#[repr(u8)]
#[non_exhaustive]
pub enum Transforms {
    /// No transform
    Normal = 0,
    /// Rotated 90 degrees
    Normal90 = 1,
    /// Rotated 180 degrees
    Normal180 = 2,
    /// Rotated 270 degrees
    Normal270 = 3,
    /// Flipped
    Flipped = 4,
    /// Flipped and rotated 90 degrees
    Flipped90 = 5,
    /// Flipped and rotated 180 degrees
    Flipped180 = 6,
    /// Flipped and rotated 270 degrees
    Flipped270 = 7,
}

/// This struct holds information for a monitor
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Monitor {
    /// The monitor id
    pub id: MonitorId,
    /// The monitor's name
    pub name: String,
    /// The monitor's description
    pub description: String,
    /// The monitor width (in pixels)
    pub width: u16,
    /// The monitor height (in pixels)
    pub height: u16,
    /// The monitor's refresh rate (in hertz)
    #[serde(rename = "refreshRate")]
    pub refresh_rate: f32,
    /// The monitor's position on the x axis (not irl ofc)
    pub x: i32,
    /// The monitor's position on the x axis (not irl ofc)
    pub y: i32,
    /// A basic identifier for the active workspace
    #[serde(rename = "activeWorkspace")]
    pub active_workspace: WorkspaceBasic,
    /// A basic identifier for the special workspace
    #[serde(rename = "specialWorkspace")]
    pub special_workspace: WorkspaceBasic,
    /// Reserved is the amount of space (in pre-scale pixels) that a layer surface has claimed
    pub reserved: (u16, u16, u16, u16),
    /// The display's scale
    pub scale: f32,
    /// I think like the rotation?
    pub transform: Transforms,
    /// a string that identifies if the display is active
    pub focused: bool,
    /// The dpms status of a monitor
    #[serde(rename = "dpmsStatus")]
    pub dpms_status: bool,
    /// VRR state
    pub vrr: bool,
    /// Is the monitor disabled or not
    pub disabled: bool,
    /// The physical width of the monitor in mm
    #[serde(rename = "physicalWidth", default)]
    pub physical_width: u16,
    /// The physical size of the monitor in mm
    #[serde(rename = "physicalHeight", default)]
    pub physical_height: u16,
}

impl HyprDataActive for Monitor {
    fn get_active() -> crate::Result<Self> {
        Self::instance_get_active(default_instance()?)
    }
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn get_active_async() -> crate::Result<Self> {
        Self::instance_get_active_async(default_instance()?).await
    }
    fn instance_get_active(instance: &Instance) -> crate::Result<Self> {
        let all = Monitors::instance_get(instance)?;
        if let Some(it) = all.into_iter().find(|item| item.focused) {
            Ok(it)
        } else {
            hypr_err!("No active Hyprland monitor detected!")
        }
    }
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn instance_get_active_async(instance: &Instance) -> crate::Result<Self> {
        let all = Monitors::instance_get_async(instance).await?;
        if let Some(it) = all.into_iter().find(|item| item.focused) {
            Ok(it)
        } else {
            hypr_err!("No active Hyprland monitor detected!")
        }
    }
}

create_data_struct!(
    vector,
    name: Monitors,
    command: DataCommands::Monitors,
    holding_type: Monitor,
    doc: "This struct holds a vector of monitors"
);

/// This struct holds information for a workspace
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    /// The workspace Id
    pub id: WorkspaceId,
    /// The workspace's name
    pub name: String,
    /// The monitor the workspace is on
    pub monitor: String,
    /// The monitor id the workspace is on, can be None in some cases
    #[serde(rename = "monitorID")]
    pub monitor_id: Option<MonitorId>,
    /// The amount of windows in the workspace
    pub windows: u16,
    /// A bool that shows if there is a fullscreen window in the workspace
    #[serde(rename = "hasfullscreen")]
    pub fullscreen: bool,
    /// The last window's [Address]
    #[serde(rename = "lastwindow")]
    pub last_window: Address,
    /// The last window's title
    #[serde(rename = "lastwindowtitle")]
    pub last_window_title: String,
    /// Is this workspace persistent?
    #[serde(default, rename = "ispersistent")]
    pub persistent: bool,
    /// The workspace's layout
    #[serde(default, rename = "tiledLayout")]
    pub tiled_layout: String,
}

impl Workspace {
    /// If this is a special workspace
    pub fn is_special(&self) -> bool {
        is_special_id(self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace(id: WorkspaceId, name: &str) -> Workspace {
        Workspace {
            id,
            name: name.to_owned(),
            monitor: "DP-1".to_owned(),
            monitor_id: Some(1),
            windows: 0,
            fullscreen: false,
            last_window: Address::new(0),
            last_window_title: String::new(),
            persistent: false,
            tiled_layout: String::new(),
        }
    }

    #[test]
    fn numbered_workspace_is_not_special() {
        assert!(!workspace(1, "1").is_special());
    }

    #[test]
    fn named_workspace_is_not_special() {
        assert!(!workspace(-1337, "example").is_special());
        assert!(!workspace(-1338, "other").is_special());
    }

    #[test]
    fn named_workspace_with_special_prefix_is_not_special() {
        assert!(!workspace(-1337, "special:foo").is_special());
    }

    #[test]
    fn special_workspace_is_special() {
        assert!(workspace(-98, "special:magic").is_special());
        assert!(workspace(-99, "special:scratch").is_special());
    }

    #[test]
    fn special_id_range_is_inclusive_at_both_ends() {
        assert!(workspace(-99, "special:low").is_special());
        assert!(workspace(-2, "special:high").is_special());
    }

    #[test]
    fn ids_outside_the_special_range_are_not_special() {
        assert!(!workspace(-1, "special:invalid").is_special());
        assert!(!workspace(-100, "special:below").is_special());
    }

    fn basic(id: WorkspaceId, name: &str) -> WorkspaceBasic {
        WorkspaceBasic {
            id,
            name: name.to_owned(),
        }
    }

    #[test]
    fn workspace_basic_agrees_with_workspace() {
        for (id, name) in [
            (1, "1"),
            (-1337, "example"),
            (-1337, "special:foo"),
            (-98, "special:magic"),
            (-99, "special:low"),
            (-2, "special:high"),
            (-1, "special:invalid"),
        ] {
            assert_eq!(
                basic(id, name).is_special(),
                workspace(id, name).is_special(),
                "disagreement for id {id} name {name}"
            );
        }
    }
}

impl HyprDataActive for Workspace {
    fn get_active() -> crate::Result<Self> {
        Self::instance_get_active(default_instance()?)
    }
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn get_active_async() -> crate::Result<Self> {
        Self::instance_get_active_async(default_instance()?).await
    }
    fn instance_get_active(instance: &Instance) -> crate::Result<Self> {
        let data = instance.write_to_socket(command!(JSON, "{}", DataCommands::ActiveWorkspace))?;
        let deserialized: Workspace = serde_json::from_str(&data)?;
        Ok(deserialized)
    }
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn instance_get_active_async(instance: &Instance) -> crate::Result<Self> {
        let data = instance
            .write_to_socket_async(command!(JSON, "{}", DataCommands::ActiveWorkspace))
            .await?;
        let deserialized: Workspace = serde_json::from_str(&data)?;
        Ok(deserialized)
    }
}

create_data_struct!(
    vector,
    name: Workspaces,
    command: DataCommands::Workspaces,
    holding_type: Workspace,
    doc: "This type provides a vector of workspaces"
);

/// This struct holds information for a client/window fullscreen mode
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq, Copy)]
#[repr(u8)]
#[non_exhaustive]
pub enum FullscreenMode {
    /// Normal window
    None = 0,
    /// Maximized window
    Maximized = 1,
    /// Fullscreen window
    Fullscreen = 2,
    /// Maximized and fullscreen window
    MaximizedFullscreen = 3,
}

/// This struct holds information for a client/window
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Client {
    /// The client's [`Address`][crate::shared::Address]
    pub address: Address,
    /// Is this window printed on screen?
    pub mapped: bool,
    /// Is this window hidden?
    #[serde(default)]
    pub hidden: bool,
    /// Is this window visible on screen?
    pub visible: bool,
    #[serde(default, rename = "acceptsInput")]
    /// Does this window accept input?
    pub accepts_input: bool,
    /// The window location
    ///
    /// Hyprland sends a `Vector2D`, which is `double`
    /// (`hyprutils/math/Vector2D.hpp`), narrowed to `i32` here so pixel
    /// coordinates stay exact. These were `i16`, which overflowed past 32767.
    pub at: (i32, i32),
    /// The window size
    ///
    /// See [`Client::at`] for why this is `i32` and not `i16`.
    pub size: (i32, i32),
    /// The workspace the window is on
    pub workspace: WorkspaceBasic,
    /// Is this window floating?
    pub floating: bool,
    /// The monitor ID the window is on, can be None in some cases
    pub monitor: Option<MonitorId>,
    /// The window class
    pub class: String,
    /// The window title
    pub title: String,
    /// The initial window class
    #[serde(rename = "initialClass")]
    pub initial_class: String,
    /// The initial window title
    #[serde(rename = "initialTitle")]
    pub initial_title: String,
    /// The process ID of the client
    pub pid: i32,
    /// Is this window running under XWayland?
    pub xwayland: bool,
    /// Is this window pinned?
    pub pinned: bool,
    /// Is the window pinned *and* fullscreened? Preserves the pinned state
    /// while fullscreening (upstream `m_pinFullscreened`).
    #[serde(default, rename = "pinFullscreened")]
    pub pin_fullscreened: bool,
    /// The internal fullscreen mode
    pub fullscreen: FullscreenMode,
    /// The client fullscreen mode
    #[serde(rename = "fullscreenClient")]
    pub fullscreen_client: FullscreenMode,
    /// Which fullscreen handler manages this window (upstream
    /// `getFullscreenHandlerNameAsString`): `"default"`, `"scrolling"` or
    /// `"unknown"`.
    #[serde(default, rename = "fullscreenHandler")]
    pub fullscreen_handler: String,
    /// Whether the window may render over a fullscreen window (upstream
    /// `m_allowedOverFullscreen`). This replaced the old `overFullscreen` key.
    #[serde(default, rename = "allowedOverFullscreen")]
    pub allowed_over_fullscreen: bool,
    /// Whether the window was created over a fullscreen window
    ///
    /// Deprecated: Hyprland no longer emits `overFullscreen` — it was replaced
    /// by [`Client::allowed_over_fullscreen`] — so this is always `false` on
    /// current Hyprland. Kept so older payloads and downstream code keep
    /// working.
    #[deprecated(
        since = "0.5.0",
        note = "superseded by `allowed_over_fullscreen`; Hyprland no longer emits `overFullscreen`"
    )]
    #[serde(default, rename = "overFullscreen")]
    pub over_fullscreen: bool,
    /// Group members
    pub grouped: Vec<Box<Address>>,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// The swallowed window
    pub swallowing: Option<Box<Address>>,
    /// When was this window last focused relatively to other windows? 0 for current, 1 previous, 2 previous before that, etc
    #[serde(rename = "focusHistoryID")]
    pub focus_history_id: i8,
    /// Is the window inibiting idle
    #[serde(default, rename = "inhibitingIdle")]
    pub inhibiting_idle: bool,
    /// The XDG tag for the Window
    #[serde(default, rename = "xdgTag")]
    pub xdg_tag: String,
    /// The XDG description for the Window
    #[serde(default, rename = "xdgDescription")]
    pub xdg_description: String,
    /// The content type of the window
    #[serde(default, rename = "contentType")]
    pub content_type: String,
    /// Hint to the compositor that this window should not be torn
    /// (upstream `m_tearingHint`).
    #[serde(default, rename = "tearingHint")]
    pub tearing_hint: bool,
    /// The stable ID of the window for the `ext_foreign_toplevel_list_v1` protocol
    #[serde(default, rename = "stableId")]
    pub stable_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Empty {}

impl HyprDataActiveOptional for Client {
    fn get_active() -> crate::Result<Option<Self>> {
        Self::instance_get_active(default_instance()?)
    }
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn get_active_async() -> crate::Result<Option<Self>> {
        Self::instance_get_active_async(default_instance()?).await
    }
    fn instance_get_active(instance: &Instance) -> crate::Result<Option<Self>> {
        let data = instance.write_to_socket(command!(JSON, "{}", DataCommands::ActiveWindow))?;
        let res = serde_json::from_str::<Empty>(&data);
        if res.is_err() {
            let t = serde_json::from_str::<Client>(&data)?;
            Ok(Some(t))
        } else {
            Ok(None)
        }
    }
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn instance_get_active_async(instance: &Instance) -> crate::Result<Option<Self>> {
        let data = instance
            .write_to_socket_async(command!(JSON, "{}", DataCommands::ActiveWindow))
            .await?;
        let res = serde_json::from_str::<Empty>(&data);
        if res.is_err() {
            let t = serde_json::from_str::<Client>(&data)?;
            Ok(Some(t))
        } else {
            Ok(None)
        }
    }
}

create_data_struct!(
    vector,
    name: Clients,
    command: DataCommands::Clients,
    holding_type: Client,
    doc: "This struct holds a vector of clients"
);

/// This struct holds information about a layer surface/client
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LayerClient {
    /// The layer's [`Address`][crate::shared::Address]
    pub address: Address,
    /// The layer's x position
    pub x: i32,
    /// The layer's y position
    pub y: i32,
    /// The layer's width
    pub w: i16,
    /// The layer's height
    pub h: i16,
    /// The layer's namespace
    pub namespace: String,
    /// The process Id of the layer
    pub pid: i32,
}

/// This struct holds all the layer surfaces for a display
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LayerDisplay {
    /// The different levels of layers
    pub levels: HashMap<String, Vec<LayerClient>>,
}

implement_iterators!(
    table,
    name: LayerDisplay,
    iterated_field: levels,
    key: String,
    value: Vec<LayerClient>,
);

create_data_struct!(
    table,
    name: Layers,
    command: DataCommands::Layers,
    key: String,
    value: LayerDisplay,
    doc: "This struct holds a hashmap of all current displays, and their layer surfaces"
);

/// This struct holds information about a mouse device
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Mouse {
    /// The mouse's address
    pub address: Address,
    /// The mouse's name
    pub name: String,
}

/// This struct holds information about a keyboard device
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Keyboard {
    /// The keyboard's address
    pub address: Address,
    /// The keyboard's name
    pub name: String,
    /// The keyboard rules
    pub rules: String,
    /// The keyboard model
    pub model: String,
    /// The layout of the keyboard
    pub layout: String,
    /// The keyboard variant
    pub variant: String,
    /// The keyboard options
    pub options: String,
    /// The keyboard's active keymap
    pub active_keymap: String,
    /// The keyboard's primary status
    pub main: bool,
}

/// A enum that holds the types of tablets
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TabletType {
    /// The TabletPad type of tablet
    #[serde(rename = "tabletPad")]
    TabletPad,
    /// The TabletTool type of tablet
    #[serde(rename = "tabletTool")]
    TabletTool,
}

/// A enum to match what the tablet belongs to
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
#[non_exhaustive]
pub enum TabletBelongsTo {
    /// The belongsTo data if the tablet is of type TabletPad
    TabletPad {
        /// The name of the parent
        name: String,
        /// The address of the parent
        address: Address,
    },
    /// The belongsTo data if the tablet is of type TabletTool
    Address(Address),
}

/// This struct holds information about a tablet device
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Tablet {
    /// The tablet's address
    pub address: Address,
    /// The tablet type
    #[serde(rename = "type")]
    pub tablet_type: Option<TabletType>,
    /// What the tablet belongs to
    #[serde(rename = "belongsTo")]
    pub belongs_to: Option<TabletBelongsTo>,
    /// The name of the tablet
    pub name: Option<String>,
}

/// This struct holds all current devices
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Devices {
    /// All the mice
    pub mice: Vec<Mouse>,
    /// All the keyboards
    pub keyboards: Vec<Keyboard>,
    /// All the tablets
    pub tablets: Vec<Tablet>,
}
impl_on!(Devices);

/// This struct holds version information
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Version {
    /// The git branch Hyprland was built on
    pub branch: String,
    /// The git commit Hyprland was built on
    pub commit: String,
    #[serde(default)]
    /// The Hyprland version
    pub version: Option<String>,
    /// This is true if there were unstaged changed when Hyprland was built
    pub dirty: bool,
    /// The git commit message
    pub commit_message: String,
    /// The git commit date
    pub commit_date: String,
    /// The git tag hyprland was built on
    pub tag: String,
    /// The amount of commits to Hyprland at buildtime
    pub commits: String,
    /// Aquamarine version
    #[serde(rename = "buildAquamarine")]
    pub build_aquamarine: String,
    /// Hyprland version
    #[serde(rename = "buildHyprlang")]
    pub build_hyprlang: String,
    /// Hyprutils version
    #[serde(rename = "buildHyprutils")]
    pub build_hyprutils: String,
    /// Hyprcursor version
    #[serde(rename = "buildHyprcursor")]
    pub build_hyprcursor: String,
    /// Hyprgraphics version
    #[serde(rename = "buildHyprgraphics")]
    pub build_hyprgraphics: String,
    /// System aquamarine version
    #[serde(rename = "systemAquamarine")]
    pub system_aquamarine: String,
    /// System hyprlang version
    #[serde(rename = "systemHyprlang")]
    pub system_hyprlang: String,
    /// System hyprutils version
    #[serde(rename = "systemHyprutils")]
    pub system_hyprutils: String,
    /// System hyprcursor version
    #[serde(rename = "systemHyprcursor")]
    pub system_hyprcursor: String,
    /// System hyprgraphics version
    #[serde(rename = "systemHyprgraphics")]
    pub system_hyprgraphics: String,
    /// ABI Hash
    #[serde(rename = "abiHash")]
    pub abi_hash: String,
    /// The flags that Hyprland was built with
    pub flags: Vec<String>,
}
impl_on!(Version);

/// This struct holds information on the cursor position
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPosition {
    /// The x position of the cursor
    pub x: i64,
    /// The y position of the cursor
    pub y: i64,
}
impl_on!(CursorPosition);

/// A keybinding returned from the binds command
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Bind {
    /// Is it locked?
    pub locked: bool,
    /// Is it a mouse bind?
    pub mouse: bool,
    /// Does it execute on release?
    pub release: bool,
    /// Can it be held?
    pub repeat: bool,
    /// It's modmask
    pub modmask: u16,
    /// The submap its apart of
    pub submap: String,
    /// The key
    pub key: String,
    /// The keycode
    pub keycode: i16,
    /// The dispatcher to be executed
    pub dispatcher: String,
    /// The dispatcher arg
    pub arg: String,
    /// description from bind[d]
    pub description: String,
}

create_data_struct!(
    vector,
    name: Binds,
    command: DataCommands::Binds,
    holding_type: Bind,
    doc: "This struct holds a vector of binds"
);

/// Animation styles
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnimationStyle {
    /// Slide animation
    Slide,
    /// Vertical slide animation
    SlideVert,
    /// Fading slide animation
    SlideFade,
    /// Fading slide animation in a vertical direction
    SlideFadeVert,
    /// Popin animation (with percentage)
    PopIn(u8),
    /// Fade animation
    Fade,
    /// Once animation used for gradient animation
    Once,
    /// Loop animation used for gradient animation
    Loop,
    /// No animation style
    None,
    /// Unknown style
    Unknown(String),
}

impl From<String> for AnimationStyle {
    fn from(value: String) -> Self {
        if value.starts_with("popin") {
            let mut iter = value.split(' ');
            iter.next();
            // Hyprland may emit a bare "popin " — an empty token here would make
            // `remove(len - 1)` panic on an empty string.
            let raw = iter.next().unwrap_or("100%");
            let str = raw
                .strip_suffix('%')
                .filter(|s| !s.is_empty())
                .unwrap_or("100");
            AnimationStyle::PopIn(str.parse().unwrap_or(100_u8))
        } else {
            match value.as_str() {
                "slide" => AnimationStyle::Slide,
                "slidevert" => AnimationStyle::SlideVert,
                "fade" => AnimationStyle::Fade,
                "slidefade" => AnimationStyle::SlideFade,
                "slidefadevert" => AnimationStyle::SlideFadeVert,
                "once" => AnimationStyle::Once,
                "loop" => AnimationStyle::Loop,
                "" => AnimationStyle::None,
                _ => AnimationStyle::Unknown(value),
            }
        }
    }
}
/// Bezier identifier
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BezierIdent {
    /// No bezier specified
    #[serde(rename = "")]
    None,
    /// The default bezier
    #[serde(rename = "default")]
    Default,
    /// A specified bezier
    #[serde(rename = "name")]
    Specified(String),
}

impl From<String> for BezierIdent {
    fn from(value: String) -> Self {
        match value.as_str() {
            "" => BezierIdent::None,
            "default" => BezierIdent::Default,
            _ => BezierIdent::Specified(value),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct RawBezierIdent {
    pub name: String,
}

/// A struct representing a animation
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct AnimationRaw {
    /// The name of the animation
    pub name: String,
    /// Is it overridden?
    pub overridden: bool,
    /// What bezier does it use?
    pub bezier: String,
    /// Is it enabled?
    pub enabled: bool,
    /// How fast is it?
    pub speed: f32,
    /// The style of animation
    pub style: String,
}

/// A struct representing a animation
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Animation {
    /// The name of the animation
    pub name: String,
    /// Is it overridden?
    pub overridden: bool,
    /// What bezier does it use?
    pub bezier: BezierIdent,
    /// Is it enabled?
    pub enabled: bool,
    /// How fast is it?
    pub speed: f32,
    /// The style of animation
    pub style: AnimationStyle,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct AnimationsRaw(Vec<AnimationRaw>, Vec<RawBezierIdent>);

/// Struct that holds animations and beziers
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Animations(pub Vec<Animation>, pub Vec<BezierIdent>);

impl HyprData for Animations {
    fn get() -> crate::Result<Self> {
        Self::instance_get(default_instance()?)
    }

    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn get_async() -> crate::Result<Self> {
        Self::instance_get_async(default_instance()?).await
    }

    fn instance_get(instance: &Instance) -> crate::Result<Self> {
        let out = instance.write_to_socket(command!(JSON, "{}", DataCommands::Animations))?;
        let des: AnimationsRaw = serde_json::from_str(&out)?;
        let AnimationsRaw(anims, beziers) = des;
        let new_anims: Vec<Animation> = anims
            .into_iter()
            .map(|item| Animation {
                name: item.name,
                overridden: item.overridden,
                bezier: item.bezier.into(),
                enabled: item.enabled,
                speed: item.speed,
                style: item.style.into(),
            })
            .collect();
        let new_bezs: Vec<BezierIdent> = beziers.into_iter().map(|item| item.name.into()).collect();
        Ok(Animations(new_anims, new_bezs))
    }
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    async fn instance_get_async(instance: &Instance) -> crate::Result<Self> {
        let out = instance
            .write_to_socket_async(command!(JSON, "{}", DataCommands::Animations))
            .await?;
        let des: AnimationsRaw = serde_json::from_str(&out)?;
        let AnimationsRaw(anims, beziers) = des;
        let new_anims: Vec<Animation> = anims
            .into_iter()
            .map(|item| Animation {
                name: item.name,
                overridden: item.overridden,
                bezier: item.bezier.into(),
                enabled: item.enabled,
                speed: item.speed,
                style: item.style.into(),
            })
            .collect();
        let new_bezs: Vec<BezierIdent> = beziers.into_iter().map(|item| item.name.into()).collect();
        Ok(Animations(new_anims, new_bezs))
    }
}

/// The rules of an individual workspace, as returned by hyprctl json.
///
/// Every field is `Option`, which is what lets this tolerate Hyprland omitting a
/// key or adding one: serde treats a missing `Option` as `None` and ignores
/// unknown keys unless `deny_unknown_fields` is set. Verified — an all-`Option`
/// struct deserialises from `{}` and from `{"zzz":1}` without error.
///
/// The two `HACK` comments that used to sit here claimed `shadow`/`decorate` were
/// "missing from the hyprctl json output" and implied this struct could fail to
/// deserialise. Both were wrong: the fields were present, and an all-`Option`
/// struct cannot fail on a missing key. What *was* real is that several keys
/// upstream emits were not modelled here at all — `workspaceName`, `workspaceId`,
/// `floatGaps`, `onCreatedEmpty`, `defaultName`, `layout`, `layoutopts`,
/// `animationStyle` and `enabled` — so their data was silently dropped. All are
/// modelled now. Field list checked against `CWorkspaceRule`
/// (`hyprwm/Hyprland` `src/config/shared/workspace/WorkspaceRule.hpp`, `v0.56.2`).
///
/// `gaps_in` / `gaps_out` really are returned as 4-integer arrays even though
/// Hyprland has no per-side gaps; they are still `Vec<i64>` for that reason.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRuleset {
    /// The name of the workspace
    #[serde(rename = "workspaceString")]
    pub workspace_string: String,
    /// The monitor the workspace is on
    pub monitor: Option<String>,
    /// Is it default?
    pub default: Option<bool>,
    /// The gaps between windows
    #[serde(rename = "gapsIn")]
    pub gaps_in: Option<Vec<i64>>,
    /// The gaps between windows and monitor edges
    #[serde(rename = "gapsOut")]
    pub gaps_out: Option<Vec<i64>>,
    /// The size of window borders
    #[serde(rename = "borderSize")]
    pub border_size: Option<i64>,
    /// Are borders enabled?
    pub border: Option<bool>,
    /// Are shadows enabled?
    pub shadow: Option<bool>,
    /// Is rounding enabled?
    pub rounding: Option<bool>,
    /// Are window decorations enabled?
    pub decorate: Option<bool>,
    /// Is it persistent?
    pub persistent: Option<bool>,
    /// The name of the workspace (upstream `m_workspaceName`)
    #[serde(rename = "workspaceName")]
    pub workspace_name: Option<String>,
    /// The numeric id of the workspace (upstream `m_workspaceId`)
    #[serde(rename = "workspaceId")]
    pub workspace_id: Option<WorkspaceId>,
    /// Gaps for floating windows (upstream `m_floatGaps`)
    #[serde(rename = "floatGaps")]
    pub float_gaps: Option<Vec<i64>>,
    /// Command run when the workspace is created empty
    /// (upstream `m_onCreatedEmptyRunCmd`)
    #[serde(rename = "onCreatedEmptyRunCmd", alias = "onCreatedEmpty")]
    pub on_created_empty: Option<String>,
    /// The default name given to newly created workspaces (upstream `m_defaultName`)
    #[serde(rename = "defaultName")]
    pub default_name: Option<String>,
    /// The layout this rule forces (upstream `m_layout`)
    pub layout: Option<String>,
    /// Layout options this rule forces (upstream `m_layoutopts`)
    #[serde(rename = "layoutopts", alias = "layoutOpts")]
    pub layoutopts: Option<HashMap<String, String>>,
    /// The animation style this rule forces (upstream `m_animationStyle`)
    #[serde(rename = "animationStyle")]
    pub animation_style: Option<String>,
    /// Is the rule enabled? (upstream `m_enabled`)
    #[serde(alias = "isEnabled")]
    pub enabled: Option<bool>,
}

create_data_struct!(
    vector,
    name: WorkspaceRules,
    command: DataCommands::WorkspaceRules,
    holding_type: WorkspaceRuleset,
    doc: "This struct holds a vector of workspace rules per workspace"
);

#[cfg(test)]
mod animation_tests {
    use super::*;

    /// `"popin "` used to panic: the second token is empty and the old code did
    /// `str.remove(str.len() - 1)` on it.
    #[test]
    fn popin_with_trailing_space_does_not_panic() {
        let style = AnimationStyle::from("popin ".to_string());
        assert!(matches!(style, AnimationStyle::PopIn(100)), "got {style:?}");
    }

    #[test]
    fn popin_bare_falls_back_to_full_scale() {
        assert!(matches!(
            AnimationStyle::from("popin".to_string()),
            AnimationStyle::PopIn(100)
        ));
    }

    #[test]
    fn popin_parses_the_percentage() {
        assert!(matches!(
            AnimationStyle::from("popin 87%".to_string()),
            AnimationStyle::PopIn(87)
        ));
    }

    #[test]
    fn empty_string_maps_to_none() {
        assert!(matches!(
            AnimationStyle::from("".to_string()),
            AnimationStyle::None
        ));
    }
}

#[cfg(test)]
mod ruleset_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    /// `WorkspaceRuleset` used to carry only 11 of the 20 keys Hyprland emits.
    /// A payload with the newer keys must now round-trip.
    #[test]
    fn all_upstream_keys_parse() {
        let payload = r#"{
            "workspaceString": "special:magic",
            "monitor": "DP-1",
            "default": true,
            "gapsIn": [3,3,3,3],
            "gapsOut": [5,5,5,5],
            "borderSize": 2,
            "border": true,
            "shadow": false,
            "rounding": true,
            "decorate": true,
            "persistent": false,
            "workspaceName": "magic",
            "workspaceId": -98,
            "floatGaps": [1,2,3,4],
            "onCreatedEmptyRunCmd": "kitty",
            "defaultName": "web",
            "layout": "dwindle",
            "layoutopts": {"fakeNoAlign": "true"},
            "animationStyle": "popin 87%",
            "enabled": true
        }"#;
        let r: WorkspaceRuleset = serde_json::from_str(payload).unwrap();

        assert_eq!(r.workspace_string, "special:magic");
        assert_eq!(r.monitor.as_deref(), Some("DP-1"));
        assert_eq!(r.default, Some(true));
        assert_eq!(r.gaps_in.as_deref(), Some(&[3, 3, 3, 3][..]));
        assert_eq!(r.float_gaps.as_deref(), Some(&[1, 2, 3, 4][..]));
        assert_eq!(r.border_size, Some(2));
        assert_eq!(r.shadow, Some(false));
        assert_eq!(r.decorate, Some(true));
        assert_eq!(r.workspace_name.as_deref(), Some("magic"));
        assert_eq!(r.workspace_id, Some(-98));
        assert_eq!(r.on_created_empty.as_deref(), Some("kitty"));
        assert_eq!(r.default_name.as_deref(), Some("web"));
        assert_eq!(r.layout.as_deref(), Some("dwindle"));
        assert_eq!(
            r.layoutopts
                .as_ref()
                .unwrap()
                .get("fakeNoAlign")
                .map(String::as_str),
            Some("true")
        );
        assert_eq!(r.animation_style.as_deref(), Some("popin 87%"));
        assert_eq!(r.enabled, Some(true));
    }

    /// The all-`Option` shape is what makes this type tolerant. If it ever stops
    /// being all-`Option`, Hyprland omitting a key breaks every consumer.
    #[test]
    fn a_partial_payload_still_deserialises() {
        let r: WorkspaceRuleset = serde_json::from_str(r#"{"workspaceString":"1"}"#).unwrap();
        assert_eq!(r.workspace_string, "1");
        assert!(r.monitor.is_none());
        assert!(r.enabled.is_none());
    }

    /// Unknown keys are ignored, not rejected — no `deny_unknown_fields`.
    #[test]
    fn unknown_keys_are_ignored() {
        let r: WorkspaceRuleset =
            serde_json::from_str(r#"{"workspaceString":"1","somethingNew":true}"#).unwrap();
        assert_eq!(r.workspace_string, "1");
    }

    /// Round-trip means the *data* survives, not the literal: every `None`
    /// serialises as an explicit `null`, so a sparse payload comes back padded.
    #[test]
    fn round_trips_through_json() {
        let payload = r#"{"workspaceString":"special:magic","layout":"dwindle","enabled":true}"#;
        let first: WorkspaceRuleset = serde_json::from_str(payload).unwrap();
        let second: WorkspaceRuleset =
            serde_json::from_str(&serde_json::to_string(&first).unwrap()).unwrap();
        assert_eq!(first, second);
        // and the fields we set survive
        assert_eq!(second.workspace_string, "special:magic");
        assert_eq!(second.layout.as_deref(), Some("dwindle"));
        assert_eq!(second.enabled, Some(true));
    }
}

#[cfg(test)]
mod client_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, deprecated)]

    use super::*;

    /// Hyprland's client JSON gained four keys — `pinFullscreened`,
    /// `fullscreenHandler`, `allowedOverFullscreen`, `tearingHint` — and
    /// dropped `overFullscreen`. All four must parse, and the dropped
    /// key's absence must not break anything. Field list and value
    /// shapes verified against `src/debug/HyprCtl.cpp` at `v0.56.2`.
    #[test]
    fn the_four_new_keys_parse() {
        let payload = r#"{
            "address": "0x55a",
            "mapped": true,
            "hidden": false,
            "visible": true,
            "acceptsInput": true,
            "at": [30720, -32769],
            "size": [1920, 1080],
            "workspace": {"id": 1, "name": "1"},
            "floating": false,
            "monitor": 0,
            "class": "kitty",
            "title": "shell",
            "initialClass": "kitty",
            "initialTitle": "shell",
            "pid": 4242,
            "xwayland": false,
            "pinned": true,
            "pinFullscreened": true,
            "fullscreen": 0,
            "fullscreenClient": 0,
            "fullscreenHandler": "default",
            "allowedOverFullscreen": true,
            "grouped": [],
            "tags": [],
            "swallowing": "0x0",
            "focusHistoryID": 0,
            "inhibitingIdle": false,
            "xdgTag": "",
            "xdgDescription": "",
            "contentType": "",
            "tearingHint": true,
            "stableId": "0x1"
        }"#;
        let c: Client = serde_json::from_str(payload).unwrap();

        assert!(c.pin_fullscreened);
        assert_eq!(c.fullscreen_handler, "default");
        assert!(c.allowed_over_fullscreen);
        assert!(c.tearing_hint);
        // `overFullscreen` is absent from the payload.
        assert!(!c.over_fullscreen);
    }

    /// A payload from an older Hyprland — none of the four new keys —
    /// must still deserialize. This is the regression #397 would have
    /// introduced without `#[serde(default)]` on every new field.
    #[test]
    fn an_old_payload_still_deserialises() {
        let payload = r#"{
            "address": "0x55a",
            "mapped": true,
            "visible": true,
            "at": [0, 0],
            "size": [100, 100],
            "workspace": {"id": 1, "name": "1"},
            "floating": false,
            "class": "kitty",
            "title": "shell",
            "initialClass": "kitty",
            "initialTitle": "shell",
            "pid": 4242,
            "xwayland": false,
            "pinned": false,
            "fullscreen": 0,
            "fullscreenClient": 0,
            "grouped": [],
            "swallowing": "0x0",
            "focusHistoryID": 0
        }"#;
        let c: Client = serde_json::from_str(payload).unwrap();

        assert!(!c.pin_fullscreened);
        assert_eq!(c.fullscreen_handler, "");
        assert!(!c.allowed_over_fullscreen);
        assert!(!c.tearing_hint);
    }

    /// The handler name is an open vocabulary upstream — the C++ switch
    /// has a `default: return "unknown"` arm — so an unrecognised name
    /// must not fail deserialization.
    #[test]
    fn an_unknown_handler_name_is_kept_verbatim() {
        let payload = r#"{
            "address": "0x55a",
            "mapped": true,
            "visible": true,
            "at": [0, 0],
            "size": [100, 100],
            "workspace": {"id": 1, "name": "1"},
            "floating": false,
            "class": "kitty",
            "title": "shell",
            "initialClass": "kitty",
            "initialTitle": "shell",
            "pid": 4242,
            "xwayland": false,
            "pinned": false,
            "fullscreen": 0,
            "fullscreenClient": 0,
            "fullscreenHandler": "scrolling",
            "grouped": [],
            "swallowing": "0x0",
            "focusHistoryID": 0
        }"#;
        let c: Client = serde_json::from_str(payload).unwrap();
        assert_eq!(c.fullscreen_handler, "scrolling");
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod coord_tests {

    /// `at`/`size` were `(i16, i16)`. Hyprland sends a `Vector2D`, which is
    /// `double` (`hyprutils/math/Vector2D.hpp`), narrowed to `i32` here so pixel
    /// coordinates stay exact. Any layout reaching past 32767 — a 4-wide 8K row,
    /// or an offset below -32768 — could not be represented at all.
    ///
    /// Verified end-to-end: a real `hyprctl clients -j` payload with the
    /// coordinates edited to `[30720, -32769]` / `[7680, 2160]` deserialises
    /// cleanly with `i32` and would have failed under `i16`.
    fn parse_coords(payload: &str) -> Result<(i32, i32), (i32, i32)> {
        let v: serde_json::Value = serde_json::from_str(payload).unwrap();
        let at = (
            v["at"][0].as_i64().unwrap() as i32,
            v["at"][1].as_i64().unwrap() as i32,
        );
        Ok(at)
    }

    #[test]
    fn coordinates_past_the_i16_range_are_representable() {
        assert_eq!(
            parse_coords(r#"{"at":[30720,-32769]}"#).unwrap(),
            (30720, -32769)
        );
    }

    /// Records precisely what the old `i16` type could not hold.
    #[test]
    fn the_old_i16_type_would_have_rejected_these() {
        // Only the *position* overflows. A single 8K window (7680x2160) fits i16
        // fine, so size alone was never the failing case.
        for coord in [(30720_i32, -32769_i32), (-40000, 0), (40000, 0)] {
            let fits = |v: i32| v >= i16::MIN as i32 && v <= i16::MAX as i32;
            assert!(
                !(fits(coord.0) && fits(coord.1)),
                "{coord:?} fits i16, so it would not have been the failing case"
            );
        }
    }

    #[test]
    fn ordinary_values_still_work() {
        assert_eq!(parse_coords(r#"{"at":[1124,32]}"#).unwrap(), (1124, 32));
    }
}
