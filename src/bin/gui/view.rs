impl ProxyApp {
    fn routes_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("将 OpenAI Responses 或 Anthropic Messages 兼容代理接入本地端点。API Base 可填写代理根地址或以 /v1 结尾的地址。");
        self.add_route_buttons(ui);
        ui.separator();
        let mut changed = self.route_cards(ui);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("本地访问密钥");
            changed |= ui
                .add(egui::TextEdit::singleline(&mut self.master_key).password(true))
                .changed();
        });
        ui.small("Key 可填写 os.environ/变量名；启动前需在环境变量或 .env 中提供。YAML 文件会保存明文输入，请自行保护文件权限。");
        if changed {
            self.sync_forms();
        }
    }

    fn add_route_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("＋ OpenAI 代理").clicked() {
                self.routes.push(RouteDraft {
                    alias: "my-openai".into(),
                    model: "openai/model-name".into(),
                    api_base: "https://proxy.example.com/v1".into(),
                    ..Default::default()
                });
                self.sync_forms();
            }
            if ui.button("＋ Claude 代理").clicked() {
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
                    ui.strong(format!("路由 {}", index + 1));
                    if ui.button("删除").clicked() {
                        remove = Some(index);
                    }
                });
                egui::Grid::new(("route", index))
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("本地模型名");
                        changed |= ui.text_edit_singleline(&mut route.alias).changed();
                        ui.end_row();
                        ui.label("上游模型");
                        changed |= ui.text_edit_singleline(&mut route.model).changed();
                        ui.end_row();
                        ui.label("API Base");
                        changed |= ui.text_edit_singleline(&mut route.api_base).changed();
                        ui.end_row();
                        ui.label("上游 API Key");
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
        ui.heading("LiteLLM Rust · 代理控制台");
        ui.horizontal(|ui| {
            ui.label("配置文件");
            ui.add_sized(
                [350.0, 22.0],
                egui::TextEdit::singleline(&mut self.config_path),
            );
            if ui.button("读取").clicked() {
                self.reload();
            }
        });
        ui.horizontal(|ui| {
            ui.label("本地端口");
            ui.add_sized([85.0, 22.0], egui::TextEdit::singleline(&mut self.port));
            ui.label("监听 127.0.0.1");
            if ui.button("保存 YAML").clicked() {
                if let Err(error) = self.save_config() {
                    self.message = format!("保存失败: {error}");
                }
            }
            if self.status == ServerStatus::Stopped && ui.button("启动代理").clicked() {
                self.start();
            }
            if matches!(self.status, ServerStatus::Running(_) | ServerStatus::Starting)
                && ui.button("停止代理").clicked()
            {
                self.stop();
            }
        });
        ui.label(&self.message);
        if self.dirty {
            ui.small("有未保存的配置；运行中的代理需重启才会使用新配置。");
        }
    }

    fn endpoints_ui(&self, ui: &mut egui::Ui) {
        if let ServerStatus::Running(addr) = self.status {
            ui.horizontal(|ui| {
                ui.monospace(format!("Responses: http://{addr}/v1/responses"));
                if ui.button("复制").clicked() {
                    ui.ctx().copy_text(format!("http://{addr}/v1/responses"));
                }
            });
            ui.horizontal(|ui| {
                ui.monospace(format!("Claude Code: http://{addr}/v1/messages"));
                if ui.button("复制").clicked() {
                    ui.ctx().copy_text(format!("http://{addr}"));
                }
            });
        }
    }

    fn yaml_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("载入表单").clicked() {
                self.parse_forms();
            }
            ui.small("可编辑完整 LiteLLM YAML；编辑后点“载入表单”刷新路由视图。");
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
                ui.selectable_value(&mut self.tab, Tab::Routes, "代理路由");
                ui.selectable_value(&mut self.tab, Tab::Yaml, "完整 YAML");
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

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("config_path", self.config_path.clone());
        storage.set_string("port", self.port.clone());
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.stop();
    }
}
