use std::{net::SocketAddr, path::PathBuf};

use clap::{Args as ClapArgs, Parser, Subcommand};
use litellm_rust::gateway_server;
use tracing_subscriber::EnvFilter;

mod cli;

#[derive(Debug, Parser)]
#[command(about = "Low-overhead LiteLLM-compatible gateway")]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    #[command(flatten)]
    serve: ServeArgs,
}

#[derive(Debug, Subcommand)]
enum Command {
    Serve(ServeArgs),
    Logout,
}

#[derive(Debug, Clone, ClapArgs)]
struct ServeArgs {
    #[arg(long, env = "LITELLM_CONFIG", default_value = "config.yaml")]
    config: PathBuf,

    #[arg(long, env = "HOST", default_value = "127.0.0.1")]
    host: String,

    #[arg(long, env = "PORT", default_value_t = 4000)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match std::env::args_os().nth(1).as_deref() {
        Some(arg) if arg == std::ffi::OsStr::new("claude") => {
            let claude_args = cli::parse_claude_args(std::env::args_os().skip(2))?;
            std::process::exit(cli::run_claude_wizard(claude_args)?);
        }
        Some(arg) if arg == std::ffi::OsStr::new("codex") => {
            let codex_args = cli::parse_codex_args(std::env::args_os().skip(2))?;
            std::process::exit(cli::run_codex_wizard(codex_args)?);
        }
        None => {
            std::process::exit(cli::run_tool_selector()?);
        }
        _ => {}
    }

    let args = Args::parse();
    match args.command {
        Some(Command::Serve(serve)) => serve_gateway(serve).await,
        Some(Command::Logout) => cli::logout(),
        None => serve_gateway(args.serve).await,
    }
}

async fn serve_gateway(args: ServeArgs) -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    init_tracing();

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    gateway_server::serve(&args.config, addr, shutdown_signal(), |bound| {
        println!("LiteLLM proxy listening on http://{bound}");
    })
    .await?;

    println!("\nINFO:     Shutting down LiteLLM Proxy Server");
    Ok(())
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
