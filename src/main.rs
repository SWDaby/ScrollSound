//! Entry point, elevation gate, message pump, and wiring (design D2/D7/D9).
//!
//! The main thread owns everything: tray/menu channels are drained after each
//! dispatched message, the WH_MOUSE_LL callback runs here (its thread must
//! pump messages), and the windowless watchdog timer feeds the WM_TIMER
//! branch. `#![windows_subsystem = "windows"]` keeps the app windowless.

#![cfg_attr(not(test), windows_subsystem = "windows")]

mod core;
mod platform;
#[cfg(windows)]
mod taskbar_menu;

#[cfg(windows)]
use std::fs::File;
#[cfg(windows)]
use std::path::PathBuf;
#[cfg(windows)]
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
#[cfg(windows)]
use std::time::Duration;

#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, KillTimer, SetTimer, TranslateMessage, MSG, WM_TIMER,
};

use platform::hook;

/// Watchdog re-hook count ceiling (C++ `MAX_HOOKTIME`).
#[cfg(windows)]
const MAX_HOOKTIME: u32 = 10;
/// Windowless timer cadence: re-hook attempt every 60 s.
#[cfg(windows)]
const WATCHDOG_MS: u32 = 60_000;

#[cfg(windows)]
static WATCHDOG_ACTIVE: AtomicBool = AtomicBool::new(true);
/// Timer id returned by SetTimer (thread timers ignore nIDEvent and generate
/// their own id; the WM_TIMER wParam carries it).
#[cfg(windows)]
static WATCHDOG_TIMER: AtomicUsize = AtomicUsize::new(0);
#[cfg(windows)]
static HOOK_TICKS: AtomicU32 = AtomicU32::new(0);

fn main() {
    #[cfg(windows)]
    win_main();
    #[cfg(not(windows))]
    {
        eprintln!("ScrollSound is Windows-only");
        std::process::exit(1);
    }
}

