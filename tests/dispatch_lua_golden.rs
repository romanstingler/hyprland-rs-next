use hyprland::dispatch::{
    BinaryState, Corner, CycleDirection, Direction, DispatchType, FloatValue, FocusMasterParam,
    FullscreenState, FullscreenType, LockType, MasterLoopParam, MonitorIdentifier, MoveToRootParam,
    OrientationParam, Position, SignalType, SubmapParam, SwapWithMasterParam, TagAction,
    WindowIdentifier, WindowMove, WindowSwitchDirection, WorkspaceIdentifier,
    WorkspaceIdentifierWithSpecial, WorkspaceOptions, ZOrder, gen_lua_dispatch_expr,
    gen_lua_dispatch_str,
};
use hyprland::shared::{Address, CommandFlag, Mod};

fn expr(dispatcher: DispatchType<'_>) -> String {
    gen_lua_dispatch_expr(&dispatcher).unwrap()
}

fn all_supported() -> Vec<DispatchType<'static>> {
    use DispatchType::*;

    let window = || WindowIdentifier::ClassRegularExpression("kitty");
    let workspace = || WorkspaceIdentifierWithSpecial::Id(3);
    let monitor = || MonitorIdentifier::Name("DP-1");

    vec![
        Exec("kitty"),
        ExecRaw("echo hi"),
        Pass(WindowIdentifier::ActiveWindow),
        Global("name"),
        KillActiveWindow,
        ForceKillActiveWindow,
        CloseWindow(WindowIdentifier::Address(Address::new("55a"))),
        Workspace(workspace()),
        MoveToWorkspace(workspace(), Some(window())),
        MoveToWorkspaceSilent(workspace(), None),
        ToggleFloating(Some(window())),
        SetFloating(None),
        SetTiled(None),
        ToggleFullscreen(FullscreenType::Real),
        ToggleFullscreen(FullscreenType::Maximize),
        ToggleFullscreen(FullscreenType::NoParam),
        ToggleFullscreenState(FullscreenState::Maximize, FullscreenState::None),
        ToggleDPMS(true, Some("DP-1")),
        ToggleDPMS(false, None),
        TogglePseudo,
        TogglePin,
        TogglePinWindow(window()),
        SendShortcut(&[Mod::SUPER, Mod::SHIFT], "A", Some(window())),
        Signal(SignalType::SIGTERM),
        SignalWindow(window(), SignalType::SIGKILL),
        MoveFocus(Direction::Left),
        MoveWindow(WindowMove::Direction(Direction::Up)),
        MoveWindow(WindowMove::Monitor(monitor())),
        CenterWindow,
        ResizeActive(Position::Delta(10, -10)),
        MoveActive(Position::Exact(100, 200)),
        ResizeActive(Position::DeltaFraction(10, 20)),
        MoveActive(Position::ExactFraction(50, 50)),
        ResizeWindowPixel(Position::Delta(5, 5), window()),
        MoveWindowPixel(Position::Exact(1, 2), window()),
        MoveWindowPixel(Position::DeltaFraction(10, 20), window()),
        CycleWindow(CycleDirection::Next),
        SwapNext(CycleDirection::Previous),
        SwapWindow(Direction::Right),
        TagWindow(TagAction::Add, "foo", Some(window())),
        FocusWindow(window()),
        FocusMonitor(monitor()),
        ChangeSplitRatio(FloatValue::Exact(0.5)),
        ToggleOpaque,
        MoveCursorToCorner(Corner::TopLeft),
        MoveCursor(10, 20),
        RenameWorkspace(5, Some("web")),
        RenameWorkspace(5, None),
        Exit,
        ForceRendererReload,
        MoveCurrentWorkspaceToMonitor(monitor()),
        MoveWorkspaceToMonitor(WorkspaceIdentifier::Id(3), monitor()),
        SwapActiveWorkspaces(MonitorIdentifier::Id(0), MonitorIdentifier::Id(1)),
        BringActiveToTop,
        AlterZOrder(ZOrder::Top, Some(window())),
        ToggleSpecialWorkspace(Some("scratch".to_string())),
        ToggleSpecialWorkspace(None),
        FocusUrgentOrLast,
        FocusCurrentOrLast,
        ToggleSwallow,
        Submap(SubmapParam::Reset),
        ToggleSplit,
        SwapSplit,
        PreSelect(Direction::Left),
        MoveToRoot(Some(window()), MoveToRootParam::Unstable),
        SwapWithMaster(SwapWithMasterParam::Master),
        FocusMaster(FocusMasterParam::Auto),
        CycleNextMaster(MasterLoopParam::Loop),
        CyclePrevMaster(MasterLoopParam::NoLoop),
        SwapNextMaster(MasterLoopParam::Loop),
        SwapPrevMaster(MasterLoopParam::NoLoop),
        AddMaster,
        RemoveMaster,
        OrientationLeft,
        OrientationRight,
        OrientationTop,
        OrientationBottom,
        OrientationCenter,
        OrientationNext,
        OrientationPrev,
        OrientationCycle(OrientationParam::Top),
        Mfact(FloatValue::Relative(0.1)),
        RollNext,
        RollPrev,
        ToggleGroup,
        ChangeGroupActive(WindowSwitchDirection::Forward),
        ChangeGroupActive(WindowSwitchDirection::Back),
        ChangeGroupActive(WindowSwitchDirection::Index(2)),
        LockGroups(LockType::Lock),
        LockActiveGroup(LockType::Unlock),
        MoveIntoGroup(Direction::Left),
        MoveWindowOrGroup(Direction::Right),
        MoveOutOfGroup,
        MoveGroupWindow(WindowSwitchDirection::Forward),
        MoveGroupWindow(WindowSwitchDirection::Back),
        DenyWindowFromGroup(BinaryState::On),
        SetIgnoreGroupLock(BinaryState::Toggle),
    ]
}

