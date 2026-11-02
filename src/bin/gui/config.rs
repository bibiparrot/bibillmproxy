use std::{
    fs,
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use eframe::egui;
use litellm_rust::{
    gateway_server,
    proxy::config::parse_config,
    sdk::{
        providers::{self, transform::ProviderRegistry},
        router::Router,
    },
};
use serde_yaml::{Mapping, Value};
use rust_i18n::t;
use tokio::sync::oneshot;

const DEFAULT_YAML: &str = "model_list: []\n\n# Add an upstream proxy in the Routes tab.\n";

#[derive(Default)]
struct RouteDraft {
    alias: String,
    model: String,
    api_base: String,
    api_key: String,
    other: Mapping,
    params_other: Mapping,
}

impl RouteDraft {
    fn from_value(value: &Value) -> Result<Self, String> {
        let mut other = value
            .as_mapping()
            .cloned()
            .ok_or("model_list entry must be a map")?;
        let alias = take_string(&mut other, "model_name")?;
        let params = other
            .remove(Value::String("litellm_params".into()))
            .ok_or("missing litellm_params")?;
        let mut params_other = params
            .as_mapping()
            .cloned()
            .ok_or("litellm_params must be a map")?;
        Ok(Self {
            alias,
            model: take_string(&mut params_other, "model")?,
            api_base: take_optional_string(&mut params_other, "api_base")?,
            api_key: take_optional_string(&mut params_other, "api_key")?,
            other,
            params_other,
        })
    }

    fn to_value(&self) -> Value {
        let mut entry = self.other.clone();
        let mut params = self.params_other.clone();
        put(&mut entry, "model_name", &self.alias);
        put(&mut params, "model", &self.model);
        if !self.api_base.is_empty() {
            put(&mut params, "api_base", &self.api_base);
        }
        if !self.api_key.is_empty() {
            put(&mut params, "api_key", &self.api_key);
        }
        entry.insert(
            Value::String("litellm_params".into()),
            Value::Mapping(params),
        );
        Value::Mapping(entry)
    }
}

fn take_string(map: &mut Mapping, key: &str) -> Result<String, String> {
    map.remove(Value::String(key.into()))
        .and_then(|value| value.as_str().map(str::to_owned))
        .ok_or_else(|| format!("{key} must be a string"))
}

fn take_optional_string(map: &mut Mapping, key: &str) -> Result<String, String> {
    match map.remove(Value::String(key.into())) {
        None | Some(Value::Null) => Ok(String::new()),
        Some(Value::String(value)) => Ok(value),
        Some(_) => Err(format!("{key} must be a string")),
    }
}

fn put(map: &mut Mapping, key: &str, value: &str) {
    map.insert(Value::String(key.into()), Value::String(value.into()));
}

fn read_routes(yaml: &str) -> Result<(Vec<RouteDraft>, String), String> {
    let document: Value = serde_yaml::from_str(yaml).map_err(|error| error.to_string())?;
    let map = document.as_mapping().ok_or("YAML root must be a map")?;
    let routes = map
        .get(Value::String("model_list".into()))
        .and_then(Value::as_sequence)
        .ok_or("model_list must be a list")?
        .iter()
        .map(RouteDraft::from_value)
        .collect::<Result<Vec<_>, _>>()?;
    let master_key = map
        .get(Value::String("general_settings".into()))
        .and_then(Value::as_mapping)
        .and_then(|settings| settings.get(Value::String("master_key".into())))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    Ok((routes, master_key))
}

fn write_routes(yaml: &str, routes: &[RouteDraft], master_key: &str) -> Result<String, String> {
    let mut document: Value = serde_yaml::from_str(yaml).map_err(|error| error.to_string())?;
    let root = document.as_mapping_mut().ok_or("YAML root must be a map")?;
    root.insert(
        Value::String("model_list".into()),
        Value::Sequence(routes.iter().map(RouteDraft::to_value).collect()),
    );
    let settings = root
        .entry(Value::String("general_settings".into()))
        .or_insert_with(|| Value::Mapping(Mapping::new()))
        .as_mapping_mut()
        .ok_or("general_settings must be a map")?;
    if master_key.is_empty() {
        settings.remove(Value::String("master_key".into()));
    } else {
        put(settings, "master_key", master_key);
    }
    serde_yaml::to_string(&document).map_err(|error| error.to_string())
}
