use std::net::SocketAddr;

use eyre::{bail, Context, Result};
use futures::future::BoxFuture;
use metrics_exporter_prometheus::PrometheusHandle;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .wrap_err("building runtime")?
        .block_on(run())
}

async fn run() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("PRETENSE_LOG").unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .json()
        .init();

    let config = std::env::var("PRETENSE_PORTS").wrap_err(
        "environment variable PRETENSE_PORTS must be set to comma separated list of ports",
    )?;

    let ports = config
        .split(',')
        .map(|port| port.parse::<u16>())
        .collect::<Result<Vec<_>, _>>()
        .wrap_err(
            "PRETENSE_PORTS contains invalid port number, must be comma separted list of ports",
        )?;

    let metrics_task = match std::env::var("PRETENSE_METRICS_PORT") {
        Ok(metrics_port) => {
            let metrics_port = metrics_port
                .parse::<u16>()
                .wrap_err("PRETENSE_METRICS_PORT")?;
            if ports.contains(&metrics_port) {
                bail!("PRETENSE_PORTS overlaps wtih PRETENSE_METRICS_PORT");
            }
            let prom_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
                .install_recorder()
                .unwrap();
            Box::pin(metrics(metrics_port, prom_handle)) as BoxFuture<Result<()>>
        }
        Err(_) => Box::pin(async {
            loop {
                tokio::task::yield_now().await;
            }
        }),
    };

    let tasks = ports
        .into_iter()
        .map(|port| {
            Box::pin(async move {
                listen_port(port)
                    .await
                    .wrap_err(format!("listening on port {port}"))
            }) as BoxFuture<Result<()>>
        })
        .chain([metrics_task]);

    let result = futures::future::select_all(tasks).await;
    if let Err(err) = result.0 {
        tracing::error!(?err, "Failed to listen on port");
    }

    Ok(())
}

async fn listen_port(port: u16) -> Result<()> {
    let local_addr = SocketAddr::new("0.0.0.0".parse().unwrap(), port);
    let listener = TcpListener::bind(local_addr)
        .await
        .wrap_err("listening on port")?;

    let labels = [("port", port.to_string())];
    let counter = metrics::counter!("pretense_connection", &labels);

    tracing::info!(addr=?local_addr, "Listening on port");

    loop {
        let (conn, addr) = listener.accept().await.wrap_err("failed to accept port")?;
        tracing::info!(remote_ip = ?addr.ip(), remote_port = ?addr.port(), ?port, "Received connection");
        counter.increment(1);
        drop(conn);
    }
}

async fn metrics(metrics_port: u16, prom_handle: PrometheusHandle) -> Result<()> {
    let router: axum::Router = axum::Router::new().route(
        "/metrics",
        axum::routing::get(move || std::future::ready(prom_handle.render())),
    );
    tracing::info!(port = ?metrics_port, "Starting up metrics server");
    let listener =
        TcpListener::bind(SocketAddr::new("0.0.0.0".parse().unwrap(), metrics_port)).await?;

    axum::serve(listener, router.into_make_service())
        .await
        .wrap_err("failed to start server")
}
