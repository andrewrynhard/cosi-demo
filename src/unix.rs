#[cfg(unix)]
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    self,
    signal::unix::{signal, SignalKind},
};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tonic::transport::server::Connected;

pub async fn handle_signals(
    hangup_handle: impl Fn(),
    interrupt_handle: impl Fn(),
    child_handle: impl Fn(),
) {
    let mut hangup_stream = signal(SignalKind::hangup()).unwrap();
    let mut interrupt_stream = signal(SignalKind::interrupt()).unwrap();
    let mut child_stream = signal(SignalKind::child()).unwrap();

    loop {
        tokio::select! {
            _ = hangup_stream.recv()=> {
                hangup_handle();
            }
            _ = interrupt_stream.recv()=> {
                interrupt_handle();
            }
            _ = child_stream.recv()=> {
                child_handle();
            }
        };
    }
}

pub struct UnixIncoming {
    inner: tokio::net::UnixListener,
}

impl UnixIncoming {
    pub fn bind<P>(path: P) -> std::io::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(Self {
            inner: tokio::net::UnixListener::bind(path)?,
        })
    }
}

impl futures::Stream for UnixIncoming {
    type Item = Result<UnixStream, std::io::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let result = futures::ready!(self
            .inner
            .poll_accept(cx)
            .map(|result| result.map(|(sock, _addr)| UnixStream(sock))));
        Poll::Ready(Some(result))
    }
}

#[derive(Debug)]
pub struct UnixStream(pub tokio::net::UnixStream);

impl Connected for UnixStream {}

impl AsyncRead for UnixStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for UnixStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}
