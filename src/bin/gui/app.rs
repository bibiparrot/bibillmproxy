enum ServerEvent {
    Running(SocketAddr),
    Stopped(Result<(), String>),
}

#[derive(PartialEq)]
enum ServerStatus {
    Stopped,
    Starting,
    Running(SocketAddr),
    Stopping,
}

#[derive(PartialEq)]
enum Tab {
    Routes,
    Yaml,
}

struct ProxyApp {
    config_path: String,
    port: String,
    yaml: String,
    routes: Vec<RouteDraft>,
    master_key: String,
    tab: Tab,
    status: ServerStatus,
    message: String,
    dirty: bool,
    events_tx: Sender<ServerEvent>,
    events_rx: Receiver<ServerEvent>,
    shutdown: Option<oneshot::Sender<()>>,
}

impl ProxyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let storage = cc.storage;
        let config_path = storage
            .and_then(|s| s.get_string("config_path"))
            .unwrap_or_else(|| "config.yaml".into());
        let port = storage
            .and_then(|s| s.get_string("port"))
            .unwrap_or_else(|| "4000".into());
        let yaml = fs::read_to_string(&config_path).unwrap_or_else(|_| DEFAULT_YAML.into());
        let (routes, master_key) = read_routes(&yaml).unwrap_or_default();
        let (events_tx, events_rx) = mpsc::channel();
        Self {
            config_path,
            port,
            yaml,
            routes,
            master_key,
            tab: Tab::Routes,
            status: ServerStatus::Stopped,
            message: String::new(),
            dirty: false,
            events_tx,
            events_rx,
            shutdown: None,
        }
    }

    fn reload(&mut self) {
        match fs::read_to_string(&self.config_path) {
            Ok(yaml) => {
                self.yaml = yaml;
                self.parse_forms();
                self.dirty = false;
                self.message = "配置已重新读取".into();
            }
            Err(error) => self.message = format!("读取失败: {error}"),
        }
    }

    fn parse_forms(&mut self) {
        match read_routes(&self.yaml) {
            Ok((routes, master_key)) => {
                self.routes = routes;
                self.master_key = master_key;
                self.message = "YAML 已载入表单".into();
            }
            Err(error) => self.message = format!("YAML 解析失败: {error}"),
        }
    }

    fn sync_forms(&mut self) {
        match write_routes(&self.yaml, &self.routes, &self.master_key) {
            Ok(yaml) => {
                self.yaml = yaml;
                self.dirty = true;
            }
            Err(error) => self.message = format!("无法更新 YAML: {error}"),
        }
    }

    fn save_config(&mut self) -> Result<(), String> {
        let config = parse_config(&self.yaml).map_err(|error| error.to_string())?;
        let mut providers = ProviderRegistry::new();
        providers::register_all(&mut providers);
        Router::from_config(&config, &providers).map_err(|error| error.to_string())?;
        let path = PathBuf::from(&self.config_path);
        fs::write(path, &self.yaml).map_err(|error| error.to_string())?;
        self.dirty = false;
        self.message = "配置已保存".into();
        Ok(())
    }

    fn start(&mut self) {
        if let Err(error) = self.save_config() {
            self.message = format!("启动前配置检查失败: {error}");
            return;
        }
        let Ok(port) = self.port.parse::<u16>() else {
            self.message = "端口必须在 1–65535 之间".into();
            return;
        };
        if port == 0 {
            self.message = "端口必须在 1–65535 之间".into();
            return;
        }
        let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        let path = PathBuf::from(&self.config_path);
        let events = self.events_tx.clone();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        self.shutdown = Some(shutdown_tx);
        self.status = ServerStatus::Starting;
        self.message = "正在启动代理…".into();
        thread::spawn(move || {
            let result = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .map_err(|error| error.to_string())
                .and_then(|runtime| {
                    runtime
                        .block_on(gateway_server::serve(
                            &path,
                            addr,
                            async move {
                                let _ = shutdown_rx.await;
                            },
                            |bound| {
                                let _ = events.send(ServerEvent::Running(bound));
                            },
                        ))
                        .map_err(|error| error.to_string())
                });
            let _ = events.send(ServerEvent::Stopped(result));
        });
    }

    fn stop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
            self.status = ServerStatus::Stopping;
            self.message = "正在停止代理…".into();
        }
    }

    fn poll_events(&mut self) {
        while let Ok(event) = self.events_rx.try_recv() {
            match event {
                ServerEvent::Running(addr) => {
                    self.status = ServerStatus::Running(addr);
                    self.message = format!("代理运行中: http://{addr}");
                }
                ServerEvent::Stopped(result) => {
                    self.status = ServerStatus::Stopped;
                    self.shutdown = None;
                    self.message = match result {
                        Ok(()) => "代理已停止".into(),
                        Err(error) => format!("代理停止或启动失败: {error}"),
                    };
                }
            }
        }
    }

}
