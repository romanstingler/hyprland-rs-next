//! # Dispatch module
//!
//! This module is used for calling dispatchers and changing keywords
//!
//! ## Usage
//!
//! ```rust,no_run
//! use hyprland::Result;
//! use hyprland::dispatch::{Dispatch, DispatchType};
//! fn main() -> Result<()> {
//!     Dispatch::call(DispatchType::Exec("kitty"))?;
//!
//!    Ok(())
//! }
//! ```

use crate::default_instance;
use crate::dispatch::fmt::*;
use crate::error::HyprError;
use crate::shared::*;
use derive_more::Display;
use std::string::ToString;

/// This enum is for identifying a window
#[derive(Debug, Clone, Display)]
#[non_exhaustive]
pub enum WindowIdentifier<'a> {
    /// The address of a window
    #[display("address:{_0}")]
    Address(Address),
    /// A Regular Expression to match the window class (handled by Hyprland)
    #[display("class:{_0}")]
    ClassRegularExpression(&'a str),
    /// The window title
    #[display("title:{_0}")]
    Title(&'a str),
    /// A window tag regex
    #[display("tag:{_0}")]
    Tag(&'a str),
    /// The window's process Id
    #[display("pid:{_0}")]
    ProcessId(u32),
    /// The active window
    #[display("activewindow")]
    ActiveWindow,
    /// The first floating window
    #[display("floating")]
    Floating,
    /// The first tiled window
    #[display("tiled")]
    Tiled,
}

/// This enum holds the fullscreen types
#[derive(Debug, Clone, Display)]
#[non_exhaustive]
pub enum FullscreenType {
    /// Fills the whole screen
    #[display("0")]
    Real,
    /// Maximizes the window
    #[display("1")]
    Maximize,
    /// Passes no param
    #[display("")]
    NoParam,
}

/// This enum holds the params to the [DispatchType::ToggleFullscreenState] dispatcher
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum FullscreenState {
    Current = -1,
    None = 0,
    Maximize = 1,
    Fullscreen = 2,
    MaximizeFullscreen = 3,
}

impl std::fmt::Display for FullscreenState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as i8)
    }
}

/// This enum holds directions, typically used for moving
#[derive(Debug, Clone, Display)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum Direction {
    #[display("u")]
    Up,
    #[display("d")]
    Down,
    #[display("r")]
    Right,
    #[display("l")]
    Left,
}

/// This enum is used for resizing and moving windows precisely
#[derive(Debug, Clone, Display)]
#[non_exhaustive]
pub enum Position {
    /// A delta in pixels
    #[display("{_0} {_1}")]
    Delta(i16, i16),
    /// The exact size in pixels
    #[display("exact {_0} {_1}")]
    Exact(i16, i16),
    /// A delta in window fraction
    #[display("{_0}% {_1}%")]
    DeltaFraction(i16, i16),
    /// The exact size in screen fraction
    #[display("exact {_0}% {_1}%")]
    ExactFraction(i16, i16),
}

/// This enum holds a direction for cycling
#[allow(missing_docs)]
#[derive(Debug, Clone, Display)]
#[non_exhaustive]
pub enum CycleDirection {
    #[display("")]
    Next,
    #[display("prev")]
    Previous,
}

/// This enum holds a direction for switch windows in a group
#[allow(missing_docs)]
#[derive(Debug, Clone, Display)]
#[non_exhaustive]
pub enum WindowSwitchDirection {
    #[display("b")]
    Back,
    #[display("f")]
    Forward,
    #[display("{}", _0)]
    Index(i32),
}

/// This enum is used for identifying monitors
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum MonitorIdentifier<'a> {
    /// The monitor that is to the specified direction of the active one
    Direction(Direction),
    /// The monitor id
    Id(MonitorId),
    /// The monitor name
    Name(&'a str),
    /// The current monitor
    Current,
    /// The workspace relative to the current workspace
    Relative(i32),
}

impl std::fmt::Display for MonitorIdentifier<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            MonitorIdentifier::Direction(dir) => dir.to_string(),
            MonitorIdentifier::Id(id) => id.to_string(),
            MonitorIdentifier::Name(name) => name.to_string(),
            MonitorIdentifier::Current => "current".to_string(),
            MonitorIdentifier::Relative(int) => format_relative(*int, ""),
        };
        write!(f, "{out}")
    }
}

/// This enum holds corners
#[allow(missing_docs)]
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Corner {
    BottomLeft = 0,
    BottomRight = 1,
    TopRight = 2,
    TopLeft = 3,
}

/// This enum holds options that are applied to the current workspace
#[derive(Debug, Clone, Display)]
#[non_exhaustive]
pub enum WorkspaceOptions {
    /// Makes all windows pseudo tiled
    #[display("allpseudo")]
    AllPseudo,
    /// Makes all windows float
    #[display("allfloat")]
    AllFloat,
}

/// This struct holds options for the first empty workspace
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirstEmpty {
    /// If the first empty workspace should be on the monitor
    pub on_monitor: bool,
    /// If the first empty workspace should be next
    pub next: bool,
}

impl std::fmt::Display for FirstEmpty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        if self.on_monitor {
            s.push('m');
        }
        if self.next {
            s.push('n');
        }
        write!(f, "{s}")
    }
}

/// This enum is for identifying workspaces that also includes the special workspace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[non_exhaustive]
pub enum WorkspaceIdentifierWithSpecial<'a> {
    /// The workspace Id
    Id(WorkspaceId),
    /// The workspace relative to the current workspace
    #[display("{}", format_relative(*_0, ""))]
    Relative(i32),
    /// The workspace on the monitor relative to the current workspace
    #[display("{}", format_relative(*_0, "m"))]
    RelativeMonitor(i32),
    /// The workspace on the monitor relative to the current workspace, including empty workspaces
    #[display("{}", format_relative(*_0, "r"))]
    RelativeMonitorIncludingEmpty(i32),
    /// The open workspace relative to the current workspace
    #[display("{}", format_relative(*_0, "e"))]
    RelativeOpen(i32),
    /// The previous Workspace
    #[display("previous")]
    Previous,
    /// The previous Workspace
    #[display("previous_per_monitor")]
    PreviousPerMonitor,
    /// The first available empty workspace
    #[display("{}", format!("empty{}", _0))]
    Empty(FirstEmpty),
    /// The name of the workspace
    #[display("name:{_0}")]
    Name(&'a str),
    /// The special workspace
    #[display("special{}", format_special_workspace_ident(_0))]
    Special(Option<&'a str>),
}

pub(super) mod fmt {
    #[inline(always)]
    pub(super) fn format_special_workspace_ident<'a>(opt: &'a Option<&'a str>) -> String {
        match opt {
            Some(o) => ":".to_owned() + o,
            None => String::new(),
        }
    }

    #[inline(always)]
    pub(super) fn format_relative(int: i32, extra: &'_ str) -> String {
        // `0` must keep the prefix (`m+0`), not collapse to a bare `+0`
        if int >= 0 {
            format!("{extra}+{int}")
        } else {
            format!("{extra}-{}", int.abs())
        }
    }
}

/// This enum is for identifying workspaces
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WorkspaceIdentifier<'a> {
    /// The workspace Id
    Id(WorkspaceId),
    /// The workspace relative to the current workspace
    Relative(i32),
    /// The workspace on the monitor relative to the current workspace
    RelativeMonitor(i32),
    /// The workspace on the monitor relative to the current workspace, including empty workspaces
    RelativeMonitorIncludingEmpty(i32),
    /// The open workspace relative to the current workspace
    RelativeOpen(i32),
    /// The previous Workspace
    Previous,
    /// The first available empty workspace
    Empty,
    /// The name of the workspace
    Name(&'a str),
}

impl std::fmt::Display for WorkspaceIdentifier<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use WorkspaceIdentifier::*;
        let out = match self {
            Id(id) => format!("{id}"),
            Name(name) => format!("name:{name}"),
            Relative(int) => format_relative(*int, ""),
            RelativeMonitor(int) => format_relative(*int, "m"),
            RelativeMonitorIncludingEmpty(int) => format_relative(*int, "r"),
            RelativeOpen(int) => format_relative(*int, "e"),
            Previous => "previous".to_string(),
            Empty => "empty".to_string(),
        };

        write!(f, "{out}")
    }
}

/// This enum is the params to [DispatchType::MoveWindow] dispatcher
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum WindowMove<'a> {
    /// Moves the window to a specified monitor
    Monitor(MonitorIdentifier<'a>),
    /// Moves the window in a specified direction
    Direction(Direction),
}

