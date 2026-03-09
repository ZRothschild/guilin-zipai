use web_sys::{WebSocket, MessageEvent};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use serde::{Deserialize, Serialize};

pub struct WebSocketService {
    ws: Option<WebSocket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    Authenticate { token: String },
    // ... logic mirror of ClientMessage
}

impl WebSocketService {
    pub fn new() -> Self {
        Self { ws: None }
    }

    pub fn connect_with_cb<F>(&mut self, url: &str, on_msg: F) -> Result<(), String> 
    where F: Fn(String) + 'static 
    {
        let ws = WebSocket::new(url).map_err(|e| format!("连接失败: {:?}", e))?;
        
        let on_msg = Arc::new(on_msg);
        let callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Some(text) = e.data().as_string() {
                on_msg(text);
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        
        ws.set_onmessage(Some(callback.as_ref().unchecked_ref()));
        callback.forget();

        self.ws = Some(ws);
        Ok(())
    }

    pub fn send_text(&self, text: &str) -> Result<(), String> {
        if let Some(ws) = &self.ws {
            ws.send_with_str(text).map_err(|e| format!("发送失败: {:?}", e))?;
            Ok(())
        } else {
            Err("未连接".into())
        }
    }
}

use std::sync::Arc;

impl Default for WebSocketService {
    fn default() -> Self {
        Self::new()
    }
}
