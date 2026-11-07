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

use nakama_device_info::DeviceInfo;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![device_info])
        .run(tauri::generate_context!())
}
