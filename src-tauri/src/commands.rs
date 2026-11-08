use crate::config::{AppConfig, ProxyConfig};
use crate::engine::AppEngine;
use crate::privilege::{is_admin, restart_as_admin};
use crate::process::{ProcessInfo, ProcessManager};
use crate::proxy::ProxyClient;
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::menu::MenuItem;
use tauri::{AppHandle, Manager, State};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyStatus {
    pub is_running: bool,
    pub is_admin: bool,
}

#[tauri::command]
pub fn check_admin_status() -> bool {
    is_admin()
}

#[tauri::command]
pub fn trigger_restart_as_admin(app: AppHandle) -> Result<(), String> {
    tauri_plugin_single_instance::destroy(&app);
    restart_as_admin(None)
}

#[tauri::command]
pub fn get_config(engine: State<'_, Arc<AppEngine>>) -> AppConfig {
    engine.current_config.read().clone()
}

#[tauri::command]
pub fn save_config(engine: State<'_, Arc<AppEngine>>, config: AppConfig) -> Result<(), String> {
    config.save()?;
    engine.update_config(config);
    Ok(())
}

#[tauri::command]
pub async fn start_proxy(app: AppHandle, engine: State<'_, Arc<AppEngine>>) -> Result<(), String> {
    if !is_admin() {
        // Explicitly release single instance lock before relaunching so elevated process doesn't get blocked
        tauri_plugin_single_instance::destroy(&app);
        return restart_as_admin(Some("--start-proxy"));
    }
    let res = engine.start().await;
    if res.is_ok() {
        update_tray_ui(&app, true);
    }
    res
}

#[tauri::command]
pub fn stop_proxy(app: AppHandle, engine: State<'_, Arc<AppEngine>>) -> Result<(), String> {
    engine.stop();
    update_tray_ui(&app, false);
    Ok(())
}

fn update_tray_ui(app: &AppHandle, running: bool) {
    if let Some(toggle_item) = app.try_state::<MenuItem<tauri::Wry>>() {
        let label = if running { "停止代理服务" } else { "启动代理服务" };
        let _ = toggle_item.set_text(label);
    }
    if let Some(tray) = app.tray_by_id("main_tray") {
        let tip = if running {
            "Marga - 代理服务运行中 (已接管)"
        } else {
            "Marga - 代理已停止"
        };
        let _ = tray.set_tooltip(Some(tip));
    }
}

#[tauri::command]
pub fn get_proxy_status(engine: State<'_, Arc<AppEngine>>) -> ProxyStatus {
    ProxyStatus {
        is_running: engine.is_running.load(Ordering::SeqCst),
        is_admin: is_admin(),
    }
}

#[tauri::command]
pub fn list_running_processes() -> Vec<ProcessInfo> {
    ProcessManager::list_running_processes()
}

#[tauri::command]
pub async fn test_proxy_connection(proxy: ProxyConfig) -> Result<String, String> {
    ProxyClient::test_proxy(&proxy).await
}