#[cfg(windows)]
fn win_main() {
    let debug = std::env::args().any(|a| a == "--debug");
    if debug {
        init_logging();
    }
    tracing::info!("ScrollSound V1.24 starting");

    // Single-instance lock. A relaunch child (`--elevated-relaunch`) briefly
    // overlaps its exiting parent, so it waits for the mutex handoff; a plain
    // duplicate launch exits immediately.
    let relaunching = std::env::args().any(|a| a == platform::single_instance::RELAUNCH_ARG);
    let _guard = if relaunching {
        platform::single_instance::acquire_wait(Duration::from_secs(6))
    } else {
        platform::single_instance::acquire_now()
    };
    let Some(_guard) = _guard else {
        tracing::info!("another instance is running; exiting");
        return;
    };

    // Adopt a legacy C++ setting.ini (CWD-relative) before reading the flag.
    platform::win_settings::migrate_legacy_setting();

    // Elevation gate (D7): admin flag set but not elevated → runas relaunch
    // before any UI. A cancellation or failed relaunch (UAC declined, or the
    // launched child dies at startup) must not leave the app unstartable:
    // revert the flag and run unelevated instead of prompting at every start.
    if platform::win_settings::admin_enabled() && !platform::elevate::is_admin() {
        if platform::elevate::relaunch_elevated() {
            tracing::info!("admin mode enabled and not elevated: relaunching");
            return;
        }
        tracing::error!("elevated relaunch failed or cancelled — reverting admin flag");
        if let Err(e) = platform::win_settings::set_admin(false) {
            tracing::error!("admin flag revert failed: {e}");
        }
    }

    // Version → taskbar class init (D2: filled once at startup).
    hook::init_taskbar_class();
    let class_name = hook::TASKBAR_CLASS
        .get()
        .map(|classes| {
            classes
                .iter()
                .map(|c| String::from_utf16_lossy(c))
                .collect::<Vec<_>>()
                .join("/")
        })
        .unwrap_or_else(|| "<none>".into());
    tracing::info!("taskbar class: {class_name}");

    // Tray + menu on the pump thread, before the hook.
    let exe = platform::win_settings::exe_path();
    let autostart = exe
        .as_deref()
        .and_then(|p| platform::win_autostart::is_enabled(p).ok())
        .unwrap_or(false);
    let admin = platform::win_settings::admin_enabled();
    if let Err(e) = taskbar_menu::init(autostart, admin) {
        // C++ InitInstance failure exits silently; log only.
        tracing::error!("tray initialization failed: {e}");
        return;
    }

    // Hook, then the windowless watchdog timer. Windows ignores nIDEvent for
    // thread timers and generates the id itself (the return value; the
    // WM_TIMER wParam carries it) — match by that returned id.
    hook::set_mouse_hook(0);
    let timer_id = unsafe { SetTimer(None, 0, WATCHDOG_MS, None) };
    WATCHDOG_TIMER.store(timer_id, Ordering::SeqCst);
    if timer_id == 0 {
        tracing::error!("SetTimer failed — watchdog disabled");
    }

    // Message pump: WH_MOUSE_LL callbacks and tray-icon's internal window are
    // both serviced here. Only OUR windowless WM_TIMER (hwnd NULL and the
    // returned timer id) feeds the watchdog — tray-icon/muda own other timers
    // (e.g. a 15 ms mouse-leave timer) that must be dispatched to their
    // windows, never mistaken for the watchdog (which would tear the hook
    // down at 60+ Hz).
    let mut msg = MSG::default();
    loop {
        let result = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        if result.0 <= 0 {
            break; // 0 → WM_QUIT (退出); negative → GetMessage error
        }
        if msg.message == WM_TIMER
            && msg.hwnd.0.is_null()
            && msg.wParam.0 as usize == WATCHDOG_TIMER.load(Ordering::SeqCst)
        {
            watchdog_tick();
            continue;
        }
        unsafe {
            let _ = TranslateMessage(&msg);
            let _ = DispatchMessageW(&msg);
        }
        // Drain tray/menu channels without blocking (D2).
        taskbar_menu::drain_events();
    }

    // Exit path: the 退出 handler already un-hooked + dropped the tray; this
    // is the idempotent backstop after the pump ends.
    hook::move_mouse_hook();
    tracing::info!("exiting");
}

/// Watchdog tick (design D5): re-hook while active, count every tick, stop at
/// MAX_HOOKTIME — C++ TimerProc parity. The stop flag makes further stray
/// ticks (reentrancy, a late duplicate) harmless even if KillTimer failed.
#[cfg(windows)]
fn watchdog_tick() {
    if !WATCHDOG_ACTIVE.load(Ordering::Relaxed) {
        return;
    }
    let count = HOOK_TICKS.fetch_add(1, Ordering::SeqCst) + 1;
    if count >= MAX_HOOKTIME {
        WATCHDOG_ACTIVE.store(false, Ordering::Relaxed);
        let id = WATCHDOG_TIMER.load(Ordering::SeqCst);
        if id != 0 {
            unsafe {
                let _ = KillTimer(None, id);
            }
        }
        tracing::debug!("watchdog stopped after {count} ticks");
    }
    if hook::RUNNING.load(Ordering::Relaxed) {
        hook::re_hook();
        tracing::debug!("watchdog re-hook #{count}");
    } else {
        tracing::debug!("watchdog tick #{count} (paused, skipped)");
    }
}

/// `--debug` file logging to `scrollsound.log` next to the exe
/// (design D9: otherwise fully silent like the C++).
#[cfg(windows)]
fn init_logging() {
    let path = platform::win_settings::exe_dir()
        .map(|d| d.join("scrollsound.log"))
        .unwrap_or_else(|| PathBuf::from("scrollsound.log"));
    match File::create(&path) {
        Ok(file) => {
            let _ = tracing_subscriber::fmt()
                .with_writer(file)
                .with_ansi(false)
                .with_max_level(tracing::Level::TRACE)
                .try_init();
            tracing::info!("debug logging: {}", path.display());
        }
        Err(e) => eprintln!("cannot create {}: {e}", path.display()),
    }
}
