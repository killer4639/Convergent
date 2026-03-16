mod dist_barrier;

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
    let endpoint = env::var("ETCD_ENDPOINT").unwrap_or_else(|_| "http://localhost:2379".to_string());
    let key = env::var("BARRIER_KEY").unwrap_or_else(|_| "barrier".to_string());

    let mut barrier = dist_barrier::DistBarrier::new(parties, &endpoint, key).await?;
    loop {
        let wait_ms = rand::rng().random_range(5000..=10000);
        sleep(Duration::from_millis(wait_ms)).await;
        barrier.wait().await?;
        println!("Barrier generation: {}", barrier.generation());
    }
}