#[test]
fn every_supported_variant_renders_a_dispatcher_expression() {
    for dispatcher in all_supported() {
        let rendered = expr(dispatcher.clone());
        assert!(
            rendered.starts_with("hl.dsp.") || rendered.starts_with("(function"),
            "{dispatcher:?} rendered as {rendered}"
        );
    }
}

#[test]
fn generic_dispatchers() {
    assert_eq!(
        expr(DispatchType::Exec("kitty")),
        r#"hl.dsp.exec_cmd("kitty")"#
    );
    assert_eq!(
        expr(DispatchType::ExecRaw("echo hi")),
        r#"hl.dsp.exec_raw("echo hi")"#
    );
    assert_eq!(
        expr(DispatchType::Global("name")),
        r#"hl.dsp.global("name")"#
    );
    assert_eq!(
        expr(DispatchType::KillActiveWindow),
        "hl.dsp.window.close()"
    );
    assert_eq!(
        expr(DispatchType::ForceKillActiveWindow),
        "hl.dsp.window.kill()"
    );
}

#[test]
fn focus_and_workspace_dispatchers() {
    assert_eq!(
        expr(DispatchType::MoveFocus(Direction::Left)),
        r#"hl.dsp.focus({ direction = "left" })"#
    );
    assert_eq!(
        expr(DispatchType::Workspace(
            WorkspaceIdentifierWithSpecial::Name("web")
        )),
        r#"hl.dsp.focus({ workspace = "name:web" })"#
    );
    assert_eq!(
        expr(DispatchType::FocusMonitor(MonitorIdentifier::Name("DP-1"))),
        r#"hl.dsp.focus({ monitor = "DP-1" })"#
    );
    assert_eq!(
        expr(DispatchType::FocusUrgentOrLast),
        "hl.dsp.focus({ urgent_or_last = true })"
    );
}

#[test]
fn window_positions_are_relative_where_legacy_deltas_were() {
    assert_eq!(
        expr(DispatchType::MoveActive(Position::Delta(10, -10))),
        "hl.dsp.window.move({ x = 10, y = -10, relative = true })"
    );
    assert_eq!(
        expr(DispatchType::ResizeActive(Position::Exact(100, 200))),
        "hl.dsp.window.resize({ x = 100, y = 200, relative = false })"
    );
    assert_eq!(
        expr(DispatchType::MoveWindowPixel(
            Position::Exact(1, 2),
            WindowIdentifier::ClassRegularExpression("kitty")
        )),
        r#"hl.dsp.window.move({ x = 1, y = 2, relative = false, window = "class:kitty" })"#
    );
}

#[test]
fn fractional_positions_are_computed_at_dispatch_time() {
    assert_eq!(
        expr(DispatchType::MoveActive(Position::DeltaFraction(10, 20))),
        "(function(w) if not w then return hl.dsp.no_op() end return hl.dsp.window.move({ x = w.at.x * 0.1, y = w.at.y * 0.2, relative = true }) end)(hl.get_active_window())"
    );
    assert_eq!(
        expr(DispatchType::ResizeActive(Position::ExactFraction(50, 50))),
        "(function(m) if not m then return hl.dsp.no_op() end return hl.dsp.window.resize({ x = m.width * 0.5, y = m.height * 0.5 }) end)(hl.get_active_monitor())"
    );
    assert_eq!(
        expr(DispatchType::MoveWindowPixel(
            Position::DeltaFraction(10, 20),
            WindowIdentifier::ClassRegularExpression("kitty")
        )),
        "(function(w) if not w then return hl.dsp.no_op() end return hl.dsp.window.move({ x = w.at.x * 0.1, y = w.at.y * 0.2, relative = true, window = \"class:kitty\" }) end)(hl.get_window(\"class:kitty\"))"
    );
}

