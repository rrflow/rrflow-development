//! A finite set of borrowed futures: cancellation drops work synchronously.
//! No spawned task can outlive its corpus window or session.
use super::DriverResult;
use std::future::{poll_fn, Future};
use std::task::Poll;

pub(super) async fn ordered<F, T>(
    futures: impl ExactSizeIterator<Item = F>,
    maximum: usize,
) -> DriverResult<Vec<T>>
where
    F: Future<Output = DriverResult<T>>,
{
    if maximum == 0 || maximum > 64 || futures.len() > maximum {
        return Err("corpus readback window exceeds admitted clients".into());
    }
    // Validate cardinality before allocating or polling caller work.
    let mut outputs: Vec<_> = (0..futures.len()).map(|_| None).collect();
    let mut pending: Vec<_> = futures.map(|future| Some(Box::pin(future))).collect();
    poll_fn(|cx| {
        let mut incomplete = false;
        for (index, slot) in pending.iter_mut().enumerate() {
            if let Some(future) = slot {
                match future.as_mut().poll(cx) {
                    Poll::Ready(Ok(value)) => {
                        outputs[index] = Some(value);
                        *slot = None;
                    }
                    Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
                    Poll::Pending => incomplete = true,
                }
            }
        }
        if incomplete {
            Poll::Pending
        } else {
            Poll::Ready(Ok(outputs.iter_mut().map(|v| v.take().unwrap()).collect()))
        }
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use std::time::Duration;

    struct Owned(Arc<AtomicUsize>);
    impl Drop for Owned {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn corpus_window_overlaps_reads_and_preserves_canonical_order() {
        let barrier = tokio::sync::Barrier::new(4);
        let completed = AtomicUsize::new(0);
        let result = tokio::time::timeout(
            Duration::from_secs(2),
            ordered(
                (0..4).map(|index| {
                    let barrier = &barrier;
                    let completed = &completed;
                    async move {
                        // Serial polling to completion deadlocks here.
                        barrier.wait().await;
                        // Force reverse completion without a wall-clock race.
                        poll_fn(|cx| {
                            if completed.load(Ordering::SeqCst) == 3 - index {
                                completed.fetch_add(1, Ordering::SeqCst);
                                cx.waker().wake_by_ref();
                                Poll::Ready(())
                            } else {
                                cx.waker().wake_by_ref();
                                Poll::Pending
                            }
                        })
                        .await;
                        Ok(index)
                    }
                }),
                4,
            ),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(completed.load(Ordering::SeqCst), 4);
        assert_eq!(result, vec![0, 1, 2, 3]);
    }

    #[tokio::test]
    async fn corpus_window_denies_before_polling_and_drops_all_owned_work() {
        for maximum in [0, 2, 65] {
            let polls = AtomicUsize::new(0);
            let result = ordered(
                (0..3).map(|_| async {
                    polls.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }),
                maximum,
            )
            .await;
            assert!(result.unwrap_err().contains("admitted clients"));
            assert_eq!(polls.load(Ordering::SeqCst), 0);
        }
        for fail in [false, true] {
            let released = Arc::new(AtomicUsize::new(0));
            let futures = (0..4).map(|i| {
                let owner = Owned(released.clone());
                async move {
                    let _owner = owner;
                    if fail && i == 3 {
                        return Err("independent readback mismatch".into());
                    }
                    std::future::pending::<DriverResult<()>>().await
                }
            });
            let result = tokio::time::timeout(Duration::from_millis(10), ordered(futures, 4)).await;
            if fail {
                assert_eq!(
                    result.unwrap().unwrap_err(),
                    "independent readback mismatch"
                );
            } else {
                assert!(result.is_err());
            }
            assert_eq!(released.load(Ordering::SeqCst), 4, "orphaned readback work");
        }
    }
}
