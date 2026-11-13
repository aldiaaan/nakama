// nakama
// Copyright (C) 2026  aldiaaan <62921668+aldiaaan@users.noreply.github.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

mod daemon;
mod error;

use std::path::PathBuf;

use daemon::{DaemonController, DaemonOverview, IpcExchange, ProfilerKind};
use error::NotificationError;
use nakama_daemon::{IpcEndpoint, IpcRequest};
use nakama_device_info::DeviceInfo;
use nakama_settings::{Settings, Theme};
use tauri::{Manager, RunEvent, State};
use tauri_plugin_notification::{NotificationExt, PermissionState};

pub use error::CommandError;

/// Read the host CPU, memory, and GPU, print the snapshot, and return it to the webview.
#[tauri::command]
async fn device_info() -> Result<DeviceInfo, CommandError> {
    let info = tauri::async_runtime::spawn_blocking(nakama_device_info::device_info)
        .await
        .map_err(|error| CommandError::Internal(error.to_string()))??;
    println!("{info}");
    Ok(info)
}

/// Read `config.ini` from the app config folder, creating it on first load.
#[tauri::command]
async fn load_settings(app: tauri::AppHandle) -> Result<Settings, CommandError> {
    let dir = app_config_dir(&app)?;
    let settings = tauri::async_runtime::spawn_blocking(move || nakama_settings::load(&dir))
        .await
        .map_err(|error| CommandError::Internal(error.to_string()))??;
    println!("settings: {} ({})", settings.theme.as_str(), settings.path);
    Ok(settings)
}

/// Write the appearance theme to `config.ini` in the app config folder.
#[tauri::command]
async fn save_settings(app: tauri::AppHandle, theme: Theme) -> Result<Settings, CommandError> {
    let dir = app_config_dir(&app)?;
    let settings = tauri::async_runtime::spawn_blocking(move || nakama_settings::save(&dir, theme))
        .await
        .map_err(|error| CommandError::Internal(error.to_string()))??;
    println!("settings: {} ({})", settings.theme.as_str(), settings.path);
    Ok(settings)
}

/// Everything the Daemon page shows: state, subsystems, metrics, an IPC ping, calculations, and profilers.
#[tauri::command]
async fn daemon_overview(
    daemon: State<'_, DaemonController>,
) -> Result<DaemonOverview, CommandError> {
    Ok(daemon.overview().await)
}

/// Start the example daemon again after it stopped or failed.
#[tauri::command]
async fn daemon_start(daemon: State<'_, DaemonController>) -> Result<(), CommandError> {
    Ok(daemon.start()?)
}

/// Send one request over the daemon's IPC channel, the way an external client would.
#[tauri::command]
async fn daemon_ipc_request(
    daemon: State<'_, DaemonController>,
    request: IpcRequest,
) -> Result<IpcExchange, CommandError> {
    Ok(daemon.send_ipc(request).await?)
}

#[tauri::command]
async fn daemon_start_profiler(
    daemon: State<'_, DaemonController>,
    kind: ProfilerKind,
) -> Result<(), CommandError> {
    Ok(daemon.start_profiler(kind)?)
}

/// Stop a profiler and write its profile into the app data folder.
#[tauri::command]
async fn daemon_stop_profiler(
    daemon: State<'_, DaemonController>,
    kind: ProfilerKind,
) -> Result<(), CommandError> {
    Ok(daemon.stop_profiler(kind).await?)
}

/// Show a notification in the operating system's notification center.
#[tauri::command]
fn push_notification(
    app: tauri::AppHandle,
    title: String,
    body: String,
) -> Result<(), CommandError> {
    show_notification(&app, &title, &body)?;
    Ok(())
}

fn show_notification(
    app: &tauri::AppHandle,
    title: &str,
    body: &str,
) -> Result<(), NotificationError> {
    let title = title.trim();
    let body = body.trim();
    if title.is_empty() {
        return Err(NotificationError::EmptyTitle);
    }
    if body.is_empty() {
        return Err(NotificationError::EmptyBody);
    }

    let notifications = app.notification();
    if notifications.permission_state()? != PermissionState::Granted
        && notifications.request_permission()? != PermissionState::Granted
    {
        return Err(NotificationError::PermissionDenied);
    }

    notifications.builder().title(title).body(body).show()?;
    Ok(())
}

fn app_config_dir(app: &tauri::AppHandle) -> Result<PathBuf, CommandError> {
    app.path()
        .app_config_dir()
        .map_err(|error| CommandError::Internal(error.to_string()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let daemon = DaemonController::new(
                IpcEndpoint::named(daemon::NAME),
                app.path().app_data_dir()?.join("profiles"),
            );
            daemon.start()?;
            app.manage(daemon);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            device_info,
            load_settings,
            save_settings,
            daemon_overview,
            daemon_start,
            daemon_ipc_request,
            daemon_start_profiler,
            daemon_stop_profiler,
            push_notification
        ])
        .build(tauri::generate_context!())?
        .run(|app, event| {
            if let RunEvent::Exit = event {
                app.state::<DaemonController>().stop_blocking();
            }
        });
    Ok(())
}
