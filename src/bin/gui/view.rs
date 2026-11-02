impl ProxyApp {
    fn routes_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(t!("intro").to_string());
        self.add_route_buttons(ui);
        ui.separator();
        let mut changed = self.route_cards(ui);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(t!("local_key").to_string());
            changed |= ui
                .add(egui::TextEdit::singleline(&mut self.master_key).password(true))
                .changed();
        });
        ui.small(t!("key_hint").to_string());
        if changed {
            self.sync_forms();
        }
    }

    fn add_route_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button(t!("add_openai").to_string()).clicked() {
                self.routes.push(RouteDraft {
                    alias: "my-openai".into(),
                    model: "openai/model-name".into(),
                    api_base: "https://proxy.example.com/v1".into(),
                    ..Default::default()
                });
                self.sync_forms();
            }
            if ui.button(t!("add_claude").to_string()).clicked() {
                self.routes.push(RouteDraft {
                    alias: "my-claude".into(),
                    model: "anthropic/model-name".into(),
                    api_base: "https://proxy.example.com".into(),
                    ..Default::default()
                });
                self.sync_forms();
            }
        });
    }

    fn route_cards(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        let mut remove = None;
        for (index, route) in self.routes.iter_mut().enumerate() {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.strong(t!("route_n", n = index + 1).to_string());
                    if ui.button(t!("delete").to_string()).clicked() {
                        remove = Some(index);
                    }
                });
                egui::Grid::new(("route", index))
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label(t!("local_model").to_string());
                        changed |= ui.text_edit_singleline(&mut route.alias).changed();
                        ui.end_row();
                        ui.label(t!("upstream_model").to_string());
                        changed |= ui.text_edit_singleline(&mut route.model).changed();
                        ui.end_row();
                        ui.label(t!("api_base").to_string());
                        changed |= ui.text_edit_singleline(&mut route.api_base).changed();
                        ui.end_row();
                        ui.label(t!("upstream_key").to_string());
                        changed |= ui
                            .add(egui::TextEdit::singleline(&mut route.api_key).password(true))
                            .changed();
                        ui.end_row();
                    });
            });
            ui.add_space(6.0);
        }
        if let Some(index) = remove {
            self.routes.remove(index);
            changed = true;
        }
        changed
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading(t!("title").to_string());
        self.language_ui(ui);
        ui.horizontal(|ui| {
            ui.label(t!("config_file").to_string());
            ui.add_sized(
                [350.0, 22.0],
                egui::TextEdit::singleline(&mut self.config_path),
            );
            if ui.button(t!("load").to_string()).clicked() {
                self.reload();
            }
        });
        ui.horizontal(|ui| {
            ui.label(t!("local_port").to_string());
            ui.add_sized([85.0, 22.0], egui::TextEdit::singleline(&mut self.port));
            ui.label(t!("listen_local").to_string());
            if ui.button(t!("save_yaml").to_string()).clicked() {
                if let Err(error) = self.save_config() {
                    self.message = t!("save_failed", error = error).to_string();
                }
            }
            if self.status == ServerStatus::Stopped && ui.button(t!("start_proxy").to_string()).clicked() {
                self.start();
            }
            if matches!(self.status, ServerStatus::Running(_) | ServerStatus::Starting)
                && ui.button(t!("stop_proxy").to_string()).clicked()
            {
                self.stop();
            }
        });
        ui.label(&self.message);
        if self.dirty {
            ui.small(t!("unsaved").to_string());
        }
    }

    fn language_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(t!("language").to_string());
            let current = LANGUAGE_CHOICES.iter()
                .find(|(id, _)| *id == self.locale.requested)
                .map(|(_, label)| *label).unwrap_or("System / 系统");
            let mut changed = false;
            egui::ComboBox::from_id_salt("language")
                .selected_text(current)
                .show_ui(ui, |ui| {
                    for (id, label) in LANGUAGE_CHOICES {
                        changed |= ui.selectable_value(&mut self.locale.requested, (*id).into(), *label).changed();
                    }
                });
            if changed { self.locale.apply(); self.persist_settings(); }
        });
    }

    fn endpoints_ui(&self, ui: &mut egui::Ui) {
        if let ServerStatus::Running(addr) = self.status {
            ui.horizontal(|ui| {
                ui.monospace(format!("Responses: http://{addr}/v1/responses"));
                if ui.button(t!("copy").to_string()).clicked() {
                    ui.ctx().copy_text(format!("http://{addr}/v1/responses"));
                }
            });
            ui.horizontal(|ui| {
                ui.monospace(format!("Claude Code: http://{addr}/v1/messages"));
                if ui.button(t!("copy").to_string()).clicked() {
                    ui.ctx().copy_text(format!("http://{addr}"));
                }
            });
        }
    }

    fn yaml_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button(t!("load_forms").to_string()).clicked() {
                self.parse_forms();
            }
            ui.small(t!("yaml_hint").to_string());
        });
        if ui
            .add(
                egui::TextEdit::multiline(&mut self.yaml)
                    .code_editor()
                    .desired_rows(30)
                    .desired_width(f32::INFINITY),
            )
            .changed()
        {
            self.dirty = true;
        }
    }
}

impl eframe::App for ProxyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_events();
        egui::CentralPanel::default().show(ctx, |ui| {
            self.settings_ui(ui);
            self.endpoints_ui(ui);
            ui.separator();
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Routes, t!("routes").to_string());
                ui.selectable_value(&mut self.tab, Tab::Yaml, t!("full_yaml").to_string());
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| match self.tab {
                Tab::Routes => self.routes_ui(ui),
                Tab::Yaml => self.yaml_ui(ui),
            });
        });
        if self.status != ServerStatus::Stopped {
            ctx.request_repaint_after(Duration::from_millis(200));
        }
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        self.persist_settings();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.stop();
    }
}
