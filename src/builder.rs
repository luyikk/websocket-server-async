use crate::peer::WSPeer;
use crate::websocket_server::{ConnectEventType, WebSocketServer};
use anyhow::Result;
use aqueue::Actor;
use futures_util::stream::SplitStream;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::WebSocketStream;

/// websocket server builder
pub struct Builder<I, R, A, T> {
    input: Option<I>,
    connect_event: Option<ConnectEventType>,
    addr: A,
    config: Option<WebSocketConfig>,
    _phantom1: PhantomData<R>,
    _phantom2: PhantomData<T>,
}

impl<I, R, A, T> Builder<I, R, A, T>
where
    I: Fn(SplitStream<WebSocketStream<TcpStream>>, Arc<Actor<WSPeer>>, T) -> R
        + Send
        + Sync
        + 'static,
    R: Future<Output = Result<()>> + Send + 'static,
    A: ToSocketAddrs,
    T: Clone + Send + 'static,
{
    pub fn new(addr: A) -> Builder<I, R, A, T> {
        Builder {
            input: None,
            connect_event: None,
            addr,
            config: None,
            _phantom1: PhantomData::default(),
            _phantom2: PhantomData::default(),
        }
    }

    /// 设置websocket server 输入事件
    pub fn set_input_event(mut self, f: I) -> Self {
        self.input = Some(f);
        self
    }

    /// 设置TCP server 连接事件
    pub fn set_connect_event(mut self, c: ConnectEventType) -> Self {
        self.connect_event = Some(c);
        self
    }

    /// 设置config
    pub fn set_config(mut self, config: WebSocketConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// 生成TCPSERVER,如果没有设置 tcp input 将报错
    pub async fn build(mut self) -> Arc<Actor<WebSocketServer<I, R, T>>> {
        if let Some(input) = self.input.take() {
            return WebSocketServer::new(self.addr, input, self.connect_event, self.config)
                .await
                .unwrap();
        }
        panic!("input event is no settings,please use set_input_event function set input event.");
    }
}
