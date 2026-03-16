#![allow(dead_code)]

use etcd_client::{Client, Compare, CompareOp, Error, Txn, TxnOp};
use tokio::time::{Duration, sleep};

pub struct DistBarrier {
    parties: usize,
    barrier_state: BarrierState,
    client: Client,
    key: String,
}

#[derive(Default)]
struct BarrierState {
    arrived: usize,
    generation: usize,
}

impl DistBarrier {
    pub async fn new(parties: usize, endpoint: &str, key: impl Into<String>) -> Result<Self, Error> {
        assert!(parties > 0, "barrier requires at least one participant");

        let client = Client::connect([endpoint], None).await?;

        Ok(Self {
            parties,
            barrier_state: BarrierState::default(),
            client,
            key: key.into(),
        })
    }

    pub async fn wait(&mut self) -> Result<usize, Error> {
        let key_value = self.increment_key().await?;
        self.barrier_state.arrived = key_value;

        loop {
            let current = self.read_key_value().await?;

            if current == self.parties {
                self.barrier_state.generation += 1;
                return Ok(self.barrier_state.generation);
            }

            sleep(Duration::from_millis(50)).await;
        }
    }

    pub fn generation(&self) -> usize {
        self.barrier_state.generation
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
        key.push_str(":");
        key.push_str(&self.generation().to_string());
        key
    }
}
