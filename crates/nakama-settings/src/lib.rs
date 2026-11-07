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

use std::fs;
use std::path::{Path, PathBuf};

pub use error::SettingsError;

use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "config.ini";

/// Appearance and the `config.ini` path inside the app config folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: Theme,
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }

    fn parse(value: &str) -> Result<Self, SettingsError> {
        match value {
            "light" => Ok(Theme::Light),
            "dark" => Ok(Theme::Dark),
            _ => Err(SettingsError::Theme(value.to_owned())),
        }
    }
}

/// Read `config.ini` from `dir`, creating a light-theme file when it is missing.
pub fn load(dir: &Path) -> Result<Settings, SettingsError> {
    let path = config_path(dir);
    if !path.exists() {
        return save(dir, Theme::Light);
    }
    let text = fs::read_to_string(&path).map_err(SettingsError::Read)?;
    Ok(Settings {
        theme: parse_theme(&text)?,
        path: path.display().to_string(),
    })
}

/// Write `config.ini` in `dir`, creating the folder when it is missing.
pub fn save(dir: &Path, theme: Theme) -> Result<Settings, SettingsError> {
    fs::create_dir_all(dir).map_err(SettingsError::CreateFolder)?;
    let path = config_path(dir);
    let body = format!("[appearance]\ntheme = {}\n", theme.as_str());
    fs::write(&path, body).map_err(SettingsError::Write)?;
    Ok(Settings {
        theme,
        path: path.display().to_string(),
    })
}

fn config_path(dir: &Path) -> PathBuf {
    dir.join(FILE_NAME)
}

fn parse_theme(text: &str) -> Result<Theme, SettingsError> {
    let mut section = String::new();
    let mut theme = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            section = name.trim().to_owned();
            continue;
        }
        if section != "appearance" {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != "theme" {
            continue;
        }
        theme = Some(Theme::parse(value.trim())?);
    }
    Ok(theme.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::{Theme, load, parse_theme, save};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let id = NEXT.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("nakama-settings-{name}-{}-{id}", process::id()));
            let _ = fs::remove_dir_all(&path);
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn creates_config_ini_when_missing() {
        let dir = TempDir::new("create");
        let settings = load(dir.path()).expect("load");
        assert_eq!(settings.theme, Theme::Light);
        assert!(dir.path().join("config.ini").is_file());
        assert_eq!(
            fs::read_to_string(dir.path().join("config.ini")).expect("read"),
            "[appearance]\ntheme = light\n"
        );
    }

    #[test]
    fn round_trips_dark_theme() {
        let dir = TempDir::new("round-trip");
        let saved = save(dir.path(), Theme::Dark).expect("save");
        assert_eq!(saved.theme, Theme::Dark);
        let loaded = load(dir.path()).expect("load");
        assert_eq!(loaded.theme, Theme::Dark);
        assert_eq!(loaded.path, saved.path);
    }

    #[test]
    fn rejects_unknown_theme() {
        let dir = TempDir::new("invalid");
        fs::create_dir_all(dir.path()).expect("dir");
        let path = dir.path().join("config.ini");
        fs::write(&path, "[appearance]\ntheme = blue\n").expect("write");
        let error = load(dir.path()).expect_err("invalid theme");
        assert_eq!(
            error.to_string(),
            "theme in config.ini must be light or dark, found blue"
        );
        assert_eq!(
            fs::read_to_string(path).expect("unchanged"),
            "[appearance]\ntheme = blue\n"
        );
    }

    #[test]
    fn defaults_when_theme_is_absent() {
        let theme = parse_theme("[other]\ntheme = dark\n\n[appearance]\n; note\n").expect("parse");
        assert_eq!(theme, Theme::Light);
    }
}