/// This enum holds the actions that can be applied to a tag
#[derive(Debug, Clone, Display)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum TagAction {
    #[display("+")]
    Add,
    #[display("-")]
    Remove,
    #[display("")]
    Toggle,
}

/// This enum holds the signals
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum SignalType {
    /// Hangup detected on controlling terminal
    SIGHUP = 1,
    /// Interrupt from keyboard
    SIGINT = 2,
    /// Quit from keyboard
    SIGQUIT = 3,
    /// Illegal Instruction
    SIGILL = 4,
    /// Trace/breakpoint trap
    SIGTRAP = 5,
    /// Abort signal from abort
    SIGABRT = 6,
    /// Bus error (bad memory access)
    SIGBUS = 7,
    /// Erroneous arithmetic operation
    SIGFPE = 8,
    /// Kill signal
    SIGKILL = 9,
    /// User-defined signal 1
    SIGUSR1 = 10,
    /// Invalid memory reference
    SIGSEGV = 11,
    /// User-defined signal 2
    SIGUSR2 = 12,
    /// Broken pipe
    SIGPIPE = 13,
    /// Timer signal from alarm
    SIGALRM = 14,
    /// Termination signal
    SIGTERM = 15,
    /// Stack fault on coprocessor
    SIGSTKFLT = 16,
    /// Child stopped, terminated, or continued
    SIGCHLD = 17,
    /// Continue if stopped
    SIGCONT = 18,
    /// Stop process
    SIGSTOP = 19,
    /// Stop typed at terminal
    SIGTSTP = 20,
    /// Terminal input for background process
    SIGTTIN = 21,
    /// Terminal output for background process
    SIGTTOU = 22,
    /// Urgent condition on socket
    SIGURG = 23,
    /// CPU time limit exceeded
    SIGXCPU = 24,
    /// File size limit exceeded
    SIGXFSZ = 25,
    /// Virtual alarm clock
    SIGVTALRM = 26,
    /// Profiling timer expired
    SIGPROF = 27,
    /// Window resize signal
    SIGWINCH = 28,
    /// I/O now possible
    SIGIO = 29,
    /// Power failure
    SIGPWR = 30,
    /// Bad system call
    SIGSYS = 31,
}

impl std::fmt::Display for SignalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

/// The state of a synthetic key event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[non_exhaustive]
pub enum KeyState {
    /// Key pressed down
    #[display("down")]
    Down,
    /// Key released
    #[display("up")]
    Up,
    /// Key repeat
    #[display("repeat")]
    Repeat,
}

#[derive(Debug, Clone, Copy, Display)]
/// This enum holds the params to the [DispatchType::MoveToRoot] dispatcher
#[non_exhaustive]
pub enum MoveToRootParam {
    /// Maximize the window in its current subtree
    #[display("")]
    Stable,
    /// Swap the window with the other subtree
    #[display("unstable")]
    Unstable,
}

/// This enum holds the zheight variants
#[derive(Debug, Clone, Copy, Display)]
#[non_exhaustive]
pub enum ZOrder {
    /// Bring the active window to top of the stack
    #[display("top")]
    Top,
    /// Bring the active window to bottom of the stack
    #[display("bottom")]
    Bottom,
}

/// This enum holds the params to the [DispatchType::Submap] dispatcher
#[derive(Debug, Clone, Copy, Display)]
#[non_exhaustive]
pub enum SubmapParam<'a> {
    /// Go back to global submap
    #[display("reset")]
    Reset,
    /// Go to named submap
    #[display("{}", _0)]
    Name(&'a str),
}

