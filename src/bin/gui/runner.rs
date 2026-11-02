fn main() -> eframe::Result {
    let _ = dotenvy::dotenv();
    let mut viewport = egui::ViewportBuilder::default().with_inner_size([900.0, 720.0]);
    if let Some(icon) = app_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "BibiLLMProxy",
        options,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);
            Ok(Box::new(ProxyApp::new(cc)))
        }),
    )
}

fn app_icon() -> Option<std::sync::Arc<egui::IconData>> {
    let png = include_bytes!("../../../assets/bibillmproxy-icon.png");
    let rgba = image::load_from_memory(png).ok()?.to_rgba8();
    let (width, height) = rgba.dimensions();
    Some(std::sync::Arc::new(egui::IconData { rgba: rgba.into_raw(), width, height }))
}

fn configure_fonts(ctx: &egui::Context) {
    let paths: &[&str] = if cfg!(target_os = "windows") {
        &["C:/Windows/Fonts/simhei.ttf", "C:/Windows/Fonts/meiryo.ttc", "C:/Windows/Fonts/malgun.ttf"]
    } else if cfg!(target_os = "macos") {
        &["/System/Library/Fonts/PingFang.ttc", "/System/Library/Fonts/Hiragino Sans GB.ttc", "/System/Library/Fonts/AppleSDGothicNeo.ttc"]
    } else {
        &["/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc"]
    };
    let mut fonts = egui::FontDefinitions::default();
    let mut loaded = false;
    for (index, path) in paths.iter().enumerate() {
        if let Ok(bytes) = fs::read(path) {
            let name = format!("system-cjk-{index}");
            fonts.font_data.insert(name.clone(), egui::FontData::from_owned(bytes).into());
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts.families.entry(family).or_default().push(name.clone());
            }
            loaded = true;
        }
    }
    if loaded { ctx.set_fonts(fonts); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_editor_keeps_other_yaml_sections() {
        let raw = "model_list: []\nmcp_servers:\n  docs:\n    url: https://example.com/mcp\n";
        let route = RouteDraft {
            alias: "codex".into(),
            model: "openai/gpt-5".into(),
            api_key: "os.environ/KEY".into(),
            ..Default::default()
        };
        let result = write_routes(raw, &[route], "local-key").unwrap();
        assert!(result.contains("mcp_servers:"));
        assert!(result.contains("os.environ/KEY"));
        assert_eq!(read_routes(&result).unwrap().0.len(), 1);
    }

    #[test]
    fn catalog_contains_requested_languages() {
        assert!(t!("title", locale = "zh-CN").contains("代理"));
        assert!(t!("title", locale = "ja").contains("プロキシ"));
        assert!(t!("title", locale = "ko").contains("프록시"));
        assert!(t!("title", locale = "ru").contains("прокси"));
        assert!(t!("title", locale = "fr").contains("Console"));
        assert!(t!("title", locale = "es").contains("Consola"));
        assert!(t!("title", locale = "pt").contains("Console"));
        assert!(t!("title", locale = "it").contains("Console"));
        assert!(t!("title", locale = "ro").contains("Consolă"));
    }
}
