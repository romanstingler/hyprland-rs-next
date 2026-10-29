# Migration Guide

## `0.6.0` ⟶ `0.7.0`

`0.7.0` adds Hyprland **0.57.0** support. Hyprland now identifies workspaces by an
`addressableName` (`"3"`, `"special:magic"`, or a named workspace) instead of always sending
a numeric id. The crate uses one identity type for data and events.

1. **Workspace events use `WorkspaceBasic`.**
   - `Event::WorkspaceChanged` / `WorkspaceAdded` / `WorkspaceDeleted` now carry
     `WorkspaceBasic` directly (`WorkspaceEventData` is a type alias).
   - `WorkspaceBasic::id` became the fallible accessor `id()`. Special and named workspaces
     may have no numeric id; identify them with `address` and `is_special()`.
   - `WorkspaceBasic::address` is the `addressableName`; `r#type` is `Normal`/`Special`.
2. **Nested workspace fields.** `WorkspaceMovedEventData`, `WindowMoveEvent`,
   `WindowOpenEvent`, `MonitorEventData`, `MonitorV2EventData`, `ChangedSpecialEventData`
   and `ChangedSpecialV2EventData` now expose `workspace: WorkspaceBasic` instead of
   `workspace_id` / `workspace_name` pairs.
3. **Empty special workspace.** The 0.57 `specialWorkspace: { address = "", type = "", name = "" }`
   sentinel deserialises as `address = None`, `type = None`; `name` remains `""`.
4. **New dispatchers.** `DispatchType::ReloadConfig` renders `hl.dsp.reload_config()` on 0.57
   Lua sessions (legacy returns `Unsupported`). `LockGroups` now renders
   `group.lock({ action, all = true })`; `LockActiveGroup` still uses the deprecated
   `lock_active` form so it keeps working on 0.56.x.
5. **Events accept both shapes.** Parsers detect a numeric first field (0.56) versus an
   addressable name (0.57), so 0.56.x remains supported.

## `0.5.x` ⟶ `0.6.0`

`0.6.0` adds **Lua-config support** for Hyprland 0.56.x while keeping the legacy `.conf`
path. The config provider is detected at runtime from `hyprctl status`, not from the
Hyprland version.

1. **`Dispatch`, `Keyword` and `Binder` now select the grammar automatically.** No call-site
   change is needed. `Instance::capabilities()` / `uses_lua_config()` expose the detected
   provider.
2. **New `DispatchType` variants.** `Lua(&str)` is the raw escape hatch for argument
   combinations the typed variants do not model. Typed additions include `Event`,
   `ForceIdle`, `ReleaseInputCapture`, `NoOp`, `Layout`, `WindowDrag`, `WindowResize`,
   `ResizeWindowKeepAspectRatio`, `ClearTags`, `SetProp`, `ChangeWorkspaceId`,
   `SwapWindowWith`, `SendKeyState`, and `CycleWindowFiltered`.
3. **New `Flag` variants.** `AutoConsuming`, `LongPress`, `Click`, `Drag`, `SubMapUniversal`,
   and `AllowInputCapture`.
4. **`ToggleFakeFullscreen` is now an error** on both providers: Hyprland removed
   `fakefullscreen` before 0.54.
5. **`Keyword::set` under Lua** rejects legacy top-level directives (`monitor`, `windowrule`,
   `env`, ...) because they are Lua config functions, not `hl.config` values.
6. **Lua-only dispatchers** (`NoOp`, `ClearTags`, `ChangeWorkspaceId`) return
   `HyprError::Unsupported` on a `.conf` session. `Custom` has no Lua form; use
   `DispatchType::Lua`.
7. `MultiKey` and `HasDescription` bind flags still return `Unsupported` under Lua.

## `0.4.x` ⟶ `0.5.0`

`0.5.0` is the last release built around the **legacy (`.conf`) Hyprland config path**
(Hyprland 0.54–0.56.x). It prepares the API for the Lua-config work that follows, so it is
deliberately breaking. The compiler will point at most of it; the non-compiler breakage is listed
first.

1. **Match arms must be exhaustive-safe.** Every public enum is now `#[non_exhaustive]`. Add a
   `_ => ...` arm to every `match` over a crate enum. Future variants will then be additive.
2. **`Workspace` / `WorkspaceBasic`.** `id` is no longer a public field:
   - `workspace.id` → `workspace.id()?` (returns `crate::Result<WorkspaceId>`).
   - On post-0.56 payloads a *special* workspace has no numeric id; the accessor errors instead of
     returning a plausible `0`. Use `address` / `type` for identity.
   - New public fields: `address: Option<Address>` and `r#type: Option<WorkspaceKind>`
     (`WorkspaceKind` is a new `#[non_exhaustive]` enum: `Normal`, `Special`).
