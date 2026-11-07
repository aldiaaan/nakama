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

mod error;

use std::path::PathBuf;

use nakama_device_info::DeviceInfo;
use nakama_settings::{Settings, Theme};
use tauri::Manager;

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

fn app_config_dir(app: &tauri::AppHandle) -> Result<PathBuf, CommandError> {
    app.path()
        .app_config_dir()
        .map_err(|error| CommandError::Internal(error.to_string()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            device_info,
            load_settings,
            save_settings
        ])
        .run(tauri::generate_context!())
}
