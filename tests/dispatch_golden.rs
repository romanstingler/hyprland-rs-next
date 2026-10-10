//! Golden tests pinning the exact wire format of every dispatcher.
//!
//! [`gen_dispatch_str`] renders a [`DispatchType`] two ways:
//! - socket mode (`dispatch = true`): `dispatch <name> <args>`, space-separated,
//!   wrapped in the JSON command flag.
//! - bind mode (`dispatch = false`): `<name>,<args>`, a comma only after the
//!   dispatcher name, wrapped in the empty command flag.
//!
//! Hyprland reads a bind line as `CVarList(value, 4)` — MODS,KEY,DISPATCHER,ARGS
//! (`ConfigManager.cpp`, `handleBind`) — so only the dispatcher name must be
//! followed by `,`; the args keep their normal space-separated syntax.
//!
//! The expected strings are literals, derived from the upstream dispatcher names,
//! not from the implementation. When a new [`DispatchType`] variant is added,
//! add a row here too.

use hyprland::dispatch::gen_dispatch_str;
use hyprland::dispatch::{
    BinaryState, Corner, CycleDirection, Direction, DispatchType, FloatValue, FocusMasterParam,
    FullscreenState, FullscreenType, LockType, MasterLoopParam, MonitorIdentifier, MoveToRootParam,
    OrientationParam, Position, SignalType, SubmapParam, SwapWithMasterParam, TagAction,
    WindowIdentifier, WindowMove, WindowSwitchDirection, WorkspaceIdentifier,
    WorkspaceIdentifierWithSpecial, WorkspaceOptions, ZOrder,
};
use hyprland::shared::{Address, CommandFlag, Mod};

fn gold<'a>(dispatcher: DispatchType<'a>, socket: &str, bind: &str) {
    let socket_cmd = gen_dispatch_str(dispatcher.clone(), true).unwrap();
    assert_eq!(socket_cmd.data, socket);
    assert_eq!(socket_cmd.flag, CommandFlag::JSON);

    let bind_cmd = gen_dispatch_str(dispatcher, false).unwrap();
    assert_eq!(bind_cmd.data, bind);
    assert_eq!(bind_cmd.flag, CommandFlag::Empty);
}

#[test]
fn generic_and_window_dispatchers() {
    let cases = [
        (
            DispatchType::Custom("foo", "bar baz"),
            "dispatch foo bar baz",
            "foo,bar baz",
        ),
        (
            DispatchType::Exec("kitty"),
            "dispatch exec kitty",
            "exec,kitty",
        ),
        (
            DispatchType::ExecRaw("echo hi"),
            "dispatch execr echo hi",
            "execr,echo hi",
        ),
        (
            DispatchType::Pass(WindowIdentifier::ActiveWindow),
            "dispatch pass activewindow",
            "pass,activewindow",
        ),
        (
            DispatchType::Pass(WindowIdentifier::ProcessId(4242)),
            "dispatch pass pid:4242",
            "pass,pid:4242",
        ),
        (
            DispatchType::Global("foo"),
            "dispatch global foo",
            "global,foo",
        ),
        (
            DispatchType::KillActiveWindow,
            "dispatch killactive",
            "killactive",
        ),
        (
            DispatchType::ForceKillActiveWindow,
            "dispatch forcekillactive",
            "forcekillactive",
        ),
        (
            DispatchType::CloseWindow(WindowIdentifier::Address(Address::new("0x123"))),
            "dispatch closewindow address:0x123",
            "closewindow,address:0x123",
        ),
        (
            DispatchType::CloseWindow(WindowIdentifier::Title("foo")),
            "dispatch closewindow title:foo",
            "closewindow,title:foo",
        ),
    ];

    for (dispatcher, socket, bind) in cases {
        gold(dispatcher, socket, bind);
    }
}

