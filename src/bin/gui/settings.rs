use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
struct AppSettings {
    config_path: String,
    port: u16,
    locale: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            config_path: app_config_dir().join("config.yaml").to_string_lossy().into_owned(),
            port: 4000,
            locale: "system".into(),
        }
    }
}

impl AppSettings {
    fn path() -> PathBuf {
        app_config_dir().join("settings.toml")
    }

    fn load() -> Result<Self, String> {
        let path = Self::path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(&path).map_err(|error| error.to_string())?;
        toml::from_str(&raw).map_err(|error| error.to_string())
    }

    fn save(&self) -> Result<(), String> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let raw = toml::to_string_pretty(self).map_err(|error| error.to_string())?;
        fs::write(path, raw).map_err(|error| error.to_string())
    }
}

fn app_config_dir() -> PathBuf {
    ProjectDirs::from("com", "bibiparrot", "BibiLLMProxy")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

struct LocaleManager {
    requested: String,
}

impl LocaleManager {
    fn new(requested: String) -> Self {
        let requested = if LANGUAGE_CHOICES.iter().any(|(id, _)| *id == requested) {
            requested
        } else {
            "system".into()
        };
        let manager = Self { requested };
        manager.apply();
        manager
    }

    fn apply(&self) {
        let system = sys_locale::get_locale().unwrap_or_else(|| "en".into());
        let locale = if self.requested == "system" { &system } else { &self.requested };
        rust_i18n::set_locale(resolve_locale(locale));
    }
}

fn resolve_locale(locale: &str) -> &'static str {
    match locale.split(['-', '_']).next().unwrap_or_default().to_ascii_lowercase().as_str() {
        "zh" => "zh-CN",
        "ja" => "ja",
        "ko" => "ko",
        "ru" => "ru",
        "fr" => "fr",
        "es" => "es",
        "pt" => "pt",
        "it" => "it",
        "ro" => "ro",
        _ => "en",
    }
}

const LANGUAGE_CHOICES: &[(&str, &str)] = &[
    ("system", "System / 系统"),
    ("zh-CN", "简体中文"),
    ("en", "English"),
    ("ja", "日本語"),
    ("ko", "한국어"),
    ("ru", "Русский"),
    ("fr", "Français"),
    ("es", "Español"),
    ("pt", "Português"),
    ("it", "Italiano"),
    ("ro", "Română"),
];

#[cfg(test)]
mod settings_tests {
    use super::*;

    #[test]
    fn system_locale_maps_to_supported_language() {
        assert_eq!(resolve_locale("zh-TW"), "zh-CN");
        assert_eq!(resolve_locale("pt_BR"), "pt");
        assert_eq!(resolve_locale("de-DE"), "en");
    }

    #[test]
    fn settings_round_trip_as_toml() {
        let settings = AppSettings { config_path: "config.yaml".into(), port: 4567, locale: "ja".into() };
        let raw = toml::to_string_pretty(&settings).unwrap();
        assert_eq!(toml::from_str::<AppSettings>(&raw).unwrap(), settings);
    }
}
