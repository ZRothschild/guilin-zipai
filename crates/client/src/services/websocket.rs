use yew::platform::spawn_local;
use web_sys::WebSocket;
use serde::{Deserialize, Serialize};

pub struct WebSocketService {
    ws: Option<WebSocket>,
    on_message: Option<Box<dyn Fn(String)>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    pub action: String,
    pub data: serde_json::Value,
}

impl WebSocketService {
    pub fn new() -> Self {
        Self { 
            ws: None, 
            on_message: None,
        }
    }

    pub fn connect(&mut self, url: &str) -> Result<(), String> {
        let ws = WebSocket::new(url).map_err(|e| format!("连接失败: {:?}", e))?;
        self.ws = Some(ws);
        Ok(())
    }

    pub fn set_on_message(&mut self, callback: Box<dyn Fn(String)>) {
        self.on_message = Some(callback);
    }

    pub fn send(&self, message: &WsMessage) -> Result<(), String> {
        if let Some(ws) = &self.ws {
            let json = serde_json::to_string(message).map_err(|e| e.to_string())?;
            ws.send_with_str(&json).map_err(|e| format!("发送失败: {:?}", e))?;
            Ok(())
        } else {
            Err("未连接到服务器".to_string())
        }
    }

    pub fn send_async(&self, message: &WsMessage) {
        if let Some(ws) = &self.ws {
            let json = serde_json::to_string(message).unwrap_or_default();
            let ws_clone = ws.clone();
            spawn_local(async move {
                let _ = ws_clone.send_with_str(&json);
            });
        }
    }
}

impl Default for WebSocketService {
    fn default() -> Self {
        Self::new()
    }
}
