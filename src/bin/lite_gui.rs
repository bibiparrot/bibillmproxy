#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

rust_i18n::i18n!("locales", fallback = "en");

include!("gui/config.rs");
include!("gui/settings.rs");
include!("gui/app.rs");
include!("gui/view.rs");
include!("gui/runner.rs");
