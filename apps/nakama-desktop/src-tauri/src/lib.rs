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

mod daemons;
mod error;
mod profilers;

use std::path::PathBuf;

use daemons::{DaemonHost, DaemonOverview, Daemons, IpcExchange};
use error::NotificationError;
use nakama_daemon::IpcRequest;
use nakama_device_info::DeviceInfo;
use nakama_example_daemon::{ExampleDaemon, ExampleReport};
use nakama_settings::{Settings, Theme};
use profilers::{ProfilerKind, ProfilerReport, Profilers};
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

/// A daemon's state, subsystems, metrics, and an IPC ping.
#[tauri::command]
async fn daemon_overview(
    daemons: State<'_, Daemons>,
    name: String,
) -> Result<DaemonOverview, CommandError> {
    Ok(daemons.get(&name)?.overview().await)
}

/// Start a daemon again after it stopped or failed.
#[tauri::command]
async fn daemon_start(daemons: State<'_, Daemons>, name: String) -> Result<(), CommandError> {
    Ok(daemons.get(&name)?.start()?)
}

/// Send one request over a daemon's IPC channel, the way an external client would.
#[tauri::command]
async fn daemon_ipc_request(
    daemons: State<'_, Daemons>,
    name: String,
    request: IpcRequest,
) -> Result<IpcExchange, CommandError> {
    Ok(daemons.get(&name)?.send_ipc(request).await?)
}

/// The example daemon's job counts, latest π estimate, and recent calculations.
#[tauri::command]
fn example_daemon_report(example: State<'_, ExampleDaemon>) -> ExampleReport {
    example.report()
}

#[tauri::command]
fn profiler_list(profilers: State<'_, Profilers>) -> Vec<ProfilerReport> {
    profilers.reports()
}

#[tauri::command]
async fn profiler_start(
    profilers: State<'_, Profilers>,
    kind: ProfilerKind,
) -> Result<(), CommandError> {
    Ok(profilers.start(kind)?)
}

/// Stop a profiler and write its profile into the app data folder.
#[tauri::command]
async fn profiler_stop(
    profilers: State<'_, Profilers>,
    kind: ProfilerKind,
) -> Result<(), CommandError> {
    Ok(profilers.stop(kind).await?)
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
            let example = ExampleDaemon::new();
            let daemons = Daemons::new([DaemonHost::new(nakama_example_daemon::NAME, {
                let example = example.clone();
                move |config| example.builder(config)
            })])?;
            daemons.start_all()?;
            app.manage(daemons);
            app.manage(example);
            app.manage(Profilers::new(app.path().app_data_dir()?.join("profiles")));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            device_info,
            load_settings,
            save_settings,
            daemon_overview,
            daemon_start,
            daemon_ipc_request,
            example_daemon_report,
            profiler_list,
            profiler_start,
            profiler_stop,
            push_notification
        ])
        .build(tauri::generate_context!())?
        .run(|app, event| {
            if let RunEvent::Exit = event {
                app.state::<Daemons>().stop_all_blocking();
            }
        });
    Ok(())
}