3. **`Flag` variant names.** `Flag::l` → `Flag::Locked`, `r` → `Release`, `e` → `Repeat`,
   `n` → `NonConsuming`, `m` → `Mouse`, `t` → `Transparent`, `i` → `IgnoreMods`,
   `s` → `MultiKey`, `d` → `HasDescription`, `p` → `DontInhibit`. The emitted bind letters are
   unchanged, so generated bind lines do not move.
4. **`HyprError`.** `InvalidHyprGradiantFormat` → `InvalidHyprGradientFormat`; new
   `Unsupported(String)` variant. `Error::source()` now returns the wrapped `serde_json`, I/O,
   UTF-8 (and `hyprpaper`) error instead of `None`.
5. **`GroupToggledEventData`.** `window_addresses: Vec<Address>` → `window_address: Address`;
   upstream sends exactly one owner address.
6. **`Client`.** `at`/`size` are `(i32, i32)`. New fields: `pin_fullscreened`,
   `fullscreen_handler`, `allowed_over_fullscreen`, `tearing_hint`. `over_fullscreen` is
   deprecated (Hyprland no longer emits it).
7. **New `Event` variants.** Ten socket-2 events were added (`Bell`, `WindowMinimized`,
   `WindowKilled`, `Custom`, `WorkspaceIdChanged`, `SpecialRemovedV2`, `ChangedSpecialV2`,
   `ActiveMonitorChangedV2`, `MonitorRemovedV2`, `ScreencastV2`) — handled by the `_` arm above.
8. **`OptionValue::Bool(bool)`** and the `OptionDescriptions` / `OptionDescription` data type for
   discovering real option names via `hyprctl -j descriptions`.
9. **Removed:** `data::regular::Bezier` (public but unused; `Animations` holds `BezierIdent`).
10. **`WorkspaceRuleset`** gained the remaining upstream keys; `WorkspaceType` deserialisation was
    corrected. Both are additive to construction but change parsed values.
11. **MSRV is now 1.88** (`chardetng` 1.0). Update your toolchain before upgrading.

Lua-config dispatch is **not** part of `0.5.0`; it is the next line of work.

## `0.2.5` ⟶ `0.3.0`
> This release is action packed so be prepared!

Steps (This order is recommended so lsp doesn't scream at you lol)
1. Add the prelude! This new release includes traits! `use hyprland::prelude::*;`
2. Switch to the `Result` type — it moved to the crate root: `use hyprland::Result;` (it was `hyprland::shared::HResult` in 0.2.5)
3. [Update your data fetcher functions to structs](#update-data-fetchers)
4. [Update your dispatch functions to structs](#update-dispatchers)
5. Switch all `String`s to `&str`s in your dispatchers
6. Switch from `Positive`/`Negative` `Relative` dispatchers to just `Relative..`, and signed ints
7. When using `WorkspaceType::Special`, now include a `Some(name: String)` or `None`
8. [Update your keyword calls to new module](#update-keyword-calls)
9. [Update start listener calls](#update-start-listener-calls)
10. Follow Rustc and Clippy for everything else! 

## `0.3.*` ⟶ `0.3.3`
* Changes some integer types
* Switch to new event structs (no longer tuple!)

## `0.3.0` ⟶ `0.3.1`
* Change some types in the data
structs, because they changed (Only int types)
* Fix some dispatchers (ex. `ToggleSpecialWorkspace`)
* Update active window event, cuz I merged it with activewindowv2
* Update location of `Result` type, cuz I changed it

## `0.3.3` ⟶ `0.3.4`
* EventListener now requires mutable reference to start

## More in-depth steps

### 0.3 Update

#### Update Data fetchers
| Notation Type | Async Notation                     | Blocking Notation           |
|---------------|------------------------------------|-----------------------------|
| Old notation  | `asynchronous::get_something()`    | `blocking::get_something()` |
| New Notation  | `Something::get_async().collect()` | `Something::get().collect()`|
> The `.collect()` is required because data fetchers are now iterators!
>
> If you are just converting to a `Vec`, **USE THE `.to_vec()`** method instead,
> this being an iterator is useful if you wanna for example loop over all the workspaces


#### Update Dispatchers

| Notation Type | Async Notation           | Blocking Notation     |
|---------------|--------------------------|-----------------------|
| Old notation  | `dispatch()`             | `dispatch_blocking()` |
| New Notation  | `Dispatch::call_async()` | `Dispatch::call()`    |

#### Update start listener calls
| Notation Type | Async Notation           | Blocking Notation           |
|---------------|--------------------------|-----------------------------|
| Old notation  | `start_listener()`       | `start_listener_blocking()` |
| New Notation  | `start_listener_async()` | `start_listener()`          |

#### Update Keyword calls
This has gone through a complete rework, and now is in the `keyword` module

##### Setting a keyword
Blocking: `Keyword::set(key, val)`<br />
Asynchronous: `Keyword::set_async(key, val)`

##### Getting a keyword 
> AKA getoption

Blocking: `Keyword::get(key)`<br />
Asynchronous: `Keyword::get_async(key)`