#[test]
fn workspace_dispatchers() {
    let cases = [
        (
            DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(5)),
            "dispatch workspace 5",
            "workspace,5",
        ),
        (
            DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Relative(2)),
            "dispatch workspace +2",
            "workspace,+2",
        ),
        (
            DispatchType::Workspace(WorkspaceIdentifierWithSpecial::RelativeMonitor(-1)),
            "dispatch workspace m-1",
            "workspace,m-1",
        ),
        (
            DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Empty(
                hyprland::dispatch::FirstEmpty {
                    on_monitor: true,
                    next: true,
                },
            )),
            "dispatch workspace emptymn",
            "workspace,emptymn",
        ),
        (
            DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Special(Some("scratch"))),
            "dispatch workspace special:scratch",
            "workspace,special:scratch",
        ),
        (
            DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Special(None)),
            "dispatch workspace special",
            "workspace,special",
        ),
        (
            DispatchType::MoveToWorkspace(
                WorkspaceIdentifierWithSpecial::Id(3),
                Some(WindowIdentifier::ActiveWindow),
            ),
            "dispatch movetoworkspace 3,activewindow",
            "movetoworkspace,3,activewindow",
        ),
        (
            DispatchType::MoveToWorkspace(WorkspaceIdentifierWithSpecial::Id(3), None),
            "dispatch movetoworkspace 3",
            "movetoworkspace,3",
        ),
        (
            DispatchType::MoveToWorkspaceSilent(
                WorkspaceIdentifierWithSpecial::Name("web"),
                Some(WindowIdentifier::ActiveWindow),
            ),
            "dispatch movetoworkspacesilent name:web,activewindow",
            "movetoworkspacesilent,name:web,activewindow",
        ),
        (
            DispatchType::MoveToWorkspaceSilent(WorkspaceIdentifierWithSpecial::Id(3), None),
            "dispatch movetoworkspacesilent 3",
            "movetoworkspacesilent,3",
        ),
        (
            DispatchType::MoveCurrentWorkspaceToMonitor(MonitorIdentifier::Name("DP-1")),
            "dispatch movecurrentworkspacetomonitor DP-1",
            "movecurrentworkspacetomonitor,DP-1",
        ),
        (
            DispatchType::MoveWorkspaceToMonitor(
                WorkspaceIdentifier::RelativeOpen(1),
                MonitorIdentifier::Id(1),
            ),
            "dispatch moveworkspacetomonitor e+1 1",
            "moveworkspacetomonitor,e+1 1",
        ),
        (
            DispatchType::SwapActiveWorkspaces(MonitorIdentifier::Id(0), MonitorIdentifier::Id(1)),
            "dispatch swapactiveworkspaces 0 1",
            "swapactiveworkspaces,0 1",
        ),
        (
            DispatchType::RenameWorkspace(5, Some("web")),
            "dispatch renameworkspace 5 web",
            "renameworkspace,5 web",
        ),
        (
            DispatchType::RenameWorkspace(5, None),
            "dispatch renameworkspace 5 5",
            "renameworkspace,5 5",
        ),
        (
            DispatchType::WorkspaceOption(WorkspaceOptions::AllPseudo),
            "dispatch workspaceopt allpseudo",
            "workspaceopt,allpseudo",
        ),
        (
            DispatchType::ToggleSpecialWorkspace(Some("scratch".to_owned())),
            "dispatch togglespecialworkspace scratch",
            "togglespecialworkspace,scratch",
        ),
        (
            DispatchType::ToggleSpecialWorkspace(None),
            "dispatch togglespecialworkspace",
            "togglespecialworkspace",
        ),
    ];

    for (dispatcher, socket, bind) in cases {
        gold(dispatcher, socket, bind);
    }
}

