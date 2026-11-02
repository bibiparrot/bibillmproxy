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
    locale: LocaleManager,
}

impl ProxyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings = AppSettings::load().unwrap_or_default();
        let mut config_path = settings.config_path;
        let mut port = settings.port.to_string();
        if !AppSettings::path().exists() {
            if let Some(storage) = cc.storage {
                config_path = storage.get_string("config_path").unwrap_or(config_path);
                port = storage.get_string("port").unwrap_or(port);
            }
        }
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
            locale: LocaleManager::new(settings.locale),
        }
    }

    fn reload(&mut self) {
        match fs::read_to_string(&self.config_path) {
            Ok(yaml) => {
                self.yaml = yaml;
                self.parse_forms();
                self.dirty = false;
                self.message = t!("config_reloaded").to_string();
            }
            Err(error) => self.message = t!("read_failed", error = error).to_string(),
        }
    }

    fn parse_forms(&mut self) {
        match read_routes(&self.yaml) {
            Ok((routes, master_key)) => {
                self.routes = routes;
                self.master_key = master_key;
                self.message = t!("forms_loaded").to_string();
            }
            Err(error) => self.message = t!("yaml_parse_failed", error = error).to_string(),
        }
    }

    fn sync_forms(&mut self) {
        match write_routes(&self.yaml, &self.routes, &self.master_key) {
            Ok(yaml) => {
                self.yaml = yaml;
                self.dirty = true;
            }
            Err(error) => self.message = t!("yaml_update_failed", error = error).to_string(),
        }
    }

    fn save_config(&mut self) -> Result<(), String> {
        let config = parse_config(&self.yaml).map_err(|error| error.to_string())?;
        let mut providers = ProviderRegistry::new();
        providers::register_all(&mut providers);
        Router::from_config(&config, &providers).map_err(|error| error.to_string())?;
        let path = PathBuf::from(&self.config_path);
        if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(path, &self.yaml).map_err(|error| error.to_string())?;
        self.dirty = false;
        self.message = t!("config_saved").to_string();
        Ok(())
    }

    fn start(&mut self) {
        if let Err(error) = self.save_config() {
            self.message = t!("start_config_failed", error = error).to_string();
            return;
        }
        let Ok(port) = self.port.parse::<u16>() else {
            self.message = t!("port_invalid").to_string();
            return;
        };
        if port == 0 {
            self.message = t!("port_invalid").to_string();
            return;
        }
        let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        let path = PathBuf::from(&self.config_path);
        let events = self.events_tx.clone();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        self.shutdown = Some(shutdown_tx);
        self.status = ServerStatus::Starting;
        self.message = t!("starting").to_string();
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
            self.message = t!("stopping").to_string();
        }
    }

    fn poll_events(&mut self) {
        while let Ok(event) = self.events_rx.try_recv() {
            match event {
                ServerEvent::Running(addr) => {
                    self.status = ServerStatus::Running(addr);
                    self.message = t!("running", addr = addr).to_string();
                }
                ServerEvent::Stopped(result) => {
                    self.status = ServerStatus::Stopped;
                    self.shutdown = None;
                    self.message = match result {
                        Ok(()) => t!("stopped").to_string(),
                        Err(error) => t!("stop_failed", error = error).to_string(),
                    };
                }
            }
        }
    }

    fn persist_settings(&mut self) {
        let settings = AppSettings {
            config_path: self.config_path.clone(),
            port: self.port.parse::<u16>().ok().filter(|port| *port > 0).unwrap_or(4000),
            locale: self.locale.requested.clone(),
        };
        if let Err(error) = settings.save() {
            self.message = t!("save_failed", error = error).to_string();
        }
    }

}
