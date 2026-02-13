use yew::platform::spawn_local;
use futures::{SinkExt, StreamExt};
use wasm_bindgen_futures::spawn_local;
use gloo::net::websocket::{futures::WebSocket, Message};
use serde::{Deserialize, Serialize};

pub struct WebSocketService {
    ws: Option<WebSocket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    pub action: String,
    pub data: serde_json::Value,
}

impl WebSocketService {
    pub fn new() -> Self {
        Self { ws: None }
    }

    pub fn connect(&mut self, url: &str) -> Result<(), String> {
        match WebSocket::open(url) {
            Ok(ws) => {
                self.ws = Some(ws);
                Ok(())
            }
            Err(e) => Err(format!("连接失败: {:?}", e)),
        }
    }

    pub fn send(&mut self, message: &WsMessage) -> Result<(), String> {
        if let Some(ws) = &mut self.ws {
            let json = serde_json::to_string(message).map_err(|e| e.to_string())?;
            spawn_local(async move {
                let _ = ws.send(Message::Text(json)).await;
            });
            Ok(())
        } else {
            Err("未连接到服务器".to_string())
        }
    }
}

impl Default for WebSocketService {
    fn default() -> Self {
        Self::new()
    }
}