#[test]
fn window_state_dispatchers() {
    let cases = [
        (
            DispatchType::ToggleFloating(Some(WindowIdentifier::ActiveWindow)),
            "dispatch togglefloating activewindow",
            "togglefloating,activewindow",
        ),
        (
            DispatchType::ToggleFloating(None),
            "dispatch togglefloating",
            "togglefloating",
        ),
        (
            DispatchType::SetFloating(Some(WindowIdentifier::ActiveWindow)),
            "dispatch setfloating activewindow",
            "setfloating,activewindow",
        ),
        (
            DispatchType::SetFloating(None),
            "dispatch setfloating",
            "setfloating",
        ),
        (
            DispatchType::SetTiled(Some(WindowIdentifier::ActiveWindow)),
            "dispatch settiled activewindow",
            "settiled,activewindow",
        ),
        (
            DispatchType::SetTiled(None),
            "dispatch settiled",
            "settiled",
        ),
        (
            DispatchType::ToggleFullscreen(FullscreenType::Real),
            "dispatch fullscreen 0",
            "fullscreen,0",
        ),
        (
            DispatchType::ToggleFullscreen(FullscreenType::Maximize),
            "dispatch fullscreen 1",
            "fullscreen,1",
        ),
        (
            DispatchType::ToggleFullscreen(FullscreenType::NoParam),
            "dispatch fullscreen ",
            "fullscreen,",
        ),
        (
            DispatchType::ToggleFullscreenState(
                FullscreenState::Maximize,
                FullscreenState::Fullscreen,
            ),
            "dispatch fullscreenstate 1 2",
            "fullscreenstate,1 2",
        ),
        (
            DispatchType::ToggleFakeFullscreen,
            "dispatch fakefullscreen",
            "fakefullscreen",
        ),
        (
            DispatchType::ToggleDPMS(true, Some("HDMI-A-1")),
            "dispatch dpms on HDMI-A-1",
            "dpms,on HDMI-A-1",
        ),
        (
            DispatchType::ToggleDPMS(false, None),
            "dispatch dpms off ",
            "dpms,off ",
        ),
        (DispatchType::TogglePseudo, "dispatch pseudo", "pseudo"),
        (DispatchType::TogglePin, "dispatch pin", "pin"),
        (
            DispatchType::TogglePinWindow(WindowIdentifier::Floating),
            "dispatch pin floating",
            "pin,floating",
        ),
        (
            DispatchType::ToggleOpaque,
            "dispatch toggleopaque",
            "toggleopaque",
        ),
        (
            DispatchType::ToggleSwallow,
            "dispatch toggleswallow",
            "toggleswallow",
        ),
        (
            DispatchType::BringActiveToTop,
            "dispatch bringactivetotop",
            "bringactivetotop",
        ),
        (
            DispatchType::AlterZOrder(ZOrder::Top, Some(WindowIdentifier::ActiveWindow)),
            "dispatch alterzorder top,activewindow",
            "alterzorder,top,activewindow",
        ),
        (
            DispatchType::AlterZOrder(ZOrder::Bottom, None),
            "dispatch alterzorder bottom",
            "alterzorder,bottom",
        ),
        (
            DispatchType::CenterWindow,
            "dispatch centerwindow",
            "centerwindow",
        ),
        (DispatchType::Exit, "dispatch exit", "exit"),
        (
            DispatchType::ForceRendererReload,
            "dispatch forcerendererreload",
            "forcerendererreload",
        ),
    ];

    for (dispatcher, socket, bind) in cases {
        gold(dispatcher, socket, bind);
    }
}

