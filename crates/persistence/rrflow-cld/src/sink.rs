use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    time::Instant,
};

// Parquet's buffered footer can stringify its sink error. Preserve the first
// failure beside this writer so an elapsed clock cannot change its identity.
pub(crate) const SINK_NO_FAILURE: u8 = 0;
pub(crate) const SINK_DEADLINE: u8 = 1;
pub(crate) const SINK_BYTE_LIMIT: u8 = 2;
pub(crate) const SINK_ALLOCATION: u8 = 3;

pub(crate) struct BoundedOutput {
    pub(crate) bytes: Vec<u8>,
    pub(crate) limit: usize,
    pub(crate) deadline: Instant,
    pub(crate) failure: Arc<AtomicU8>,
}

impl Write for BoundedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if Instant::now() >= self.deadline {
            let _ = self.failure.compare_exchange(
                SINK_NO_FAILURE,
                SINK_DEADLINE,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Parquet deadline exceeded",
            ));
        }
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            let _ = self.failure.compare_exchange(
                SINK_NO_FAILURE,
                SINK_BYTE_LIMIT,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
            return Err(io::Error::other("Parquet byte budget exceeded"));
        }
        self.bytes.try_reserve_exact(bytes.len()).map_err(|error| {
            let _ = self.failure.compare_exchange(
                SINK_NO_FAILURE,
                SINK_ALLOCATION,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
            io::Error::other(error)
        })?;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
