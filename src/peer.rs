use anyhow::{bail, ensure, Result};
use aqueue::Actor;
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use crate::stream::MaybeRustlsStream;

pub struct WSPeer {
    addr: SocketAddr,
    sender: Option<SplitSink<WebSocketStream<MaybeRustlsStream<TcpStream>>, Message>>,
}

impl WSPeer {
    /// 创建一个TCP PEER
    #[inline]
    pub fn new(
        addr: SocketAddr,
        sender: SplitSink<WebSocketStream<MaybeRustlsStream<TcpStream>>, Message>,
    ) -> Arc<Actor<WSPeer>> {
        Arc::new(Actor::new(WSPeer {
            addr,
            sender: Some(sender),
        }))
    }
    /// 是否断线
    #[inline]
    fn is_disconnect(&self) -> bool {
        self.sender.is_none()
    }

    /// 发送
    #[inline]
    async fn send_message(&mut self, message: Message) -> Result<()> {
        if let Some(ref mut sender) = self.sender {
            sender.send(message).await?;
            Ok(())
        } else {
            bail!("ConnectionReset")
        }
    }

    /// 发送
    #[inline]
    async fn send_vec(&mut self, buff: Vec<u8>) -> Result<usize> {
        if let Some(ref mut sender) = self.sender {
            let len = buff.len();
            sender.send(Message::Binary(buff)).await?;
            Ok(len)
        } else {
            bail!("ConnectionReset")
        }
    }

    /// 发送
    #[inline]
    async fn send<'a>(&'a mut self, buff: &'a [u8]) -> Result<usize> {
        if let Some(ref mut sender) = self.sender {
            sender.send(Message::binary(buff)).await?;
            Ok(buff.len())
        } else {
            bail!("ConnectionReset")
        }
    }

    /// flush
    #[inline]
    async fn flush(&mut self) -> Result<()> {
        if let Some(ref mut sender) = self.sender {
            sender.flush().await?;
            Ok(())
        } else {
            bail!("ConnectionReset")
        }
    }

    /// 掐线
    #[inline]
    async fn disconnect(&mut self) -> Result<()> {
        if let Some(mut sender) = self.sender.take() {
            sender.close().await?;
            Ok(())
        } else {
            Ok(())
        }
    }
}

#[async_trait::async_trait]
pub trait IPeer: Sync + Send {
    fn addr(&self) -> SocketAddr;
    async fn is_disconnect(&self) -> Result<bool>;
    async fn send_message(&self, message: Message) -> Result<()>;
    async fn send(&self, buff: Vec<u8>) -> Result<usize>;
    async fn send_all(&self, buff: Vec<u8>) -> Result<()>;
    async fn send_ref(&self, buff: &[u8]) -> Result<usize>;
    async fn send_all_ref(&self, buff: &[u8]) -> Result<()>;
    async fn flush(&self) -> Result<()>;
    async fn disconnect(&self) -> Result<()>;
}

#[async_trait::async_trait]
impl IPeer for Actor<WSPeer> {
    #[inline]
    fn addr(&self) -> SocketAddr {
        unsafe { self.deref_inner().addr }
    }

    #[inline]
    async fn is_disconnect(&self) -> Result<bool> {
        self.inner_call(|inner| async move { Ok(inner.get().is_disconnect()) })
            .await
    }
    #[inline]
    async fn send_message(&self, message: Message) -> Result<()> {
        self.inner_call(|inner| async move { inner.get_mut().send_message(message).await })
            .await
    }

    #[inline]
    async fn send(&self, buff: Vec<u8>) -> Result<usize> {
        ensure!(!buff.is_empty(), "send buff is null");
        self.inner_call(|inner| async move { inner.get_mut().send_vec(buff).await })
            .await
    }
    #[inline]
    async fn send_all(&self, buff: Vec<u8>) -> Result<()> {
        ensure!(!buff.is_empty(), "send buff is null");
        self.inner_call(|inner| async move {
            inner.get_mut().send_vec(buff).await?;
            Ok(())
        })
        .await
    }
    #[inline]
    async fn send_ref(&self, buff: &[u8]) -> Result<usize> {
        self.inner_call(|inner| async move { inner.get_mut().send(buff).await })
            .await
    }

    #[inline]
    async fn send_all_ref(&self, buff: &[u8]) -> Result<()> {
        self.inner_call(|inner| async move {
            inner.get_mut().send(buff).await?;
            Ok(())
        })
        .await
    }

    #[inline]
    async fn flush(&self) -> Result<()> {
        self.inner_call(|inner| async move { inner.get_mut().flush().await })
            .await
    }

    #[inline]
    async fn disconnect(&self) -> Result<()> {
        self.inner_call(|inner| async move { inner.get_mut().disconnect().await })
            .await
    }
}