#[test]
fn focus_move_and_resize_dispatchers() {
    let cases = [
        (
            DispatchType::MoveFocus(Direction::Left),
            "dispatch movefocus l",
            "movefocus,l",
        ),
        (
            DispatchType::MoveWindow(WindowMove::Direction(Direction::Right)),
            "dispatch movewindow r",
            "movewindow,r",
        ),
        (
            DispatchType::MoveWindow(WindowMove::Monitor(MonitorIdentifier::Id(2))),
            "dispatch movewindow mon:2",
            "movewindow,mon:2",
        ),
        (
            DispatchType::FocusWindow(WindowIdentifier::ClassRegularExpression("kitty")),
            "dispatch focuswindow class:kitty",
            "focuswindow,class:kitty",
        ),
        (
            DispatchType::FocusMonitor(MonitorIdentifier::Current),
            "dispatch focusmonitor current",
            "focusmonitor,current",
        ),
        (
            DispatchType::FocusMonitor(MonitorIdentifier::Direction(Direction::Up)),
            "dispatch focusmonitor u",
            "focusmonitor,u",
        ),
        (
            DispatchType::FocusMonitor(MonitorIdentifier::Relative(-1)),
            "dispatch focusmonitor -1",
            "focusmonitor,-1",
        ),
        (
            DispatchType::FocusUrgentOrLast,
            "dispatch focusurgentorlast",
            "focusurgentorlast",
        ),
        (
            DispatchType::FocusCurrentOrLast,
            "dispatch focuscurrentorlast",
            "focuscurrentorlast",
        ),
        (
            DispatchType::CycleWindow(CycleDirection::Next),
            "dispatch cyclenext ",
            "cyclenext,",
        ),
        (
            DispatchType::CycleWindow(CycleDirection::Previous),
            "dispatch cyclenext prev",
            "cyclenext,prev",
        ),
        (
            DispatchType::SwapNext(CycleDirection::Previous),
            "dispatch swapnext prev",
            "swapnext,prev",
        ),
        (
            DispatchType::SwapWindow(Direction::Up),
            "dispatch swapwindow u",
            "swapwindow,u",
        ),
        (
            DispatchType::TagWindow(TagAction::Add, "1", Some(WindowIdentifier::ActiveWindow)),
            "dispatch tagwindow +1 activewindow",
            "tagwindow,+1 activewindow",
        ),
        (
            DispatchType::TagWindow(TagAction::Toggle, "2", None),
            "dispatch tagwindow 2",
            "tagwindow,2",
        ),
        (
            DispatchType::MoveCursorToCorner(Corner::TopLeft),
            "dispatch movecursortocorner 3",
            "movecursortocorner,3",
        ),
        (
            DispatchType::MoveCursor(100, 200),
            "dispatch movecursor 100 200",
            "movecursor,100 200",
        ),
        (
            DispatchType::ResizeActive(Position::Delta(10, -5)),
            "dispatch resizeactive 10 -5",
            "resizeactive,10 -5",
        ),
        (
            DispatchType::ResizeActive(Position::ExactFraction(50, 50)),
            "dispatch resizeactive exact 50% 50%",
            "resizeactive,exact 50% 50%",
        ),
        (
            DispatchType::MoveActive(Position::Delta(0, 50)),
            "dispatch moveactive 0 50",
            "moveactive,0 50",
        ),
        (
            DispatchType::ResizeWindowPixel(
                Position::Delta(10, 10),
                WindowIdentifier::ActiveWindow,
            ),
            "dispatch resizewindowpixel 10 10,activewindow",
            "resizewindowpixel,10 10,activewindow",
        ),
        (
            DispatchType::MoveWindowPixel(
                Position::Exact(100, 100),
                WindowIdentifier::ActiveWindow,
            ),
            "dispatch movewindowpixel exact 100 100,activewindow",
            "movewindowpixel,exact 100 100,activewindow",
        ),
        (
            DispatchType::ChangeSplitRatio(FloatValue::Relative(-0.05)),
            "dispatch splitratio -0.05",
            "splitratio,-0.05",
        ),
        (
            DispatchType::ChangeSplitRatio(FloatValue::Exact(0.75)),
            "dispatch splitratio exact 0.75",
            "splitratio,exact 0.75",
        ),
    ];

    for (dispatcher, socket, bind) in cases {
        gold(dispatcher, socket, bind);
    }
}

