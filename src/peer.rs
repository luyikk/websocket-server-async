use std::net::SocketAddr;
use futures_util::stream::SplitSink;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

pub struct WSPeer<C>{
    pub addr: SocketAddr,
    pub sender: Option<SplitSink<WebSocketStream<C>, Message>>,
}

