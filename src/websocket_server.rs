use crate::peer::{IPeer, WSPeer};
use anyhow::{bail, Result};
use aqueue::Actor;
use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use log::*;
use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::{accept_async_with_config, WebSocketStream};

pub type ConnectEventType = fn(SocketAddr) -> bool;

/// websocket server
pub struct WebSocketServer<I, R, T> {
    listener: Option<TcpListener>,
    connect_event: Option<ConnectEventType>,
    input_event: Arc<I>,
    config: Option<WebSocketConfig>,
    _phantom1: PhantomData<R>,
    _phantom2: PhantomData<T>,
}

unsafe impl<I, R, T> Send for WebSocketServer<I, R, T> {}
unsafe impl<I, R, T> Sync for WebSocketServer<I, R, T> {}

impl<I, R, T> WebSocketServer<I, R, T>
where
    I: Fn(SplitStream<WebSocketStream<TcpStream>>, Arc<Actor<WSPeer>>, T) -> R
        + Send
        + Sync
        + 'static,
    R: Future<Output = Result<()>> + Send + 'static,
    T: Clone + Send + 'static,
{
    /// 创建一个websocket server
    pub(crate) async fn new<A: ToSocketAddrs>(
        addr: A,
        input: I,
        connect_event: Option<ConnectEventType>,
        config: Option<WebSocketConfig>,
    ) -> Result<Arc<Actor<WebSocketServer<I, R, T>>>> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Arc::new(Actor::new(WebSocketServer {
            listener: Some(listener),
            connect_event,
            input_event: Arc::new(input),
            config,
            _phantom1: PhantomData::default(),
            _phantom2: PhantomData::default(),
        })))
    }

    /// 启动websocket server
    pub async fn start(&mut self, token: T) -> Result<JoinHandle<Result<()>>> {
        if let Some(listener) = self.listener.take() {
            let connect_event = self.connect_event.take();
            let input_event = self.input_event.clone();
            let config = self.config.clone();
            let join: JoinHandle<Result<()>> = tokio::spawn(async move {
                loop {
                    let (socket, addr) = listener.accept().await?;
                    if let Some(ref connect_event) = connect_event {
                        if !connect_event(addr) {
                            warn!("addr:{} not connect", addr);
                            continue;
                        }
                    }
                    trace!("start read:{}", addr);
                    let input = input_event.clone();
                    let peer_token = token.clone();
                    tokio::spawn(async move {
                        match accept_async_with_config(socket, config).await {
                            Ok(ws_stream) => {
                                let (sender, reader) = ws_stream.split();
                                let peer = WSPeer::new(addr, sender);
                                if let Err(err) = (*input)(reader, peer.clone(), peer_token).await {
                                    error!("input data error:{}", err);
                                }
                                if let Err(er) = peer.disconnect().await {
                                    debug!("disconnect client:{:?} err:{}", peer.addr(), er);
                                } else {
                                    debug!("{} disconnect", peer.addr())
                                }
                            }
                            Err(err) => {
                                error!(
                                    "ipaddress:{} init websocket error:{:?} disconnect!",
                                    addr, err
                                );
                            }
                        }
                    });
                }
            });

            return Ok(join);
        }
        bail!("not listener or repeat start")
    }
}

#[async_trait::async_trait]
pub trait IWebSocketServer<T> {
    async fn start(&self, token: T) -> Result<JoinHandle<Result<()>>>;
    async fn start_block(&self, token: T) -> Result<()>;
}

#[async_trait::async_trait]
impl<I, R, T> IWebSocketServer<T> for Actor<WebSocketServer<I, R, T>>
where
    I: Fn(SplitStream<WebSocketStream<TcpStream>>, Arc<Actor<WSPeer>>, T) -> R
        + Send
        + Sync
        + 'static,
    R: Future<Output = Result<()>> + Send + 'static,
    T: Clone + Send + Sync + 'static,
{
    async fn start(&self, token: T) -> Result<JoinHandle<Result<()>>> {
        self.inner_call(|inner| async move { inner.get_mut().start(token).await })
            .await
    }

    async fn start_block(&self, token: T) -> Result<()> {
        Self::start(self, token).await?.await??;
        Ok(())
    }
}
