use http::Uri;
use hyper_util::rt::TokioIo;
use secure_crypt::user_application::BoxedApplicationStream;
use std::future::{Ready, ready};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tonic::transport::server::Connected;
use tower::Service;

pub struct BoxedApplicationWrapper {
    pub inner: BoxedApplicationStream,
}

impl Connected for BoxedApplicationWrapper {
    type ConnectInfo = ();

    fn connect_info(&self) -> Self::ConnectInfo {
        ()
    }
}

impl AsyncRead for BoxedApplicationWrapper {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for BoxedApplicationWrapper {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

pub struct SecureCryptConnector {
    pub stream: Option<BoxedApplicationWrapper>,
}

impl Service<Uri> for SecureCryptConnector {
    type Response = hyper_util::rt::TokioIo<BoxedApplicationWrapper>;
    type Error = std::io::Error;
    type Future = Ready<Result<TokioIo<BoxedApplicationWrapper>, std::io::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _uri: Uri) -> Self::Future {
        ready(self.stream.take().map(TokioIo::new).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "SecureCrypt stream already consumed",
            )
        }))
    }
}
