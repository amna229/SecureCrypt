//! Application integration interface.
//!
//! This module defines the stream abstraction exposed to applications
//! integrating SecureCrypt. The application can use this stream to
//! implement its own application-level protocol without depending on
//! the concrete TLS implementation.

use std::pin::Pin;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::task::{Context, Poll};
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// Asynchronous bidirectional stream exposed to the application.
///
/// The application uses this stream to implement its own application-level
/// protocol, such as HTTP/1.1, WebSocket or gRPC.
pub trait ApplicationStream: AsyncRead + AsyncWrite + Unpin + Send {}

impl<T> ApplicationStream for T where T: AsyncRead + AsyncWrite + Unpin + Send {}

/// Boxed stream exposed to the application.
pub type BoxedApplicationStream = Pin<Box<dyn ApplicationStream>>;

/// Shared transfer metrics collected by a `MetricsStream`.
#[derive(Debug)]
struct TransferMetricsState {
    bytes_read: AtomicU64,
    bytes_written: AtomicU64,
    first_activity_nanos: AtomicU64,
}

/// Handle used to retrieve the metrics collected by a `MetricsStream`.
#[derive(Clone, Debug)]
pub struct TransferMetricsHandle {
    state: Arc<TransferMetricsState>,
}

/// Generic stream wrapper that measures application-level traffic.
///
/// The wrapper is independent of the protocol used by the application.
/// It counts bytes read and written through the SecureCrypt-protected
/// stream and records the time between the first application activity
/// and the end of the application operation.
pub struct MetricsStream {
    inner: BoxedApplicationStream,
    state: Arc<TransferMetricsState>,
}

impl MetricsStream {
    /// Creates a metrics-enabled stream and its corresponding metrics handle.
    pub fn new(inner: BoxedApplicationStream) -> (Self, TransferMetricsHandle) {
        let state = Arc::new(TransferMetricsState {
            bytes_read: AtomicU64::new(0),

            bytes_written: AtomicU64::new(0),

            first_activity_nanos: AtomicU64::new(0),
        });

        let handle = TransferMetricsHandle {
            state: Arc::clone(&state),
        };

        let stream = Self { inner, state };

        (stream, handle)
    }

    /// Records the first application-level activity.
    fn record_first_activity(&self) {
        let now = current_time_nanos();

        let _ = self.state.first_activity_nanos.compare_exchange(
            0,
            now,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );
    }
}

impl TransferMetricsHandle {
    /// Returns the total number of bytes received and sent.
    pub fn bytes_received(&self) -> u64 {
        self.state.bytes_read.load(Ordering::Relaxed)
    }

    /// Returns the total number of bytes sent.
    pub fn bytes_sent(&self) -> u64 {
        self.state.bytes_written.load(Ordering::Relaxed)
    }

    /// Returns the total number of application bytes transferred.
    pub fn bytes_transferred(&self) -> u64 {
        self.bytes_received() + self.bytes_sent()
    }

    /// Returns the elapsed transfer duration in milliseconds.
    pub fn duration_ms(&self) -> i64 {
        let started = self.state.first_activity_nanos.load(Ordering::Relaxed);

        if started == 0 {
            return 0;
        }

        let now = current_time_nanos();

        now.saturating_sub(started).saturating_div(1_000_000) as i64
    }

    /// Calculates the aggregate throughput in megabits per second.
    pub fn throughput_mbps(&self) -> f64 {
        let bytes = self.bytes_transferred();

        let duration_ms = self.duration_ms();

        if bytes == 0 || duration_ms <= 0 {
            return 0.0;
        }

        (bytes as f64 * 8.0) / (duration_ms as f64 * 1000.0)
    }
}

impl AsyncRead for MetricsStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let before = buf.filled().len();

        let result = self.inner.as_mut().poll_read(cx, buf);

        if let Poll::Ready(Ok(())) = &result {
            let after = buf.filled().len();

            let bytes = after.saturating_sub(before);

            if bytes > 0 {
                self.record_first_activity();

                self.state
                    .bytes_read
                    .fetch_add(bytes as u64, Ordering::Relaxed);
            }
        }

        result
    }
}

impl AsyncWrite for MetricsStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let result = self.inner.as_mut().poll_write(cx, buf);

        if let Poll::Ready(Ok(bytes)) = &result {
            if *bytes > 0 {
                self.record_first_activity();

                self.state
                    .bytes_written
                    .fetch_add(*bytes as u64, Ordering::Relaxed);
            }
        }

        result
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.inner.as_mut().poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.inner.as_mut().poll_shutdown(cx)
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> Poll<std::io::Result<usize>> {
        self.inner.as_mut().poll_write_vectored(cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }
}

/// Returns the current system time in nanoseconds since the Unix epoch.
fn current_time_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0)
}
