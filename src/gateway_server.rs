use std::{future::Future, net::SocketAddr, path::Path, sync::Arc};

use axum::Router as AxumRouter;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::{
    db::managed_agents::pool as managed_agents_pool,
    http::routes::router,
    model_prices,
    proxy::{
        config::{load_config, GatewayConfig},
        state::AppState,
    },
    sdk::{
        providers::{self, transform::ProviderRegistry},
        router::Router,
    },
};

pub async fn serve(
    config_path: &Path,
    addr: SocketAddr,
    shutdown: impl Future<Output = ()> + Send + 'static,
    ready: impl FnOnce(SocketAddr),
) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config(config_path)?;
    let mut providers = ProviderRegistry::new();
    providers::register_all(&mut providers);
    let model_router = Router::from_config(&config, &providers)?;
    let http = AppState::build_http_client()?;
    let model_cost_map = model_prices::load(&http).await;
    let db = build_managed_agents_pool(&config).await?;
    let state = Arc::new(AppState::new(
        config.clone(),
        model_router,
        http,
        model_cost_map,
        db,
    )?);
    let app: AxumRouter = router(state).layer(TraceLayer::new_for_http());
    let listener = TcpListener::bind(addr).await?;
    let bound_addr = listener.local_addr()?;
    ready(bound_addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;
    Ok(())
}

async fn build_managed_agents_pool(
    config: &GatewayConfig,
) -> Result<Option<sqlx::PgPool>, Box<dyn std::error::Error>> {
    let Some(database_url) = config.general_settings.database_url.as_deref() else {
        return Ok(None);
    };
    let pool = managed_agents_pool::connect(database_url).await?;
    managed_agents_pool::migrate(&pool).await?;
    Ok(Some(pool))
}
