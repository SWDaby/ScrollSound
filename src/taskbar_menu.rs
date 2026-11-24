//! Tray icon + context menu (tray-icon + muda) and menu action handlers
//! (design D3/D8/D10). All usage of the pre-1.0 tray crates is isolated here:
//! a breaking upgrade touches this module only.

use std::cell::RefCell;

use tray_icon::menu::{
    CheckMenuItem, IconMenuItem, IsMenuItem, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem,
};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

use windows::core::PCWSTR;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR};

use crate::platform;
use crate::platform::hook;

/// Application UI state (design D2's state layout). Lives on the pump thread:
/// menu handlers take it briefly; the hook callback never touches it.
pub struct App {
    /// Held for the tray icon's lifetime — drop removes it from the shell.
    #[allow(dead_code)]
    tray: TrayIcon,
    /// Held per design D2; the muda Rc tree keeps menu items addressable.
    #[allow(dead_code)]
    menu: Menu,
    pause_item: MenuItem,
    autostart_item: CheckMenuItem,
    admin_item: CheckMenuItem,
}

// Single-thread confinement: the pump and every handler run on the main
// thread, so a plain thread-local cell (no lock) is exactly the D2 contract.
thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

/// Constructs tray + menu and installs them as the app state.
pub fn init(autostart_enabled: bool, admin_enabled: bool) -> Result<(), String> {
    let autostart_item =
        CheckMenuItem::with_id("autostart", "开机自启(&R)", true, autostart_enabled, None);
    let separator = PredefinedMenuItem::separator();
    let admin_item =
        CheckMenuItem::with_id("admin", "管理员权限运行(&A)", true, admin_enabled, None);
    let pause_item = MenuItem::with_id("pause", "暂停", true, None);
    let rehook_item = MenuItem::with_id("rehook", "重新HOOK", true, None);
    let separator2 = PredefinedMenuItem::separator();
    let about_item =
        IconMenuItem::with_id("about", "关于(&G)", true, menu_icon(APP_ICON_ORDINAL), None);
    let exit_item =
        IconMenuItem::with_id("exit", "退出(&X)", true, menu_icon(EXIT_ICON_ORDINAL), None);

    let menu = Menu::new();
    let items: [&dyn IsMenuItem; 8] = [
        &autostart_item,
        &separator,
        &admin_item,
        &pause_item,
        &rehook_item,
        &separator2,
        &about_item,
        &exit_item,
    ];
    menu.append_items(&items)
        .map_err(|e| format!("build menu: {e}"))?;

    // Icon from the embedded resource (stable ordinal 156); fall back to a
    // `.ico` next to the exe (dev convenience) before giving up.
    let icon = match Icon::from_resource(APP_ICON_ORDINAL, None) {
        Ok(icon) => icon,
        Err(resource_err) => {
            tracing::warn!("Icon::from_resource failed: {resource_err}");
            let fallback = platform::win_settings::exe_dir().map(|d| d.join("scrollsound.ico"));
            match fallback {
                Some(p) => {
                    Icon::from_path(p, None).map_err(|e| format!("icon load failed: {e}"))?
                }
                None => return Err(format!("icon resource missing: {resource_err}")),
            }
        }
    };

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu.clone()))
        .with_icon(icon)
        .with_tooltip("ScrollSound V1.24")
        // C++ shows the menu on right-click only; muda's default also pops on
        // left click, so disable that explicitly.
        .with_menu_on_left_click(false)
        .build()
        .map_err(|e| format!("tray build failed: {e}"))?;

    APP.with(|cell| {
        *cell.borrow_mut() = Some(App {
            tray,
            menu,
            pause_item,
            autostart_item,
            admin_item,
        });
    });
    Ok(())
}

const APP_ICON_ORDINAL: u16 = 156;
/// 退出 item icon. Must stay above `APP_ICON_ORDINAL`: Explorer shows the
/// lowest RT_GROUP_ICON as the executable's file icon.
const EXIT_ICON_ORDINAL: u16 = 157;

/// Loads a menu-item icon from the executable's embedded resources. A missing
/// resource leaves the item icon-less instead of failing tray init (the
/// original C++ code had the same "menu still works" behavior).
fn menu_icon(ordinal: u16) -> Option<tray_icon::menu::Icon> {
    match tray_icon::menu::Icon::from_resource(ordinal, None) {
        Ok(icon) => Some(icon),
        Err(e) => {
            tracing::warn!("menu icon resource {ordinal} failed: {e}");
            None
        }
    }
}

/// Drains both channels (design D2: `try_recv`, never blocking the pump).
pub fn drain_events() {
    while TrayIconEvent::receiver().try_recv().is_ok() {}
    handle_menu_events();
}

fn handle_menu_events() {
    // Returns true when 退出 was chosen (pump should stop).
    let mut quit = false;
    while let Ok(event) = MenuEvent::receiver().try_recv() {
        if dispatch(event.id()) {
            quit = true;
        }
    }
    if quit {
        request_exit();
    }
}

/// Dispatches one menu event by its string id; returns true for 退出.
fn dispatch(id: &MenuId) -> bool {
    match id.as_ref() {
        "exit" => return true,
        "autostart" => toggle_autostart(),
        "admin" => toggle_admin(),
        "pause" => toggle_pause(),
        "rehook" => rehook_action(),
        "about" => about_action(),
        _ => {}
    }
    false
}