/// This enum holds every dispatcher
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum DispatchType<'a> {
    /// This lets you use dispatchers not supported by hyprland-rs yet, please make issues before
    /// using
    Custom(
        /// Name of event
        &'a str,
        /// Args
        &'a str,
    ),
    /// A raw Lua dispatcher expression, for Lua sessions only.
    Lua(&'a str),
    /// This dispatcher changes the current cursor
    SetCursor(
        /// The cursor theme
        &'a str,
        /// The size
        u16,
    ),
    /// This dispatcher executes a program
    Exec(&'a str),
    /// This dispatcher executes a raw shell command ignoring window rules
    ExecRaw(&'a str),
    /// This dispatcher passes a keybind to a window when called in a
    /// keybind, its used for global keybinds. And should **ONLY** be used with keybinds
    Pass(WindowIdentifier<'a>),
    /// Executes a Global Shortcut using the GlobalShortcuts portal.
    Global(&'a str),
    /// Emits a named event
    Event(&'a str),
    /// Forces the idle inhibitor for the given duration
    ForceIdle(f32),
    /// Releases the current input capture
    ReleaseInputCapture,
    /// Does nothing
    NoOp,
    /// This dispatcher closes the active window/client
    KillActiveWindow,
    /// This dispatcher kills the active window/client
    ForceKillActiveWindow,
    /// This dispatcher closes the specified window
    CloseWindow(WindowIdentifier<'a>),
    /// This dispatcher changes the current workspace
    Workspace(WorkspaceIdentifierWithSpecial<'a>),
    /// This dispatcher moves a window (focused if not specified) to a workspace
    MoveToWorkspace(
        WorkspaceIdentifierWithSpecial<'a>,
        Option<WindowIdentifier<'a>>,
    ),
    /// This dispatcher moves a window (focused if not specified) to a workspace, without switching to that
    /// workspace
    MoveToWorkspaceSilent(
        WorkspaceIdentifierWithSpecial<'a>,
        Option<WindowIdentifier<'a>>,
    ),
    /// This dispatcher toggles the floating state of a window (current if not specified)
    ToggleFloating(Option<WindowIdentifier<'a>>),
    /// This dispatcher floats a window (current if not specified)
    SetFloating(Option<WindowIdentifier<'a>>),
    /// This dispatcher tiles a window (current if not specified)
    SetTiled(Option<WindowIdentifier<'a>>),
    /// This dispatcher toggles the current window fullscreen state
    ToggleFullscreen(FullscreenType),
    /// This dispatcher sets the focused window’s fullscreen mode and the one sent to the client
    ToggleFullscreenState(FullscreenState, FullscreenState),
    /// This dispatcher toggles the focused window’s internal
    /// fullscreen state without altering the geometry
    ToggleFakeFullscreen,
    /// This dispatcher sets the DPMS status for all monitors
    ToggleDPMS(bool, Option<&'a str>),
    /// This dispatcher toggles pseudo tiling for the current window
    TogglePseudo,
    /// This dispatcher pins the active window to all workspaces
    TogglePin,
    /// This dispatcher pins the specified window to all workspaces
    TogglePinWindow(WindowIdentifier<'a>),
    /// This dispatcher sends a shortcut to the specified window
    SendShortcut(
        /// The modifiers
        &'a [crate::shared::Mod],
        /// The key, e.g., "A"
        &'a str,
        /// The window identifier
        Option<WindowIdentifier<'a>>,
    ),
    /// Sends a synthetic key state
    SendKeyState(
        /// The modifiers
        &'a [crate::shared::Mod],
        /// The key, e.g., "A"
        &'a str,
        /// The key state
        KeyState,
        /// The window identifier
        Option<WindowIdentifier<'a>>,
    ),
    /// This dispatcher sends a signal to the active window
    Signal(SignalType),
    /// This dispatcher sends a signal to the specified window
    SignalWindow(WindowIdentifier<'a>, SignalType),
    /// This dispatcher moves the window focus in a specified direction
    MoveFocus(Direction),
    /// This dispatcher moves the current window to a monitor or in a specified direction
    MoveWindow(WindowMove<'a>),
    /// This dispatcher centers the active window
    CenterWindow,
    /// Drags the active window (mouse binds)
    WindowDrag,
    /// Resizes the active window interactively (mouse binds)
    WindowResize,
    /// Resizes interactively, forcing or blocking the aspect ratio
    ResizeWindowKeepAspectRatio(bool),
    /// This dispatcher resizes the active window using a [Position] enum
    ResizeActive(Position),
    /// This dispatcher moves the active window using a [Position] enum
    MoveActive(Position),
    /// This dispatcher resizes the specified window using a [Position] enum
    ResizeWindowPixel(Position, WindowIdentifier<'a>),
    /// This dispatcher moves the specified window using a [Position] enum
    MoveWindowPixel(Position, WindowIdentifier<'a>),
    /// This dispatcher cycles windows using a specified direction
    CycleWindow(CycleDirection),
    /// This dispatcher cycles windows with tiled/floating filters
    CycleWindowFiltered(CycleDirection, Option<bool>, Option<bool>),
    /// This dispatcher swaps the focused window with the window on a workspace using a specified direction
    SwapNext(CycleDirection),
    /// This dispatcher swaps windows using a specified direction
    SwapWindow(Direction),
    /// This dispatcher swaps the active window with a specific window
    SwapWindowWith(WindowIdentifier<'a>),
    /// Apply tag to current or the first window matching
    TagWindow(TagAction, &'a str, Option<WindowIdentifier<'a>>),
    /// Clears all tags from the specified window
    ClearTags(Option<WindowIdentifier<'a>>),
    /// Sets a window property
    SetProp(&'a str, &'a str, Option<WindowIdentifier<'a>>),
    /// This dispatcher focuses a specified window
    FocusWindow(WindowIdentifier<'a>),
    /// This dispatcher focuses a specified monitor
    FocusMonitor(MonitorIdentifier<'a>),
    /// This dispatcher changed the split ratio
    ChangeSplitRatio(FloatValue),
    /// This dispatcher toggle opacity for the current window/client
    ToggleOpaque,
    /// This dispatcher moves the cursor to a specified corner of a window
    MoveCursorToCorner(Corner),
    /// This dispatcher moves the cursor to a specified position
    /// (x, y) where x starts from left to right, and y starts from top to bottom
    MoveCursor(i64, i64),
    /// This dispatcher applied a option to all windows in a workspace
    WorkspaceOption(WorkspaceOptions),
    /// This dispatcher renames a workspace
    RenameWorkspace(WorkspaceId, Option<&'a str>),
    /// This dispatcher changes the numeric id of a workspace
    ChangeWorkspaceId(WorkspaceIdentifierWithSpecial<'a>, u32),
    /// This exits Hyprland **(DANGEROUS)**
    Exit,
    /// This dispatcher forces the renderer to reload
    ForceRendererReload,
    /// This dispatcher moves the current workspace to a specified monitor
    MoveCurrentWorkspaceToMonitor(MonitorIdentifier<'a>),
    /// This dispatcher moves a specified workspace to a specified monitor
    MoveWorkspaceToMonitor(WorkspaceIdentifier<'a>, MonitorIdentifier<'a>),
    /// This dispatcher swaps the active workspaces of two monitors
    SwapActiveWorkspaces(MonitorIdentifier<'a>, MonitorIdentifier<'a>),
    /// This dispatcher brings the active window to the top of the stack
    BringActiveToTop,
    /// This dispatcher brings the active window to the top or bottom of the stack
    AlterZOrder(ZOrder, Option<WindowIdentifier<'a>>),
    /// This toggles the special workspace (AKA scratchpad)
    ToggleSpecialWorkspace(Option<String>),
    /// This dispatcher jump to urgent or the last window
    FocusUrgentOrLast,
    /// Switch focus from current to previously focused window
    FocusCurrentOrLast,
    /// Swallow or Unswallow a window
    ToggleSwallow,
    /// Change the current mapping group
    Submap(SubmapParam<'a>),

    // LAYOUT DISPATCHERS
    // DWINDLE
    /// Toggles the split (top/side) of the current window. `preserve_split` must be enabled for toggling to work
    ToggleSplit,
    /// Swaps the two halves of the split of the current window
    SwapSplit,
    /// One-time override for the split direction (only works on tiled windows)
    PreSelect(Direction),
    /// Moves the selected window (active window if unspecified) to the root of its workspace tree
    MoveToRoot(Option<WindowIdentifier<'a>>, MoveToRootParam),

    // MASTER
    /// Swaps the current window with master.
    /// If the current window is the master,
    /// swaps it with the first child.
    SwapWithMaster(SwapWithMasterParam),
    /// Focuses the master window.
    FocusMaster(FocusMasterParam),
    /// Focuses the next window respecting the layout
    CycleNextMaster(MasterLoopParam),
    /// Focuses the previous window respecting the layout
    CyclePrevMaster(MasterLoopParam),
    /// Swaps the focused window with the next window respecting the layout
    SwapNextMaster(MasterLoopParam),
    /// Swaps the focused window with the previous window respecting the layout
    SwapPrevMaster(MasterLoopParam),
    /// Adds a master to the master side. That will be the active window,
    /// if it’s not a master, or the first non-master window.
    AddMaster,
    /// Removes a master from the master side. That will be the
    /// active window, if it’s a master, or the last master window.
    RemoveMaster,
    /// Sets the orientation for the current workspace to left
    /// (master area left, slave windows to the right, vertically stacked)
    OrientationLeft,
    /// Sets the orientation for the current workspace to right
    /// (master area right, slave windows to the left, vertically stacked)
    OrientationRight,
    /// Sets the orientation for the current workspace to top
    /// (master area top, slave windows to the bottom, horizontally stacked)
    OrientationTop,
    /// Sets the orientation for the current workspace to bottom
    /// (master area bottom, slave windows to the top, horizontally stacked)
    OrientationBottom,
    /// Sets the orientation for the current workspace to center
    /// (master area center, slave windows alternate to the left and right, vertically stacked)
    OrientationCenter,
    /// Cycle to the next orientation for the current workspace (clockwise)
    OrientationNext,
    /// Cycle to the previous orientation for the current workspace (counter-clockwise)
    OrientationPrev,
    /// Cycle to the next orientation from the provided list, for the current workspace
    OrientationCycle(OrientationParam),
    /// Change mfact, the master split ratio
    Mfact(FloatValue),
    /// Rotate the next window in stack to be the master, while keeping the focus on master
    RollNext,
    /// Rotate the previous window in stack to be the master, while keeping the focus on master
    RollPrev,
    /// Sends a raw layout message
    Layout(&'a str),

    // Group Dispatchers
    /// Toggles the current active window into a group
    ToggleGroup,
    /// Switches to the next window in a group.
    ChangeGroupActive(WindowSwitchDirection),
    /// Locks the groups
    LockGroups(LockType),
    /// Locks the currently focused group
    LockActiveGroup(LockType),
    /// Moves the active window into a group in a specified direction
    MoveIntoGroup(Direction),
    /// Moves the active window into or out of a group in a specified direction
    MoveWindowOrGroup(Direction),
    /// Moves the active window out of a group.
    MoveOutOfGroup,
    /// Swaps the active window with the next or previous in a group
    MoveGroupWindow(WindowSwitchDirection),
    /// Prohibit the active window from becoming or being inserted into group
    DenyWindowFromGroup(BinaryState),
    /// Temporarily enable or disable ignore_group_lock
    SetIgnoreGroupLock(BinaryState),
}

/// Enum used for options with a binary on/off state
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Display, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum BinaryState {
    #[display("on")]
    On,
    #[display("off")]
    Off,
    #[display("toggle")]
    Toggle,
}

/// Enum used with [DispatchType::LockGroups], to determine how to lock/unlock
#[derive(Debug, Clone, Copy, Display, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum LockType {
    /// Lock Group
    #[display("lock")]
    Lock,
    /// Unlock Group
    #[display("unlock")]
    Unlock,
    /// Toggle lock state of Group
    #[display("toggle")]
    ToggleLock,
}

/// Param for [DispatchType::SwapWithMaster] dispatcher
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[non_exhaustive]
pub enum SwapWithMasterParam {
    /// New focus is the new master window
    #[display("master")]
    Master,
    /// New focus is the new child
    #[display("child")]
    Child,
    /// Keep the focus of the previously focused window
    #[display("auto")]
    Auto,
    /// Noop if master is already focused
    #[display("ignoremaster")]
    IgnoreMaster,
}

/// Param for [DispatchType::FocusMaster] dispatcher
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[non_exhaustive]
pub enum FocusMasterParam {
    /// Focus stays at master, (even if it was selected before)
    #[display("master")]
    Master,
    /// If the current window is the master, focuses the first child
    #[display("auto")]
    Auto,
    /// If the current window is the master, focuses the previously focused one
    #[display("previous")]
    Previous,
}

/// Param for some master layout dispatchers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[non_exhaustive]
pub enum MasterLoopParam {
    /// Allow looping through the pile
    #[display("loop")]
    Loop,
    /// Do not allow looping through the pile
    #[display("noloop")]
    NoLoop,
}

/// Param for [DispatchType::OrientationCycle] dispatcher
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[non_exhaustive]
pub enum OrientationParam {
    /// Set orientation to left
    #[display("left")]
    Left,
    /// Set orientation to right
    #[display("right")]
    Right,
    /// Set orientation to bottom
    #[display("bottom")]
    Bottom,
    /// Set orientation to top
    #[display("top")]
    Top,
    /// Set orientation to center
    #[display("center")]
    Center,
}

/// Param for split ratio changes
#[derive(Debug, Clone, Copy, PartialEq, Display)]
#[non_exhaustive]
pub enum FloatValue {
    /// Change relative to current factor
    #[display("{}", _0)]
    Relative(f32),
    /// Set factor to exact value between 0 and 1
    #[display("exact {}", _0)]
    Exact(f32),
}

/// Renders a dispatcher into the exact command string.
///
/// With `dispatch = true` this is the socket form, `dispatch <name> <args>`,
/// wrapped in the JSON command flag — what `Dispatch::call` sends. With
/// `dispatch = false` it is the bind form, `<name>,<args>`, with a comma only
/// after the dispatcher name, wrapped in the empty command flag — what the
/// `bind!` macro writes into the config, because Hyprland reads a bind line as
/// `CVarList(value, 4)` (MODS,KEY,DISPATCHER,ARGS).
pub fn gen_dispatch_str(cmd: DispatchType, dispatch: bool) -> crate::Result<CommandContent> {
    use DispatchType::*;
    let sep = if dispatch { " " } else { "," };
    let string_to_pass = match &cmd {
        Custom(name, args) => format!("{name}{sep}{args}"),
        Lua(_) => {
            return Err(crate::HyprError::Unsupported(
                "raw Lua dispatcher on a legacy session".to_string(),
            ));
        }
        Exec(sh) => format!("exec{sep}{sh}"),
        ExecRaw(sh) => format!("execr{sep}{sh}"),
        Pass(win) => format!("pass{sep}{win}"),
        Global(name) => format!("global{sep}{name}"),
        Event(name) => format!("event{sep}{name}"),
        ForceIdle(duration) => format!("forceidle{sep}{duration}"),
        ReleaseInputCapture => "releaseinputcapture".to_string(),
        NoOp => {
            return Err(crate::HyprError::Unsupported(
                "no_op is only available on a Lua session".to_string(),
            ));
        }
        KillActiveWindow => "killactive".to_string(),
        ForceKillActiveWindow => "forcekillactive".to_string(),
        CloseWindow(win) => format!("closewindow{sep}{win}"),
        Workspace(work) => format!("workspace{sep}{work}"),
        MoveToWorkspace(work, Some(win)) => format!("movetoworkspace{sep}{work},{win}"),
        MoveToWorkspace(work, None) => format!("movetoworkspace{sep}{work}"),
        MoveToWorkspaceSilent(work, Some(win)) => format!("movetoworkspacesilent{sep}{work},{win}"),
        MoveToWorkspaceSilent(work, None) => format!("movetoworkspacesilent{sep}{work}"),
        ToggleFloating(Some(v)) => format!("togglefloating{sep}{v}"),
        ToggleFloating(None) => "togglefloating".to_string(),
        SetFloating(Some(v)) => format!("setfloating{sep}{v}"),
        SetFloating(None) => "setfloating".to_string(),
        SetTiled(Some(v)) => format!("settiled{sep}{v}"),
        SetTiled(None) => "settiled".to_string(),
        ToggleFullscreen(ftype) => format!("fullscreen{sep}{ftype}"),
        ToggleFullscreenState(int, cl) => format!("fullscreenstate{sep}{int} {cl}"),
        ToggleFakeFullscreen => {
            return Err(crate::HyprError::Unsupported(
                "fakefullscreen was removed from Hyprland".to_string(),
            ));
        }
        ToggleDPMS(stat, mon) => {
            format!(
                "dpms{sep}{} {}",
                if *stat { "on" } else { "off" },
                mon.unwrap_or_default()
            )
        }
        TogglePseudo => "pseudo".to_string(),
        TogglePin => "pin".to_string(),
        TogglePinWindow(win) => format!("pin{sep}{win}"),
        SendShortcut(mods, key, win_opt) => {
            let mods_str: String = mods.iter().map(|m| m.to_string()).collect();
            match win_opt {
                Some(win) => format!("sendshortcut{sep}{mods_str},{key},{win}"),
                None => format!("sendshortcut{sep}{mods_str},{key},"),
            }
        }
        SendKeyState(mods, key, state, win_opt) => {
            let mods_str: String = mods.iter().map(|m| m.to_string()).collect();
            let win = win_opt
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default();
            format!("sendkeystate{sep}{mods_str},{key},{state},{win}")
        }
        MoveFocus(dir) => format!("movefocus{sep}{dir}",),
        MoveWindow(ident) => format!(
            "movewindow{sep}{}",
            match ident {
                WindowMove::Direction(dir) => dir.to_string(),
                WindowMove::Monitor(mon) => format!("mon:{mon}"),
            }
        ),
        CenterWindow => "centerwindow".to_string(),
        WindowDrag => "movewindow".to_string(),
        WindowResize => "resizewindow".to_string(),
        ResizeWindowKeepAspectRatio(force) => {
            format!("resizewindow{sep}{}", if *force { 1 } else { 2 })
        }
        ResizeActive(pos) => format!("resizeactive{sep}{pos}"),
        MoveActive(pos) => format!("moveactive{sep}{pos}"),
        ResizeWindowPixel(pos, win) => format!("resizewindowpixel{sep}{pos},{win}"),
        MoveWindowPixel(pos, win) => format!("movewindowpixel{sep}{pos},{win}"),
        CycleWindow(dir) => format!("cyclenext{sep}{dir}"),
        CycleWindowFiltered(dir, tiled, floating) => {
            if *tiled == Some(false) || *floating == Some(false) {
                return Err(crate::HyprError::Unsupported(
                    "negative tiled/floating filters have no legacy cyclenext form".to_string(),
                ));
            }
            let mut args = Vec::new();
            if matches!(dir, CycleDirection::Previous) {
                args.push("prev");
            }
            if *tiled == Some(true) {
                args.push("tiled");
            }
            if *floating == Some(true) {
                args.push("floating");
            }
            format!("cyclenext{sep}{}", args.join(" "))
        }
        SwapNext(dir) => format!("swapnext{sep}{dir}"),
        SwapWindow(dir) => format!("swapwindow{sep}{dir}"),
        SwapWindowWith(win) => format!("swapwindow{sep}{win}"),
        TagWindow(act, tag, Some(win)) => format!("tagwindow{sep}{act}{tag} {win}"),
        TagWindow(act, tag, None) => format!("tagwindow{sep}{act}{tag}"),
        ClearTags(_) => {
            return Err(crate::HyprError::Unsupported(
                "clear_tags is only available on a Lua session".to_string(),
            ));
        }
        SetProp(prop, value, win) => {
            let win = win
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "activewindow".to_string());
            format!("setprop{sep}{win} {prop} {value}")
        }
        FocusWindow(win) => format!("focuswindow{sep}{win}"),
        FocusMonitor(mon) => format!("focusmonitor{sep}{mon}"),
        ChangeSplitRatio(FloatValue::Relative(fv)) => {
            format!("layoutmsg{sep}splitratio {fv}")
        }
        ChangeSplitRatio(FloatValue::Exact(fv)) => {
            format!("layoutmsg{sep}splitratio {fv} exact")
        }
        ToggleOpaque => format!("setprop{sep}activewindow opaque toggle"),
        MoveCursorToCorner(corner) => format!("movecursortocorner{sep}{}", corner.clone() as u8),
        MoveCursor(x, y) => format!("movecursor{sep}{x} {y}"),
        WorkspaceOption(opt) => format!("workspaceopt{sep}{opt}"),
        Exit => "exit".to_string(),
        ForceRendererReload => "forcerendererreload".to_string(),
        MoveCurrentWorkspaceToMonitor(mon) => format!("movecurrentworkspacetomonitor{sep}{mon}"),
        MoveWorkspaceToMonitor(work, mon) => format!("moveworkspacetomonitor{sep}{work} {mon}"),
        ToggleSpecialWorkspace(Some(name)) => format!("togglespecialworkspace{sep}{name}"),
        ToggleSpecialWorkspace(None) => "togglespecialworkspace".to_string(),
        RenameWorkspace(id, name) => {
            format!(
                "renameworkspace{sep}{id} {}",
                name.unwrap_or(&id.to_string())
            )
        }
        ChangeWorkspaceId(_, _) => {
            return Err(crate::HyprError::Unsupported(
                "change_id is only available on a Lua session".to_string(),
            ));
        }
        SwapActiveWorkspaces(mon, mon2) => format!("swapactiveworkspaces{sep}{mon} {mon2}",),
        BringActiveToTop => "bringactivetotop".to_string(),
        AlterZOrder(z, Some(win)) => format!("alterzorder{sep}{z},{win}"),
        AlterZOrder(z, None) => format!("alterzorder{sep}{z}"),
        SetCursor(theme, size) => format!("{theme} {}", *size),
        Signal(sig) => format!("signal{sep}{sig}"),
        SignalWindow(win, sig) => format!("signalwindow{sep}{win},{sig}"),
        FocusUrgentOrLast => "focusurgentorlast".to_string(),
        FocusCurrentOrLast => "focuscurrentorlast".to_string(),
        ToggleSwallow => "toggleswallow".to_string(),
        ToggleSplit => format!("layoutmsg{sep}togglesplit"),
        SwapSplit => format!("layoutmsg{sep}swapsplit"),
        PreSelect(dir) => format!("layoutmsg{sep}preselect {dir}"),
        MoveToRoot(Some(win), param) => format!("layoutmsg{sep}movetoroot {win} {param}"),
        MoveToRoot(None, _) => format!("layoutmsg{sep}movetoroot"),
        Submap(param) => format!("submap{sep}{param}"),
        SwapWithMaster(param) => format!("layoutmsg{sep}swapwithmaster {param}"),
        FocusMaster(param) => format!("layoutmsg{sep}focusmaster {param}"),
        CycleNextMaster(param) => format!("layoutmsg{sep}cyclenext {param}"),
        CyclePrevMaster(param) => format!("layoutmsg{sep}cycleprev {param}"),
        SwapNextMaster(param) => format!("layoutmsg{sep}swapnext {param}"),
        SwapPrevMaster(param) => format!("layoutmsg{sep}swapprev {param}"),
        AddMaster => format!("layoutmsg{sep}addmaster"),
        RemoveMaster => format!("layoutmsg{sep}removemaster"),
        OrientationLeft => format!("layoutmsg{sep}orientationleft"),
        OrientationRight => format!("layoutmsg{sep}orientationright"),
        OrientationTop => format!("layoutmsg{sep}orientationtop"),
        OrientationBottom => format!("layoutmsg{sep}orientationbottom"),
        OrientationCenter => format!("layoutmsg{sep}orientationcenter"),
        OrientationNext => format!("layoutmsg{sep}orientationnext"),
        OrientationPrev => format!("layoutmsg{sep}orientationprev"),
        OrientationCycle(param) => format!("layoutmsg{sep}orientationcycle {param}"),
        Mfact(fv) => format!("layoutmsg{sep}mfact {fv}"),
        RollNext => format!("layoutmsg{sep}rollnext"),
        RollPrev => format!("layoutmsg{sep}rollprev"),
        Layout(message) => format!("layoutmsg{sep}{message}"),
        ToggleGroup => "togglegroup".to_string(),
        ChangeGroupActive(dir) => format!("changegroupactive{sep}{dir}"),
        LockGroups(how) => format!("lockgroups{sep}{how}"),
        LockActiveGroup(how) => format!("lockactivegroups{sep}{how}"),
        MoveIntoGroup(dir) => format!("moveintogroup{sep}{dir}"),
        MoveWindowOrGroup(dir) => format!("movewindoworgroup{sep}{dir}"),
        MoveOutOfGroup => "moveoutofgroup".to_string(),
        MoveGroupWindow(dir) => format!("movegroupwindow{sep}{dir}"),
        DenyWindowFromGroup(state) => format!("denywindowfromgroup{sep}{state}"),
        SetIgnoreGroupLock(state) => format!("setignoregrouplock{sep}{state}"),
    };

    if let SetCursor(_, _) = cmd {
        Ok(command!(JSON, "setcursor {string_to_pass}"))
    } else if dispatch {
        Ok(command!(JSON, "dispatch {string_to_pass}"))
    } else {
        Ok(command!(Empty, "{string_to_pass}"))
    }
}

fn lua_direction(dir: &Direction) -> &'static str {
    match dir {
        Direction::Up => "up",
        Direction::Down => "down",
        Direction::Right => "right",
        Direction::Left => "left",
    }
}

fn lua_lock_action(action: &LockType) -> &'static str {
    match action {
        LockType::Lock => "enable",
        LockType::Unlock => "disable",
        LockType::ToggleLock => "toggle",
    }
}

fn lua_binary_state(state: &BinaryState) -> &'static str {
    match state {
        BinaryState::On => "on",
        BinaryState::Off => "off",
        BinaryState::Toggle => "toggle",
    }
}

fn lua_window_call(function: &str, window: Option<&WindowIdentifier<'_>>, args: &str) -> String {
    let mut fields = args.to_string();
    if let Some(window) = window {
        if !fields.is_empty() {
            fields.push_str(", ");
        }
        fields.push_str(&format!(
            "window = {}",
            crate::lua::quote(&window.to_string())
        ));
    }

    if fields.is_empty() {
        format!("hl.dsp.window.{function}()")
    } else {
        format!("hl.dsp.window.{function}({{ {fields} }})")
    }
}

fn lua_position_call(
    function: &str,
    position: &Position,
    window: Option<&WindowIdentifier<'_>>,
) -> crate::Result<String> {
    let window_arg = window
        .map(|window| format!(", window = {}", crate::lua::quote(&window.to_string())))
        .unwrap_or_default();

    match position {
        Position::Delta(x, y) | Position::Exact(x, y) => {
            let relative = matches!(position, Position::Delta(_, _));
            Ok(format!(
                "hl.dsp.window.{function}({{ x = {x}, y = {y}, relative = {relative}{window_arg} }})"
            ))
        }
        Position::DeltaFraction(x, y) => Ok(format!(
            "(function(w) if not w then return hl.dsp.no_op() end return hl.dsp.window.{function}({{ x = w.{field}.x * {x}, y = w.{field}.y * {y}, relative = true{window_arg} }}) end)({target})",
            field = if function == "move" { "at" } else { "size" },
            target = window
                .map(|window| format!("hl.get_window({})", crate::lua::quote(&window.to_string())))
                .unwrap_or_else(|| "hl.get_active_window()".to_string()),
            x = *x as f64 / 100.0,
            y = *y as f64 / 100.0,
        )),
        Position::ExactFraction(x, y) => Ok(format!(
            "(function(m) if not m then return hl.dsp.no_op() end return hl.dsp.window.{function}({{ x = m.width * {x}, y = m.height * {y}{window_arg} }}) end)(hl.get_active_monitor())",
            x = *x as f64 / 100.0,
            y = *y as f64 / 100.0,
        )),
    }
}

fn lua_layout_expr(cmd: &DispatchType<'_>) -> crate::Result<String> {
    let legacy = gen_dispatch_str(cmd.clone(), true)?;
    let message = legacy
        .data
        .strip_prefix("dispatch layoutmsg ")
        .ok_or_else(|| {
            crate::HyprError::Internal(
                "layout dispatcher did not render as a layout message".to_string(),
            )
        })?;
    Ok(format!("hl.dsp.layout({})", crate::lua::quote(message)))
}

/// Render a dispatcher as a Lua expression.
///
/// The result is the dispatcher object passed to `hl.dispatch`, e.g.
/// `hl.dsp.window.close()`. `SetCursor` is an IPC command rather than a dispatcher and is
/// rejected here; use [`gen_lua_dispatch_str`] for the socket form.
pub fn gen_lua_dispatch_expr(cmd: &DispatchType<'_>) -> crate::Result<String> {
    use DispatchType::*;

    let expr = match cmd {
        Lua(raw) => raw.to_string(),
        Custom(name, _) => {
            return Err(crate::HyprError::Unsupported(format!(
                "legacy custom dispatcher `{name}` on a Lua session; use DispatchType::Lua"
            )));
        }
        SetCursor(_, _) => {
            return Err(crate::HyprError::Unsupported(
                "setcursor is an IPC command, not a Lua dispatcher".to_string(),
            ));
        }
        Exec(sh) => format!("hl.dsp.exec_cmd({})", crate::lua::quote(sh)),
        ExecRaw(sh) => format!("hl.dsp.exec_raw({})", crate::lua::quote(sh)),
        Pass(win) => format!(
            "hl.dsp.pass({{ window = {} }})",
            crate::lua::quote(&win.to_string())
        ),
        Global(name) => format!("hl.dsp.global({})", crate::lua::quote(name)),
        Event(name) => format!("hl.dsp.event({})", crate::lua::quote(name)),
        ForceIdle(duration) => format!("hl.dsp.force_idle({duration})"),
        ReleaseInputCapture => "hl.dsp.release_input_capture()".to_string(),
        NoOp => "hl.dsp.no_op()".to_string(),
        KillActiveWindow => "hl.dsp.window.close()".to_string(),
        ForceKillActiveWindow => "hl.dsp.window.kill()".to_string(),
        CloseWindow(win) => lua_window_call("close", Some(win), ""),
        Workspace(workspace) => format!(
            "hl.dsp.focus({{ workspace = {} }})",
            crate::lua::quote(&workspace.to_string())
        ),
        MoveToWorkspace(workspace, win) => lua_window_call(
            "move",
            win.as_ref(),
            &format!(
                "workspace = {}, follow = true",
                crate::lua::quote(&workspace.to_string())
            ),
        ),
        MoveToWorkspaceSilent(workspace, win) => lua_window_call(
            "move",
            win.as_ref(),
            &format!(
                "workspace = {}, follow = false",
                crate::lua::quote(&workspace.to_string())
            ),
        ),
        ToggleFloating(win) => lua_window_call("float", win.as_ref(), "action = \"toggle\""),
        SetFloating(win) => lua_window_call("float", win.as_ref(), "action = \"enable\""),
        SetTiled(win) => lua_window_call("float", win.as_ref(), "action = \"disable\""),
        ToggleFullscreen(mode) => match mode {
            FullscreenType::Real => {
                "hl.dsp.window.fullscreen({ mode = \"fullscreen\", action = \"set\" })".to_string()
            }
            FullscreenType::Maximize => {
                "hl.dsp.window.fullscreen({ mode = \"maximized\", action = \"set\" })".to_string()
            }
            FullscreenType::NoParam => "hl.dsp.window.fullscreen()".to_string(),
        },
        ToggleFullscreenState(internal, client) => format!(
            "hl.dsp.window.fullscreen_state({{ internal = {}, client = {} }})",
            *internal as i8, *client as i8
        ),
        ToggleFakeFullscreen => {
            return Err(crate::HyprError::Unsupported(
                "fakefullscreen has no Lua dispatcher".to_string(),
            ));
        }
        ToggleDPMS(on, monitor) => {
            let action = if *on { "enable" } else { "disable" };
            let monitor = monitor
                .map(|monitor| format!(", monitor = {}", crate::lua::quote(monitor)))
                .unwrap_or_default();
            format!("hl.dsp.dpms({{ action = \"{action}\"{monitor} }})")
        }
        TogglePseudo => "hl.dsp.window.pseudo()".to_string(),
        TogglePin => "hl.dsp.window.pin()".to_string(),
        TogglePinWindow(win) => lua_window_call("pin", Some(win), ""),
        SendShortcut(mods, key, win) => {
            let mods = mods
                .iter()
                .filter(|modifier| **modifier != Mod::NONE)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ");
            let mut args = format!(
                "mods = {}, key = {}",
                crate::lua::quote(&mods),
                crate::lua::quote(key)
            );
            if let Some(win) = win {
                args.push_str(&format!(
                    ", window = {}",
                    crate::lua::quote(&win.to_string())
                ));
            }
            format!("hl.dsp.send_shortcut({{ {args} }})")
        }
        SendKeyState(mods, key, state, win) => {
            let mods = mods
                .iter()
                .filter(|modifier| **modifier != Mod::NONE)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ");
            let mut args = format!(
                "mods = {}, key = {}, state = {}",
                crate::lua::quote(&mods),
                crate::lua::quote(key),
                crate::lua::quote(&state.to_string())
            );
            if let Some(win) = win {
                args.push_str(&format!(
                    ", window = {}",
                    crate::lua::quote(&win.to_string())
                ));
            }
            format!("hl.dsp.send_key_state({{ {args} }})")
        }
        Signal(signal) => format!("hl.dsp.window.signal({{ signal = {} }})", *signal as i32),
        SignalWindow(win, signal) => format!(
            "hl.dsp.window.signal({{ signal = {}, window = {} }})",
            *signal as i32,
            crate::lua::quote(&win.to_string())
        ),
        MoveFocus(direction) => format!(
            "hl.dsp.focus({{ direction = {} }})",
            crate::lua::quote(lua_direction(direction))
        ),
        MoveWindow(movement) => match movement {
            WindowMove::Direction(direction) => lua_window_call(
                "move",
                None,
                &format!(
                    "direction = {}",
                    crate::lua::quote(lua_direction(direction))
                ),
            ),
            WindowMove::Monitor(monitor) => lua_window_call(
                "move",
                None,
                &format!("monitor = {}", crate::lua::quote(&monitor.to_string())),
            ),
        },
        CenterWindow => "hl.dsp.window.center()".to_string(),
        WindowDrag => "hl.dsp.window.drag()".to_string(),
        WindowResize => "hl.dsp.window.resize()".to_string(),
        ResizeWindowKeepAspectRatio(force) => {
            format!("hl.dsp.window.resize({{ keep_aspect_ratio = {force} }})")
        }
        ResizeActive(position) => lua_position_call("resize", position, None)?,
        MoveActive(position) => lua_position_call("move", position, None)?,
        ResizeWindowPixel(position, win) => lua_position_call("resize", position, Some(win))?,
        MoveWindowPixel(position, win) => lua_position_call("move", position, Some(win))?,
        CycleWindow(direction) => match direction {
            CycleDirection::Next => "hl.dsp.window.cycle_next({ next = true })".to_string(),
            CycleDirection::Previous => "hl.dsp.window.cycle_next({ next = false })".to_string(),
        },
        SwapNext(direction) => match direction {
            CycleDirection::Next => "hl.dsp.window.swap({ next = true })".to_string(),
            CycleDirection::Previous => "hl.dsp.window.swap({ prev = true })".to_string(),
        },
        CycleWindowFiltered(direction, tiled, floating) => {
            let mut args = format!("next = {}", matches!(direction, CycleDirection::Next));
            if let Some(tiled) = tiled {
                args.push_str(&format!(", tiled = {tiled}"));
            }
            if let Some(floating) = floating {
                args.push_str(&format!(", floating = {floating}"));
            }
            format!("hl.dsp.window.cycle_next({{ {args} }})")
        }
        SwapWindow(direction) => format!(
            "hl.dsp.window.swap({{ direction = {} }})",
            crate::lua::quote(lua_direction(direction))
        ),
        SwapWindowWith(win) => format!(
            "hl.dsp.window.swap({{ target = {} }})",
            crate::lua::quote(&win.to_string())
        ),
        TagWindow(action, tag, win) => lua_window_call(
            "tag",
            win.as_ref(),
            &format!("tag = {}", crate::lua::quote(&format!("{action}{tag}"))),
        ),
        ClearTags(win) => lua_window_call("clear_tags", win.as_ref(), ""),
        SetProp(prop, value, win) => lua_window_call(
            "set_prop",
            win.as_ref(),
            &format!(
                "prop = {}, value = {}",
                crate::lua::quote(prop),
                crate::lua::quote(value)
            ),
        ),
        FocusWindow(win) => format!(
            "hl.dsp.focus({{ window = {} }})",
            crate::lua::quote(&win.to_string())
        ),
        FocusMonitor(monitor) => format!(
            "hl.dsp.focus({{ monitor = {} }})",
            crate::lua::quote(&monitor.to_string())
        ),
        ChangeSplitRatio(_)
        | ToggleSplit
        | SwapSplit
        | PreSelect(_)
        | MoveToRoot(_, _)
        | SwapWithMaster(_)
        | FocusMaster(_)
        | CycleNextMaster(_)
        | CyclePrevMaster(_)
        | SwapNextMaster(_)
        | SwapPrevMaster(_)
        | AddMaster
        | RemoveMaster
        | OrientationLeft
        | OrientationRight
        | OrientationTop
        | OrientationBottom
        | OrientationCenter
        | OrientationNext
        | OrientationPrev
        | OrientationCycle(_)
        | Mfact(_)
        | RollNext
        | RollPrev => lua_layout_expr(cmd)?,
        Layout(message) => format!("hl.dsp.layout({})", crate::lua::quote(message)),
        ToggleOpaque => {
            "hl.dsp.window.set_prop({ prop = \"opaque\", value = \"toggle\" })".to_string()
        }
        MoveCursorToCorner(corner) => format!(
            "hl.dsp.cursor.move_to_corner({{ corner = {} }})",
            corner.clone() as u8
        ),
        MoveCursor(x, y) => format!("hl.dsp.cursor.move({{ x = {x}, y = {y} }})"),
        WorkspaceOption(_) => {
            return Err(crate::HyprError::Unsupported(
                "workspaceopt is deprecated and has no Lua equivalent".to_string(),
            ));
        }
        RenameWorkspace(id, name) => {
            let name = name
                .map(|name| format!(", name = {}", crate::lua::quote(name)))
                .unwrap_or_default();
            format!("hl.dsp.workspace.rename({{ workspace = {id}{name} }})")
        }
        ChangeWorkspaceId(workspace, id) => format!(
            "hl.dsp.workspace.change_id({{ workspace = {}, id = {id} }})",
            crate::lua::quote(&workspace.to_string())
        ),
        Exit => "hl.dsp.exit()".to_string(),
        ForceRendererReload => "hl.dsp.force_renderer_reload()".to_string(),
        MoveCurrentWorkspaceToMonitor(monitor) => format!(
            "hl.dsp.workspace.move({{ monitor = {} }})",
            crate::lua::quote(&monitor.to_string())
        ),
        MoveWorkspaceToMonitor(workspace, monitor) => format!(
            "hl.dsp.workspace.move({{ workspace = {}, monitor = {} }})",
            crate::lua::quote(&workspace.to_string()),
            crate::lua::quote(&monitor.to_string())
        ),
        SwapActiveWorkspaces(first, second) => format!(
            "hl.dsp.workspace.swap_monitors({{ monitor1 = {}, monitor2 = {} }})",
            crate::lua::quote(&first.to_string()),
            crate::lua::quote(&second.to_string())
        ),
        BringActiveToTop => "hl.dsp.window.bring_to_top()".to_string(),
        AlterZOrder(order, win) => lua_window_call(
            "alter_zorder",
            win.as_ref(),
            &format!("mode = {}", crate::lua::quote(&order.to_string())),
        ),
        ToggleSpecialWorkspace(name) => match name {
            Some(name) => format!(
                "hl.dsp.workspace.toggle_special({})",
                crate::lua::quote(name)
            ),
            None => "hl.dsp.workspace.toggle_special()".to_string(),
        },
        FocusUrgentOrLast => "hl.dsp.focus({ urgent_or_last = true })".to_string(),
        FocusCurrentOrLast => "hl.dsp.focus({ last = true })".to_string(),
        ToggleSwallow => "hl.dsp.window.toggle_swallow()".to_string(),
        Submap(param) => format!("hl.dsp.submap({})", crate::lua::quote(&param.to_string())),
        ToggleGroup => "hl.dsp.group.toggle()".to_string(),
        ChangeGroupActive(direction) => match direction {
            WindowSwitchDirection::Forward => "hl.dsp.group.next()".to_string(),
            WindowSwitchDirection::Back => "hl.dsp.group.prev()".to_string(),
            WindowSwitchDirection::Index(index) => {
                format!("hl.dsp.group.active({{ index = {index} }})")
            }
        },
        LockGroups(action) => format!(
            "hl.dsp.group.lock({{ action = {} }})",
            crate::lua::quote(lua_lock_action(action))
        ),
        LockActiveGroup(action) => format!(
            "hl.dsp.group.lock_active({{ action = {} }})",
            crate::lua::quote(lua_lock_action(action))
        ),
        MoveIntoGroup(direction) => lua_window_call(
            "move",
            None,
            &format!(
                "into_group = {}",
                crate::lua::quote(lua_direction(direction))
            ),
        ),
        MoveWindowOrGroup(direction) => lua_window_call(
            "move",
            None,
            &format!(
                "direction = {}, group_aware = true",
                crate::lua::quote(lua_direction(direction))
            ),
        ),
        MoveOutOfGroup => lua_window_call("move", None, "out_of_group = true"),
        MoveGroupWindow(direction) => match direction {
            WindowSwitchDirection::Forward => {
                "hl.dsp.group.move_window({ forward = true })".to_string()
            }
            WindowSwitchDirection::Back => {
                "hl.dsp.group.move_window({ forward = false })".to_string()
            }
            _ => {
                return Err(crate::HyprError::Unsupported(
                    "group move_window does not accept an index on a Lua session".to_string(),
                ));
            }
        },
        DenyWindowFromGroup(state) => format!(
            "hl.dsp.window.deny_from_group({{ action = {} }})",
            crate::lua::quote(lua_binary_state(state))
        ),
        SetIgnoreGroupLock(_) => "hl.dsp.no_op()".to_string(),
    };

    Ok(expr)
}

/// Render the socket command for a dispatcher on a Lua session.
pub fn gen_lua_dispatch_str(cmd: DispatchType<'_>) -> crate::Result<CommandContent> {
    if let DispatchType::SetCursor(theme, size) = cmd {
        return Ok(command!(JSON, "setcursor {theme} {size}"));
    }

    Ok(command!(JSON, "dispatch {}", gen_lua_dispatch_expr(&cmd)?))
}

/// The struct that provides all dispatching methods
pub struct Dispatch;

impl Dispatch {
    /// This function calls a specified dispatcher (blocking)
    ///
    /// ```rust,no_run
    /// # use hyprland::Result;
    /// # fn main() -> Result<()> {
    /// use hyprland::dispatch::{DispatchType,Dispatch};
    /// // This is an example of just one dispatcher, there are many more!
    /// Dispatch::call(DispatchType::Exec("kitty"))
    /// # }
    /// ```
    pub fn call(dispatch_type: DispatchType) -> crate::Result<()> {
        Self::instance_call(default_instance()?, dispatch_type)
    }

    /// This function calls a specified dispatcher (blocking)
    ///
    /// ```rust,no_run
    /// # use hyprland::Result;
    /// # fn main() -> Result<()> {
    /// use hyprland::dispatch::{DispatchType,Dispatch};
    /// let instance = hyprland::instance::Instance::from_current_env()?;
    /// // This is an example of just one dispatcher, there are many more!
    /// Dispatch::instance_call(&instance, DispatchType::Exec("kitty"))
    /// # }
    /// ```
    pub fn instance_call(
        instance: &crate::instance::Instance,
        dispatch_type: DispatchType,
    ) -> crate::Result<()> {
        let command = if instance.uses_lua_config() {
            gen_lua_dispatch_str(dispatch_type)?
        } else {
            gen_dispatch_str(dispatch_type, true)?
        };
        let output = instance.write_to_socket(command);
        match output {
            Ok(msg) => match msg.as_str() {
                "ok" => Ok(()),
                msg => Err(HyprError::NotOkDispatch(msg.to_string())),
            },
            Err(error) => Err(error),
        }
    }

    /// This function calls a specified dispatcher (async)
    ///
    /// ```rust,no_run
    /// # use hyprland::Result;
    /// #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> Result<()> {
    /// use hyprland::dispatch::{Dispatch,DispatchType};
    /// let instance = hyprland::instance::Instance::from_current_env()?;
    /// // This is an example of just one dispatcher, there are many more!
    /// Dispatch::call_async(DispatchType::Exec("kitty")).await
    /// # }
    /// ```
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn call_async(dispatch_type: DispatchType<'_>) -> crate::Result<()> {
        Self::instance_call_async(default_instance()?, dispatch_type).await
    }

    /// This function calls a specified dispatcher (async)
    ///
    /// ```rust,no_run
    /// # use hyprland::Result;
    /// #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> Result<()> {
    /// use hyprland::dispatch::{Dispatch,DispatchType};
    /// let instance = hyprland::instance::Instance::from_current_env()?;
    /// // This is an example of just one dispatcher, there are many more!
    /// Dispatch::call_async(DispatchType::Exec("kitty")).await
    /// # }
    /// ```
    #[cfg(any(feature = "async-lite", feature = "tokio"))]
    pub async fn instance_call_async(
        instance: &crate::instance::Instance,
        dispatch_type: DispatchType<'_>,
    ) -> crate::Result<()> {
        let command = if instance.uses_lua_config_async().await {
            gen_lua_dispatch_str(dispatch_type)?
        } else {
            gen_dispatch_str(dispatch_type, true)?
        };
        let output = instance.write_to_socket_async(command).await;
        match output {
            Ok(msg) => match msg.as_str() {
                "ok" => Ok(()),
                msg => Err(HyprError::NotOkDispatch(msg.to_string())),
            },
            Err(error) => Err(error),
        }
    }
}

/// Macro abstraction over [Dispatch::call]
#[macro_export]
macro_rules! dispatch {
    (async; $dis:ident) => {
        $crate::dispatch::Dispatch::call_async($crate::dispatch::DispatchType::$dis)
    };
    (async; $dis:ident, $( $arg:expr ), *) => {
        $crate::dispatch::Dispatch::call_async($crate::dispatch::DispatchType::$dis($($arg), *))
    };
    (async; $instance:expr; $dis:ident) => {
        $crate::dispatch::Dispatch::instance_call_async($instance, $crate::dispatch::DispatchType::$dis)
    };
    (async; $instance:expr; $dis:ident, $( $arg:expr ), *) => {
        $crate::dispatch::Dispatch::instance_call_async($instance, $crate::dispatch::DispatchType::$dis($($arg), *))
    };
    ($dis:ident) => {
        $crate::dispatch::Dispatch::call($crate::dispatch::DispatchType::$dis)
    };
    ($dis:ident, $( $arg:expr ), *) => {
        $crate::dispatch::Dispatch::call($crate::dispatch::DispatchType::$dis($($arg), *))
    };
    ($instance:expr; $dis:ident) => {
        $crate::dispatch::Dispatch::instance_call($instance, $crate::dispatch::DispatchType::$dis)
    };
    ($instance:expr; $dis:ident, $( $arg:expr ), *) => {
        $crate::dispatch::Dispatch::instance_call($instance, $crate::dispatch::DispatchType::$dis($($arg), *))
    };
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::{
        DispatchType, FirstEmpty, FloatValue, Position, WindowIdentifier,
        WorkspaceIdentifierWithSpecial, WorkspaceOptions, gen_dispatch_str,
    };

    /// Hyprland reads a bind line as `CVarList(value, 4)` — MODS,KEY,DISPATCHER,ARGS
    /// (`ConfigManager.cpp`, `handleBind`) — so only the dispatcher name must be
    /// followed by `,`; the args keep their normal space-separated syntax.
    /// On the socket the separator is a space. Assert both.
    #[test]
    fn separators_follow_the_target_syntax() {
        let cases = [
            (
                DispatchType::MoveActive(Position::Delta(0, 50)),
                "dispatch moveactive 0 50",
                "moveactive,0 50",
            ),
            (
                DispatchType::ChangeSplitRatio(FloatValue::Relative(-0.05)),
                "dispatch layoutmsg splitratio -0.05",
                "layoutmsg,splitratio -0.05",
            ),
            (
                DispatchType::ToggleSpecialWorkspace(Some("scratch".to_owned())),
                "dispatch togglespecialworkspace scratch",
                "togglespecialworkspace,scratch",
            ),
        ];

        for (dispatcher, socket, bind) in cases {
            assert_eq!(
                gen_dispatch_str(dispatcher.clone(), true).unwrap().data,
                socket
            );
            assert_eq!(gen_dispatch_str(dispatcher, false).unwrap().data, bind);
        }
    }

    /// `RelativeMonitor(0)` used to collapse to a bare `+0`, dropping the `m`.
    #[test]
    fn relative_identifiers_keep_their_prefix_at_zero() {
        // `RelativeMonitor` is the workspace-side variant and keeps its `m`.
        assert_eq!(
            WorkspaceIdentifierWithSpecial::RelativeMonitor(0).to_string(),
            "m+0",
            "a bare +0 makes Hyprland read it as a relative workspace"
        );
        assert_eq!(
            WorkspaceIdentifierWithSpecial::RelativeMonitor(-2).to_string(),
            "m-2"
        );
        assert_eq!(
            WorkspaceIdentifierWithSpecial::RelativeMonitor(3).to_string(),
            "m+3"
        );
        assert_eq!(
            WorkspaceIdentifierWithSpecial::RelativeOpen(0).to_string(),
            "e+0"
        );
        assert_eq!(
            WorkspaceIdentifierWithSpecial::RelativeMonitorIncludingEmpty(0).to_string(),
            "r+0"
        );
        assert_eq!(
            WorkspaceIdentifierWithSpecial::Relative(0).to_string(),
            "+0"
        );
    }

    /// All four `FirstEmpty` combinations are accepted by Hyprland (`workspace
    /// empty`, `empty m`, `empty n`, `empty mn` all return ok on 0.56.2).
    #[test]
    fn first_empty_combinations_are_all_valid() {
        for (on_monitor, next, expected) in [
            (false, false, ""),
            (true, false, "m"),
            (false, true, "n"),
            (true, true, "mn"),
        ] {
            let fe = FirstEmpty { on_monitor, next };
            assert_eq!(fe.to_string(), expected);
            assert_eq!(
                WorkspaceIdentifierWithSpecial::Empty(fe).to_string(),
                format!("empty{expected}")
            );
        }
    }

    #[test]
    fn toggle_special_without_a_name_has_no_arguments() {
        assert_eq!(
            gen_dispatch_str(DispatchType::ToggleSpecialWorkspace(None), true)
                .unwrap()
                .data,
            "dispatch togglespecialworkspace"
        );
    }

    #[test]
    fn position_renders_both_axes() {
        assert_eq!(Position::Delta(10, 20).to_string(), "10 20");
        assert_eq!(Position::Delta(-10, 20).to_string(), "-10 20");
        assert_eq!(Position::Delta(0, -50).to_string(), "0 -50");
        assert_eq!(Position::Exact(300, 400).to_string(), "exact 300 400");
        assert_eq!(Position::DeltaFraction(10, 20).to_string(), "10% 20%");
        assert_eq!(Position::DeltaFraction(-5, 25).to_string(), "-5% 25%");
        assert_eq!(Position::ExactFraction(10, 90).to_string(), "exact 10% 90%");
    }

    #[test]
    fn move_and_resize_dispatchers_carry_both_axes() {
        let cases = [
            (
                DispatchType::MoveActive(Position::Delta(0, 50)),
                "dispatch moveactive 0 50",
            ),
            (
                DispatchType::MoveActive(Position::Delta(30, -30)),
                "dispatch moveactive 30 -30",
            ),
            (
                DispatchType::MoveActive(Position::Exact(300, 400)),
                "dispatch moveactive exact 300 400",
            ),
            (
                DispatchType::MoveActive(Position::DeltaFraction(0, 50)),
                "dispatch moveactive 0% 50%",
            ),
            (
                DispatchType::MoveActive(Position::ExactFraction(25, 75)),
                "dispatch moveactive exact 25% 75%",
            ),
            (
                DispatchType::ResizeActive(Position::Delta(0, 50)),
                "dispatch resizeactive 0 50",
            ),
            (
                DispatchType::ResizeActive(Position::Exact(0, 400)),
                "dispatch resizeactive exact 0 400",
            ),
            (
                DispatchType::MoveWindowPixel(
                    Position::Delta(5, -5),
                    WindowIdentifier::ActiveWindow,
                ),
                "dispatch movewindowpixel 5 -5,activewindow",
            ),
            (
                DispatchType::ResizeWindowPixel(
                    Position::Exact(10, 20),
                    WindowIdentifier::ActiveWindow,
                ),
                "dispatch resizewindowpixel exact 10 20,activewindow",
            ),
        ];

        for (dispatcher, expected) in cases {
            let label = format!("{dispatcher:?}");
            let Ok(generated) = gen_dispatch_str(dispatcher, true) else {
                panic!("failed to generate a dispatch string for {label}");
            };
            assert_eq!(generated.data, expected);
        }
    }

    #[test]
    fn workspace_options_match_their_variant() {
        assert_eq!(WorkspaceOptions::AllPseudo.to_string(), "allpseudo");
        assert_eq!(WorkspaceOptions::AllFloat.to_string(), "allfloat");
    }

    #[test]
    fn workspace_option_dispatcher_uses_the_matching_label() {
        let cases = [
            (
                DispatchType::WorkspaceOption(WorkspaceOptions::AllPseudo),
                "dispatch workspaceopt allpseudo",
            ),
            (
                DispatchType::WorkspaceOption(WorkspaceOptions::AllFloat),
                "dispatch workspaceopt allfloat",
            ),
        ];

        for (dispatcher, expected) in cases {
            let label = format!("{dispatcher:?}");
            let Ok(generated) = gen_dispatch_str(dispatcher, true) else {
                panic!("failed to generate a dispatch string for {label}");
            };
            assert_eq!(generated.data, expected);
        }
    }
}