#[test]
fn signal_and_shortcut_dispatchers() {
    let cases = [
        (
            DispatchType::Signal(SignalType::SIGTERM),
            "dispatch signal 15",
            "signal,15",
        ),
        (
            DispatchType::SignalWindow(WindowIdentifier::ActiveWindow, SignalType::SIGHUP),
            "dispatch signalwindow activewindow,1",
            "signalwindow,activewindow,1",
        ),
        (
            DispatchType::SendShortcut(
                &[Mod::SUPER, Mod::SHIFT],
                "A",
                Some(WindowIdentifier::ActiveWindow),
            ),
            "dispatch sendshortcut SUPERSHIFT,A,activewindow",
            "sendshortcut,SUPERSHIFT,A,activewindow",
        ),
        (
            DispatchType::SendShortcut(&[Mod::CTRL], "B", None),
            "dispatch sendshortcut CTRL,B,",
            "sendshortcut,CTRL,B,",
        ),
    ];

    for (dispatcher, socket, bind) in cases {
        gold(dispatcher, socket, bind);
    }
}

#[test]
fn layout_dispatchers() {
    let cases = [
        (
            DispatchType::ToggleSplit,
            "dispatch layoutmsg togglesplit",
            "layoutmsg,togglesplit",
        ),
        (
            DispatchType::SwapSplit,
            "dispatch layoutmsg swapsplit",
            "layoutmsg,swapsplit",
        ),
        (
            DispatchType::PreSelect(Direction::Down),
            "dispatch layoutmsg preselect d",
            "layoutmsg,preselect d",
        ),
        (
            DispatchType::MoveToRoot(
                Some(WindowIdentifier::ActiveWindow),
                MoveToRootParam::Unstable,
            ),
            "dispatch layoutmsg movetoroot activewindow unstable",
            "layoutmsg,movetoroot activewindow unstable",
        ),
        (
            DispatchType::MoveToRoot(None, MoveToRootParam::Stable),
            "dispatch layoutmsg movetoroot",
            "layoutmsg,movetoroot",
        ),
        (
            DispatchType::Submap(SubmapParam::Reset),
            "dispatch submap reset",
            "submap,reset",
        ),
        (
            DispatchType::Submap(SubmapParam::Name("foo")),
            "dispatch submap foo",
            "submap,foo",
        ),
        (
            DispatchType::SwapWithMaster(SwapWithMasterParam::Master),
            "dispatch layoutmsg swapwithmaster master",
            "layoutmsg,swapwithmaster master",
        ),
        (
            DispatchType::FocusMaster(FocusMasterParam::Auto),
            "dispatch layoutmsg focusmaster auto",
            "layoutmsg,focusmaster auto",
        ),
        (
            DispatchType::CycleNextMaster(MasterLoopParam::Loop),
            "dispatch layoutmsg cyclenext loop",
            "layoutmsg,cyclenext loop",
        ),
        (
            DispatchType::CyclePrevMaster(MasterLoopParam::NoLoop),
            "dispatch layoutmsg cycleprev noloop",
            "layoutmsg,cycleprev noloop",
        ),
        (
            DispatchType::SwapNextMaster(MasterLoopParam::Loop),
            "dispatch layoutmsg swapnext loop",
            "layoutmsg,swapnext loop",
        ),
        (
            DispatchType::SwapPrevMaster(MasterLoopParam::NoLoop),
            "dispatch layoutmsg swapprev noloop",
            "layoutmsg,swapprev noloop",
        ),
        (
            DispatchType::AddMaster,
            "dispatch layoutmsg addmaster",
            "layoutmsg,addmaster",
        ),
        (
            DispatchType::RemoveMaster,
            "dispatch layoutmsg removemaster",
            "layoutmsg,removemaster",
        ),
        (
            DispatchType::OrientationLeft,
            "dispatch layoutmsg orientationleft",
            "layoutmsg,orientationleft",
        ),
        (
            DispatchType::OrientationRight,
            "dispatch layoutmsg orientationright",
            "layoutmsg,orientationright",
        ),
        (
            DispatchType::OrientationTop,
            "dispatch layoutmsg orientationtop",
            "layoutmsg,orientationtop",
        ),
        (
            DispatchType::OrientationBottom,
            "dispatch layoutmsg orientationbottom",
            "layoutmsg,orientationbottom",
        ),
        (
            DispatchType::OrientationCenter,
            "dispatch layoutmsg orientationcenter",
            "layoutmsg,orientationcenter",
        ),
        (
            DispatchType::OrientationNext,
            "dispatch layoutmsg orientationnext",
            "layoutmsg,orientationnext",
        ),
        (
            DispatchType::OrientationPrev,
            "dispatch layoutmsg orientationprev",
            "layoutmsg,orientationprev",
        ),
        (
            DispatchType::OrientationCycle(OrientationParam::Center),
            "dispatch layoutmsg orientationcycle center",
            "layoutmsg,orientationcycle center",
        ),
        (
            DispatchType::Mfact(FloatValue::Relative(0.5)),
            "dispatch layoutmsg mfact 0.5",
            "layoutmsg,mfact 0.5",
        ),
        (
            DispatchType::RollNext,
            "dispatch layoutmsg rollnext",
            "layoutmsg,rollnext",
        ),
        (
            DispatchType::RollPrev,
            "dispatch layoutmsg rollprev",
            "layoutmsg,rollprev",
        ),
    ];

    for (dispatcher, socket, bind) in cases {
        gold(dispatcher, socket, bind);
    }
}