/// Focused access to the app state from the pump thread.
fn with_app<R>(f: impl FnOnce(&App) -> R) -> Option<R> {
    APP.with(|cell| cell.borrow().as_ref().map(f))
}

/// 开机自启(&R): toggle the Run value and refresh the check state.
fn toggle_autostart() {
    let Some(exe) = platform::win_settings::exe_path() else {
        tracing::error!("cannot resolve exe path for autostart");
        return;
    };
    let enabled = match platform::win_autostart::is_enabled(&exe) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("autostart query failed: {e}");
            return;
        }
    };
    let result = if enabled {
        platform::win_autostart::disable()
    } else {
        platform::win_autostart::enable(&exe)
    };
    match result {
        Ok(()) => {
            let now_enabled = !enabled;
            with_app(|app| app.autostart_item.set_checked(now_enabled));
            tracing::info!(
                "autostart {}",
                if now_enabled { "enabled" } else { "disabled" }
            );
        }
        Err(e) => tracing::error!("autostart toggle failed: {e}"),
    }
}

/// 管理员权限运行(&A): toggle the ini flag. Enabling while not elevated
/// relaunches elevated and exits; disabling while elevated relaunches
/// unelevated (privilege downgrade) and exits.
fn toggle_admin() {
    let enabled = platform::win_settings::admin_enabled();
    let target = !enabled;
    if let Err(e) = platform::win_settings::set_admin(target) {
        tracing::error!("admin flag write failed: {e}");
        return;
    }
    with_app(|app| app.admin_item.set_checked(target));
    tracing::info!("admin mode {}", if target { "enabled" } else { "disabled" });
    if target {
        if !platform::elevate::is_admin() {
            if platform::elevate::relaunch_elevated() {
                request_exit();
            } else {
                // UAC declined: revert the flag so the app keeps running
                // unelevated instead of prompting again at every start.
                tracing::error!("elevated relaunch declined — reverting admin flag");
                if let Err(e) = platform::win_settings::set_admin(false) {
                    tracing::error!("admin flag revert failed: {e}");
                }
                with_app(|app| app.admin_item.set_checked(false));
            }
        }
        return;
    }
    // Disabling while still elevated: relaunch with a standard-user token so
    // the downgrade takes effect immediately. On failure keep running — the
    // flag is already off and a manual restart applies it then.
    if platform::elevate::is_admin() {
        if platform::elevate::relaunch_unelevated() {
            request_exit();
        } else {
            let text = crate::platform::wide_cstr("降权失败：请退出后以普通权限重新运行");
            let caption = crate::platform::wide_cstr("ScrollSound");
            unsafe {
                MessageBoxW(
                    None,
                    PCWSTR(text.as_ptr()),
                    PCWSTR(caption.as_ptr()),
                    MB_ICONERROR,
                );
            }
        }
    }
}

/// 暂停/继续: unhook/rehook and flip the label (C++ ID_PAUSE semantics).
fn toggle_pause() {
    if hook::RUNNING.load(std::sync::atomic::Ordering::Relaxed) {
        hook::RUNNING.store(false, std::sync::atomic::Ordering::Relaxed);
        hook::move_mouse_hook();
        with_app(|app| app.pause_item.set_text("继续"));
        tracing::info!("paused");
    } else {
        hook::RUNNING.store(true, std::sync::atomic::Ordering::Relaxed);
        hook::set_mouse_hook(0);
        with_app(|app| app.pause_item.set_text("暂停"));
        tracing::info!("resumed");
    }
}

/// 重新HOOK: re-install and return to the running state. Silent on success;
fn rehook_action() {
    let ok = hook::re_hook();
    hook::RUNNING.store(true, std::sync::atomic::Ordering::Relaxed);
    with_app(|app| app.pause_item.set_text("暂停"));
    if ok {
        tracing::info!("re-hook succeeded");
    } else {
        tracing::error!("re-hook failed");
        let text = crate::platform::wide_cstr("HOOK 失败");
        let caption = crate::platform::wide_cstr("REHOOK");
        unsafe {
            MessageBoxW(
                None,
                PCWSTR(text.as_ptr()),
                PCWSTR(caption.as_ptr()),
                MB_ICONERROR,
            );
        }
    }
}

/// 关于(&G): open the project page in the default browser.
fn about_action() {
    let url = crate::platform::wide_cstr("https://github.com/SWDaby/ScrollSound");
    let op = crate::platform::wide_cstr("open");
    let rc = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(op.as_ptr()),
            PCWSTR(url.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            windows::Win32::UI::WindowsAndMessaging::SW_SHOW,
        )
    };
    // HINSTANCE return: values <= 32 are SE_ERR_* failures.
    if rc.0 as isize <= 32 {
        tracing::error!("ShellExecuteW(open url) failed: {}", rc.0 as isize);
    }
}

/// 退出(&X): unhook, drop the tray icon, and stop the pump.
fn request_exit() {
    hook::move_mouse_hook();
    APP.with(|cell| *cell.borrow_mut() = None);
    unsafe {
        windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
    }
}
