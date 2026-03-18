mod dist_barrier;
mod metrics;

use etcd_client::Error;
use rand::Rng;
use std::env;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let parties = env::var("PARTIES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(3);
    let endpoint =
        env::var("ETCD_ENDPOINT").unwrap_or_else(|_| "http://localhost:2379".to_string());
    let key = env::var("BARRIER_KEY").unwrap_or_else(|_| "barrier".to_string());
    let metrics_port: u16 = env::var("METRICS_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(9090);

    metrics::register_metrics();
    tokio::spawn(metrics::start_metrics_server(metrics_port));

    let mut barrier = dist_barrier::DistBarrier::new(parties, &endpoint, key).await?;
    loop {
        let wait_ms = rand::rng().random_range(5000..=10000);
        sleep(Duration::from_millis(wait_ms)).await;
        let result = barrier.wait().await?;
        let latency_secs = result.detection_latency.as_secs_f64();
        metrics::DETECTION_LATENCY.observe(latency_secs);
        println!(
            "Generation {}: detection latency = {:.3}ms",
            result.generation,
            latency_secs * 1000.0
        );
    }
}
