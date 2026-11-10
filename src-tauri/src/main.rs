#![windows_subsystem = "windows"]

mod commands;
mod config;
mod engine;
mod firewall;
mod interceptor;
mod privilege;
mod process;
mod proxy;
mod relay;
mod sni;

use commands::*;
use engine::AppEngine;
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

fn main() {
    let engine = Arc::new(AppEngine::new(None));
    let engine_exit_clone = engine.clone();
    let engine_setup_clone = engine.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // When a second instance is launched, bring the existing window to focus
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .manage(engine)
        .setup(move |app| {
            // Connection logs are pushed to the frontend through the app handle
            engine_setup_clone.relay.set_app_handle(app.app_handle().clone());

            let show_i = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let toggle_proxy_i = MenuItem::with_id(app, "toggle_proxy", "启动代理服务", true, None::<&str>)?;
            let sep_i = PredefinedMenuItem::separator(app)?;
            let quit_i = MenuItem::with_id(app, "quit", "退出 Marga", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &toggle_proxy_i, &sep_i, &quit_i])?;

            let engine_tray_clone = engine_setup_clone.clone();
            let toggle_item_clone = toggle_proxy_i.clone();

            let _tray = TrayIconBuilder::with_id("main_tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Marga - 代理未启动")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event({
                    let toggle_item = toggle_item_clone.clone();
                    move |app, event| match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                        "toggle_proxy" => {
                            let engine = engine_tray_clone.clone();
                            let app_handle = app.clone();
                            let toggle_item = toggle_item.clone();
                            tauri::async_runtime::spawn(async move {
                                if engine.is_running.load(std::sync::atomic::Ordering::SeqCst) {
                                    engine.stop();
                                } else {
                                    if privilege::is_admin() {
                                        let _ = engine.start().await;
                                    } else {
                                        tauri_plugin_single_instance::destroy(&app_handle);
                                        let _ = privilege::restart_as_admin(Some("--start-proxy"));
                                        return;
                                    }
                                }
                                let running = engine.is_running.load(std::sync::atomic::Ordering::SeqCst);
                                let label = if running { "停止代理服务" } else { "启动代理服务" };
                                let _ = toggle_item.set_text(label);
                                if let Some(tray) = app_handle.tray_by_id("main_tray") {
                                    let tip = if running {
                                        "Marga - 代理服务运行中"
                                    } else {
                                        "Marga - 代理已停止"
                                    };
                                    let _ = tray.set_tooltip(Some(tip));
                                }

                                use tauri::Emitter;
                                let _ = app_handle.emit("proxy-status-changed", ());
                            });
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    }
                    | TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            app.manage(toggle_item_clone.clone());

            // Check if relaunched with --start-proxy
            let args: Vec<String> = std::env::args().collect();
            if args.iter().any(|arg| arg == "--start-proxy") && privilege::is_admin() {
                // Ensure the window is shown and focused when elevated
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }

                let engine_start = engine_setup_clone.clone();
                let app_handle = app.app_handle().clone();
                let toggle_item = toggle_item_clone;
                tauri::async_runtime::spawn(async move {
                    if let Ok(()) = engine_start.start().await {
                        let _ = toggle_item.set_text("停止代理服务");
                        if let Some(tray) = app_handle.tray_by_id("main_tray") {
                            let _ = tray.set_tooltip(Some("Marga - 代理服务运行中"));
                        }
                        use tauri::Emitter;
                        let _ = app_handle.emit("proxy-status-changed", ());
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_admin_status,
            trigger_restart_as_admin,
            get_config,
            save_config,
            start_proxy,
            stop_proxy,
            get_proxy_status,
            list_running_processes,
            test_proxy_connection,
        ])
        .on_window_event(move |window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                // Prevent window from closing, hide it to system tray instead
                api.prevent_close();
                let _ = window.hide();
            }
            tauri::WindowEvent::Resized(_) => {
                if let Ok(is_minimized) = window.is_minimized() {
                    if is_minimized {
                        let _ = window.hide();
                    }
                }
            }
            tauri::WindowEvent::Destroyed => {
                // Ensure interceptor handle and sockets are cleanly dropped on close
                engine_exit_clone.stop();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
