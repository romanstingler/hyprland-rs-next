# Hyprland-rs (next)

[![Crates.io](https://img.shields.io/crates/l/hyprland)](https://www.gnu.org/licenses/gpl-3.0.html)
[![Hyprland](https://img.shields.io/badge/Made%20for-Hyprland-blue)](https://github.com/hyprwm/Hyprland)

An unofficial rust wrapper for Hyprland's IPC

## This is a fork

`hyprland-rs-next` is maintained independently of
[hyprland-community/hyprland-rs](https://github.com/hyprland-community/hyprland-rs).
It tracks upstream `master` and finishes the legacy (`.conf`) code path that
upstream is removing.

One Hyprland binary serves both config grammars and picks at runtime, so
`dispatch`, `keyword::set` and `config::binds` work on a `.conf` config and
are dead on a `.lua` one, where `hyprctl dispatch` evaluates its argument as
Lua source. The goal here is to make both paths complete and correct, then
add a runtime probe that picks the right one automatically, so this crate
keeps working on either config until the legacy path is finally dropped.

Fixes landed so far: `Position` was discarding its Y coordinate, so
`moveactive 10 20` emitted `moveactive 10 10`; `WorkspaceOptions` labels were
swapped; `changefloatingmode` reported the inverted value;
`Screencast::monitor` was permanently false; and three dispatchers produced
malformed bind lines.

**The Lua dispatch path does not work yet**, in this fork or upstream. That
is the next piece of work.

To depend on it:

```toml
hyprland = { git = "https://github.com/romanstingler/hyprland-rs-next", branch = "master" }
```

Upstream remains at `hyprland = "0.4.0"`. See [CHANGELOG notes](https://github.com/romanstingler/hyprland-rs-next/releases) for what is fixed.

### What this crate provides

This crate provides 8 modules (+1 for shared things)

- `data` for getting information on the compositor
- `event_listener` which provides the `EventListener` struct for listening for events
- `dispatch` for calling dispatchers
- `keyword` for dealing with config option (aka keywords)
- `config::binds` for changing binds (in future `config` might have config generation)
- `ctl` for calling hyprctl commands
- `hyprpaper` for talking to hyprpaper
- `instance` for targeting a specific running Hyprland instance

## Example Usage

Check the examples in the [`examples` directory](https://github.com/romanstingler/hyprland-rs-next/tree/master/examples)
