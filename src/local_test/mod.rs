pub mod barrier;

#[cfg(test)]
mod tests {
    use super::barrier::{Barrier, BarrierError};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn barrier_blocks_until_all_threads_arrive() {
        let participants = 4;
        let barrier = Arc::new(Barrier::new(participants));
        let arrived = Arc::new(AtomicUsize::new(0));
        let (tx, rx) = mpsc::channel();

        for _ in 0..(participants - 1) {
            let barrier = Arc::clone(&barrier);
            let arrived = Arc::clone(&arrived);
            let tx = tx.clone();

            thread::spawn(move || {
                arrived.fetch_add(1, Ordering::SeqCst);
                barrier.wait().expect("worker should pass the barrier");
                tx.send(()).expect("worker should report completion");
            });
        }

        while arrived.load(Ordering::SeqCst) < participants - 1 {
            thread::yield_now();
        }

        assert!(
            rx.recv_timeout(Duration::from_millis(100)).is_err(),
            "threads passed the barrier before all participants arrived"
        );

        arrived.fetch_add(1, Ordering::SeqCst);
        barrier
            .wait()
            .expect("final participant should release the barrier");

        for _ in 0..(participants - 1) {
            rx.recv_timeout(Duration::from_secs(1))
                .expect("all worker threads should pass once the final participant arrives");
        }
    }

    #[test]
    fn barrier_returns_error_when_reused() {
        let barrier = Barrier::new(1);

        barrier
            .wait()
            .expect("single participant should complete the barrier");

        assert_eq!(barrier.wait(), Err(BarrierError::AlreadyUsed));
    }
}
