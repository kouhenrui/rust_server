//! Binary entrypoint. Wires up tracing, builds [`AppState`], and starts axum.

use std::net::SocketAddr;
use thumbor::{config::Config, logger, router, state::AppState};
use tower_http::cors::CorsLayer;

/// 进程入口。
///
/// `tokio::main` 选择多线程 runtime 是因为图像处理（尤其 WebP/PNG 编码）
/// 是 CPU 密集的，多 worker 才能让 I/O 与编解码重叠；信号处理走
/// `with_graceful_shutdown` 是为了让在飞的请求在退出前有机会完成。
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 尽早加载 .env，使 RUST_LOG 等变量在 init 之前生效
    Config::load_dotenv();
    logger::init();

    let config = Config::from_env();
    thumbor::info!("config: {:?}", config);
    let bind_addr: SocketAddr = config.bind_addr;

    let state = AppState::connect(config).await?;
    let app = router::router(state).layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    thumbor::info!(%bind_addr, "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

/// 等待任意一种停机信号。
///
/// 同时监听 SIGINT（Ctrl-C）和 SIGTERM 是容器化部署的基本盘：Docker/k8s
/// 优先发 SIGTERM，开发者本地 Ctrl-C 发 SIGINT；只听一种会卡死任一场景。
/// 收到信号后只 `info!` 一行就把控制权交还给 axum 的 graceful shutdown，
/// 由它来等在飞请求完成。
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        let mut term = tokio::signal::unix::signal(
            tokio::signal::unix::SignalKind::terminate(),
        )
        .expect("install SIGTERM handler");
        term.recv().await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    thumbor::info!("shutdown signal received");
}
