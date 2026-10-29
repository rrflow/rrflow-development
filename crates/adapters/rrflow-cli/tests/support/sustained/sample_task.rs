//! One owned asynchronous sample. This does not isolate synchronous OS calls.
use super::DriverResult;
use std::future::Future;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::task::JoinSet;

/// Explicitly constructed by the installed smoke only; no environment or
/// engine input can activate this harness-owned asynchronous test gate.
#[derive(Default)]
pub(super) struct SampleGate {
    pub entered: tokio::sync::Notify,
    pub release: tokio::sync::Notify,
    pub dropped: AtomicBool,
    open: AtomicBool,
}

impl SampleGate {
    pub(super) fn open(&self) {
        self.open.store(true, Ordering::Release);
        // Only one sample can own this gate at a time. notify_one retains a
        // permit if opening races with registration of that one waiter.
        self.release.notify_one();
    }

    pub(super) async fn wait(self: Arc<Self>) {
        if self.open.load(Ordering::Acquire) {
            return;
        }
        struct Guard(Arc<SampleGate>);
        impl Drop for Guard {
            fn drop(&mut self) {
                self.0.dropped.store(true, Ordering::SeqCst);
            }
        }
        let _guard = Guard(self.clone());
        self.entered.notify_one();
        if !self.open.load(Ordering::Acquire) {
            self.release.notified().await;
        }
    }
}

pub(super) struct SampleTask<T> {
    tasks: JoinSet<DriverResult<T>>,
}

impl<T: Send + 'static> SampleTask<T> {
    pub(super) fn new() -> Self {
        Self {
            tasks: JoinSet::new(),
        }
    }

    pub(super) fn is_pending(&self) -> bool {
        !self.tasks.is_empty()
    }

    pub(super) fn start(
        &mut self,
        work: impl Future<Output = DriverResult<T>> + Send + 'static,
    ) -> DriverResult<()> {
        if self.is_pending() {
            return Err("resource sample already pending".into());
        }
        self.tasks.spawn(work);
        Ok(())
    }

    pub(super) async fn finish(&mut self) -> DriverResult<Option<T>> {
        self.tasks
            .join_next()
            .await
            .map(|result| result.map_err(|_| "resource sample task panicked".to_owned())?)
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::time::Duration;

    #[tokio::test]
    async fn opening_gate_releases_current_and_future_samples() {
        let gate = Arc::new(SampleGate::default());
        let first_gate = gate.clone();
        let first = tokio::spawn(async move { first_gate.wait().await });
        tokio::time::timeout(Duration::from_secs(1), gate.entered.notified())
            .await
            .unwrap();
        assert!(!first.is_finished());
        gate.open();
        tokio::time::timeout(Duration::from_secs(1), first)
            .await
            .unwrap()
            .unwrap();
        tokio::time::timeout(Duration::from_secs(1), gate.clone().wait())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn held_sample_allows_offer_and_denies_overlap() {
        let (release, wait) = tokio::sync::oneshot::channel();
        let mut sample = SampleTask::new();
        sample
            .start(async move {
                wait.await.map_err(|_| "closed".to_owned())?;
                Ok(7)
            })
            .unwrap();
        tokio::task::yield_now().await;
        assert!(sample.start(async { Ok(8) }).is_err());
        let offer = tokio::time::timeout(Duration::from_millis(100), async { 11 })
            .await
            .unwrap();
        assert_eq!(offer, 11);
        release.send(()).unwrap();
        assert_eq!(sample.finish().await.unwrap(), Some(7));
        assert!(!sample.is_pending());
        assert_eq!(sample.finish().await.unwrap(), None);
    }

    #[tokio::test]
    async fn errors_and_panics_are_not_lost() {
        let mut sample = SampleTask::<u8>::new();
        sample
            .start(async { Err("measurement failed".into()) })
            .unwrap();
        assert_eq!(sample.finish().await.unwrap_err(), "measurement failed");
        sample
            .start(async { panic!("controlled sample panic") })
            .unwrap();
        assert_eq!(
            sample.finish().await.unwrap_err(),
            "resource sample task panicked"
        );
        assert!(!sample.is_pending());
    }

    #[tokio::test]
    async fn dropping_task_cancels_owned_async_sample() {
        struct Release(Arc<AtomicBool>);
        impl Drop for Release {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }
        let released = Arc::new(AtomicBool::new(false));
        let owner = Release(released.clone());
        let mut sample = SampleTask::<u8>::new();
        sample
            .start(async move {
                let _owner = owner;
                std::future::pending().await
            })
            .unwrap();
        tokio::task::yield_now().await;
        drop(sample);
        tokio::time::timeout(Duration::from_secs(1), async {
            while !released.load(Ordering::SeqCst) {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
    }
}
