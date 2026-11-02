fn main() -> eframe::Result {
    let _ = dotenvy::dotenv();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "LiteLLM Rust Proxy",
        options,
        Box::new(|cc| {
            if let Ok(bytes) = fs::read("C:/Windows/Fonts/simhei.ttf") {
                let mut fonts = egui::FontDefinitions::default();
                fonts
                    .font_data
                    .insert("chinese".into(), egui::FontData::from_owned(bytes).into());
                if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                    family.insert(0, "chinese".into());
                }
                if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                    family.push("chinese".into());
                }
                cc.egui_ctx.set_fonts(fonts);
            }
            Ok(Box::new(ProxyApp::new(cc)))
        }),
    )
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
}
