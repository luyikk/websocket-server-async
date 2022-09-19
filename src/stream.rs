use std::{
    pin::Pin,
    task::{Context, Poll},
};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

#[non_exhaustive]
#[derive(Debug)]
pub enum MaybeRustlsStream<S> {
    Plain(S),
    ServerTls(tokio_rustls::server::TlsStream<S>),
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncRead for MaybeRustlsStream<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            MaybeRustlsStream::Plain(ref mut s) => Pin::new(s).poll_read(cx, buf),
            MaybeRustlsStream::ServerTls(ref mut s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncWrite for MaybeRustlsStream<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            MaybeRustlsStream::Plain(ref mut s) => Pin::new(s).poll_write(cx, buf),
            MaybeRustlsStream::ServerTls(ref mut s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            MaybeRustlsStream::Plain(ref mut s) => Pin::new(s).poll_flush(cx),
            MaybeRustlsStream::ServerTls(ref mut s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            MaybeRustlsStream::Plain(ref mut s) => Pin::new(s).poll_shutdown(cx),
            MaybeRustlsStream::ServerTls(ref mut s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}