#[test]
fn layout_dispatchers_go_through_layoutmsg() {
    assert_eq!(
        expr(DispatchType::ToggleSplit),
        r#"hl.dsp.layout("togglesplit")"#
    );
    assert_eq!(
        expr(DispatchType::ChangeSplitRatio(FloatValue::Exact(0.5))),
        r#"hl.dsp.layout("splitratio exact 0.5")"#
    );
    assert_eq!(
        expr(DispatchType::MoveToRoot(
            Some(WindowIdentifier::ClassRegularExpression("kitty")),
            MoveToRootParam::Unstable
        )),
        r#"hl.dsp.layout("movetoroot class:kitty unstable")"#
    );
}

#[test]
fn group_and_fullscreen_dispatchers() {
    assert_eq!(
        expr(DispatchType::ChangeGroupActive(
            WindowSwitchDirection::Forward
        )),
        "hl.dsp.group.next()"
    );
    assert_eq!(
        expr(DispatchType::ChangeGroupActive(WindowSwitchDirection::Back)),
        "hl.dsp.group.prev()"
    );
    assert_eq!(
        expr(DispatchType::LockGroups(LockType::Lock)),
        r#"hl.dsp.group.lock({ action = "enable" })"#
    );
    assert_eq!(
        expr(DispatchType::ToggleFullscreen(FullscreenType::Real)),
        r#"hl.dsp.window.fullscreen({ mode = "fullscreen", action = "set" })"#
    );
    assert_eq!(
        expr(DispatchType::ToggleFullscreenState(
            FullscreenState::Maximize,
            FullscreenState::None
        )),
        "hl.dsp.window.fullscreen_state({ internal = 1, client = 0 })"
    );
}

#[test]
fn unsupported_legacy_forms_name_the_capability() {
    for dispatcher in [
        DispatchType::Custom("foo", "bar"),
        DispatchType::SetCursor("Adwaita", 24),
        DispatchType::ToggleFakeFullscreen,
        DispatchType::WorkspaceOption(WorkspaceOptions::AllFloat),
    ] {
        assert!(
            matches!(
                gen_lua_dispatch_expr(&dispatcher),
                Err(hyprland::error::HyprError::Unsupported(_))
            ),
            "{dispatcher:?}"
        );
    }
}

#[test]
fn setcursor_uses_the_ipc_command() {
    let command = gen_lua_dispatch_str(DispatchType::SetCursor("Adwaita", 24)).unwrap();
    assert_eq!(command.data, "setcursor Adwaita 24");
    assert_eq!(command.flag, CommandFlag::JSON);
}

#[test]
fn raw_lua_is_passed_through() {
    assert_eq!(
        expr(DispatchType::Lua("hl.dsp.window.close()")),
        "hl.dsp.window.close()"
    );
}

#[test]
fn values_are_escaped() {
    assert_eq!(
        expr(DispatchType::Exec("kitty \"x\"\n")),
        r#"hl.dsp.exec_cmd("kitty \"x\"\n")"#
    );
    assert_eq!(
        expr(DispatchType::Global("a\") ; os.execute(\"id")),
        r#"hl.dsp.global("a\") ; os.execute(\"id")"#
    );
}

#[test]
fn bind_flags_are_rendered() {
    use hyprland::config::binds::{Binder, Binding, Flag, Key};

    let binding = Binding {
        mods: &[Mod::SUPER, Mod::SHIFT],
        key: Key::Key("Q"),
        flags: &[Flag::Locked, Flag::Repeat],
        dispatcher: DispatchType::KillActiveWindow,
    };
    assert_eq!(
        Binder::gen_lua_bind_str(binding).unwrap(),
        r#"hl.bind("SUPER + SHIFT + Q", hl.dsp.window.close(), { locked = true, repeating = true })"#
    );

    let set_cursor = Binding {
        mods: &[Mod::SUPER],
        key: Key::Key("C"),
        flags: &[],
        dispatcher: DispatchType::SetCursor("Adwaita", 24),
    };
    assert_eq!(
        Binder::gen_lua_bind_str(set_cursor).unwrap(),
        r#"hl.bind("SUPER + C", hl.dsp.exec_cmd("hyprctl setcursor Adwaita 24"))"#
    );
}