#[test]
fn group_dispatchers() {
    let cases = [
        (
            DispatchType::ToggleGroup,
            "dispatch togglegroup",
            "togglegroup",
        ),
        (
            DispatchType::ChangeGroupActive(WindowSwitchDirection::Forward),
            "dispatch changegroupactive f",
            "changegroupactive,f",
        ),
        (
            DispatchType::ChangeGroupActive(WindowSwitchDirection::Index(3)),
            "dispatch changegroupactive 3",
            "changegroupactive,3",
        ),
        (
            DispatchType::LockGroups(LockType::Lock),
            "dispatch lockgroups lock",
            "lockgroups,lock",
        ),
        (
            DispatchType::LockActiveGroup(LockType::Unlock),
            "dispatch lockactivegroups unlock",
            "lockactivegroups,unlock",
        ),
        (
            DispatchType::MoveIntoGroup(Direction::Right),
            "dispatch moveintogroup r",
            "moveintogroup,r",
        ),
        (
            DispatchType::MoveWindowOrGroup(Direction::Left),
            "dispatch movewindoworgroup l",
            "movewindoworgroup,l",
        ),
        (
            DispatchType::MoveOutOfGroup,
            "dispatch moveoutofgroup",
            "moveoutofgroup",
        ),
        (
            DispatchType::MoveGroupWindow(WindowSwitchDirection::Back),
            "dispatch movegroupwindow b",
            "movegroupwindow,b",
        ),
        (
            DispatchType::DenyWindowFromGroup(BinaryState::On),
            "dispatch denywindowfromgroup on",
            "denywindowfromgroup,on",
        ),
        (
            DispatchType::SetIgnoreGroupLock(BinaryState::Toggle),
            "dispatch setignoregrouplock toggle",
            "setignoregrouplock,toggle",
        ),
    ];

    for (dispatcher, socket, bind) in cases {
        gold(dispatcher, socket, bind);
    }
}

/// `SetCursor` is the one dispatcher that ignores the mode: it always renders
/// as a JSON `setcursor` command, in both socket and bind form.
#[test]
fn set_cursor_is_always_json() {
    let socket_cmd = gen_dispatch_str(DispatchType::SetCursor("Adwaita", 24), true).unwrap();
    assert_eq!(socket_cmd.data, "setcursor Adwaita 24");
    assert_eq!(socket_cmd.flag, CommandFlag::JSON);

    let bind_cmd = gen_dispatch_str(DispatchType::SetCursor("Adwaita", 24), false).unwrap();
    assert_eq!(bind_cmd.data, "setcursor Adwaita 24");
    assert_eq!(bind_cmd.flag, CommandFlag::JSON);
}
