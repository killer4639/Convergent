use std::sync::LazyLock;

use axum::{Router, response::IntoResponse, routing::get};
use prometheus::{Encoder, Histogram, HistogramOpts, TextEncoder, opts};

pub static DETECTION_LATENCY: LazyLock<Histogram> = LazyLock::new(|| {
    Histogram::with_opts(HistogramOpts {
        common_opts: opts!(
            "detection_latency_seconds",
            "Time from barrier completion to node detection"
        ),
        buckets: vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.9, 0.95, 0.99, 1.0],
    })
    .expect("failed to create detection_latency histogram")
});

/// Register all metrics with the default Prometheus registry.
/// Call once at startup before observing any values.
pub fn register_metrics() {
    prometheus::default_registry()
        .register(Box::new(DETECTION_LATENCY.clone()))
        .expect("failed to register detection_latency");
}

async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    (
        [("content-type", "text/plain; charset=utf-8")],
        String::from_utf8(buffer).unwrap(),
    )
}

pub async fn start_metrics_server(port: u16) {
    let app = Router::new().route("/metrics", get(metrics_handler));
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind metrics server");
    println!("Metrics server listening on {addr}");
    axum::serve(listener, app)
        .await
        .expect("metrics server failed");
}
