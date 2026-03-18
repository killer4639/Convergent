#![allow(dead_code)]

use etcd_client::{Client, Compare, CompareOp, Error, Txn, TxnOp};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

pub struct DistBarrier {
    parties: usize,
    barrier_state: BarrierState,
    client: Client,
    key: String,
}

/// Result returned by `wait()` with generation number and detection latency.
pub struct WaitResult {
    pub generation: usize,
    pub detection_latency: Duration,
}

#[derive(Default)]
struct BarrierState {
    arrived: usize,
    generation: usize,
}

impl DistBarrier {
    pub async fn new(
        parties: usize,
        endpoint: &str,
        key: impl Into<String>,
    ) -> Result<Self, Error> {
        assert!(parties > 0, "barrier requires at least one participant");

        let client = Client::connect([endpoint], None).await?;

        Ok(Self {
            parties,
            barrier_state: BarrierState::default(),
            client,
            key: key.into(),
        })
    }

    pub async fn wait(&mut self) -> Result<WaitResult, Error> {
        let next = self.increment_key().await?;
        self.barrier_state.arrived = next;

        let detection_latency = if next >= self.parties {
            // This node completed the barrier — write the completion timestamp.
            self.write_completion_timestamp().await?;
            Duration::ZERO
        } else {
            // Poll until the barrier is satisfied, then compute detection latency.
            loop {
                sleep(tokio::time::Duration::from_millis(50)).await;
                let key_value = self.read_key_value().await?;
                if key_value >= self.parties {
                    break self.compute_detection_latency().await?;
                }
            }
        };

        self.barrier_state.generation += 1;
        Ok(WaitResult {
            generation: self.barrier_state.generation,
            detection_latency,
        })
    }

    pub fn generation(&self) -> usize {
        self.barrier_state.generation
    }

    /// Write the current wall-clock time (epoch millis) to the completion key.
    async fn write_completion_timestamp(&mut self) -> Result<(), Error> {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let key = self.get_completed_at_key();
        self.client.put(key, millis.to_string(), None).await?;
        Ok(())
    }

    /// Read the completion timestamp and compute how long ago it was.
    /// Retries once after 5ms if the key doesn't exist yet (race with the completing node).
    async fn compute_detection_latency(&mut self) -> Result<Duration, Error> {
        let key = self.get_completed_at_key();

        for attempt in 0..2 {
            let response = self.client.get(key.clone(), None).await?;
            if let Some(kv) = response.kvs().first() {
                if let Some(completed_millis) =
                    kv.value_str().ok().and_then(|v| v.parse::<u128>().ok())
                {
                    let now_millis = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis();
                    let latency_millis = now_millis.saturating_sub(completed_millis);
                    return Ok(Duration::from_millis(latency_millis as u64));
                }
            }
            if attempt == 0 {
                sleep(tokio::time::Duration::from_millis(5)).await;
            }
        }

        // Fallback: couldn't read the timestamp, report zero.
        Ok(Duration::ZERO)
    }

    async fn increment_key(&mut self) -> Result<usize, Error> {
        loop {
            let key = self.get_key();
            let response = self.client.get(key.clone(), None).await?;
            let current = response.kvs().first().map(Self::parse_counter).unwrap_or(0);
            let version = response.kvs().first().map(|kv| kv.version()).unwrap_or(0);
            let next = current + 1;

            let txn = Txn::new()
                .when([Compare::version(key.clone(), CompareOp::Equal, version)])
                .and_then([TxnOp::put(key, next.to_string(), None)]);

            let response = self.client.txn(txn).await?;

            if response.succeeded() {
                return Ok(next);
            }
        }
    }

    async fn read_key_value(&mut self) -> Result<usize, Error> {
        let response = self.client.get(self.get_key(), None).await?;

        Ok(response.kvs().first().map(Self::parse_counter).unwrap_or(0))
    }

    fn parse_counter(kv: &etcd_client::KeyValue) -> usize {
        kv.value_str()
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0)
    }

    fn get_key(&self) -> String {
        let mut key = self.key.clone();
        key.push(':');
        key.push_str(&self.generation().to_string());
        key
    }

    fn get_completed_at_key(&self) -> String {
        let mut key = self.get_key();
        key.push_str(":completed_at");
        key
    }
}
