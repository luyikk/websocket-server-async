#rust websocket tokio server frame.

# Examples Echo
```rust
use anyhow::Result;
use futures_util::StreamExt;
use log::*;
use websocket_server_async::{Builder, IPeer, IWebSocketServer};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();
    let websocket_server = Builder::new("0.0.0.0:8888")
        .set_connect_event(|addr| {
            info!("{} connect", addr);
            true
        })
        .set_input_event(|mut reader, peer, _| async move {
            while let Some(msg) = reader.next().await {
                let msg = msg?;
                if msg.is_text() || msg.is_binary() {
                    peer.send_message(msg).await?;
                }
            }
            Ok(())
        })
        .build()
        .await;
    websocket_server.start_block(()).await?;
    Ok(())
}


```