use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures::{StreamExt, SinkExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use guilin_paizi_core::{PlayerId, GameState};
use guilin_paizi_economy::EconomySystem;
use crate::room::GameRoom;
use crate::message::{ClientMessage, ServerMessage};
use tracing::{info, error, warn};

pub struct GameServer {
    rooms: Arc<RwLock<HashMap<String, Arc<RwLock<GameRoom>>>>>,
    economy: Arc<RwLock<EconomySystem>>,
    connections: Arc<RwLock<HashMap<PlayerId, mpsc::UnboundedSender<ServerMessage>>>>,
}

impl GameServer {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            economy: Arc::new(RwLock::new(EconomySystem::default())),
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn run(&self, addr: &str) -> anyhow::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        info!("游戏服务器启动于 {}", addr);

        while let Ok((stream, peer_addr)) = listener.accept().await {
            info!("新连接: {}", peer_addr);
            
            let rooms = self.rooms.clone();
            let economy = self.economy.clone();
            let connections = self.connections.clone();
            
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, rooms, economy, connections).await {
                    error!("连接处理错误: {}", e);
                }
            });
        }

        Ok(())
    }

    pub async fn create_room(&self, room_id: String, max_players: usize) {
        let room = GameRoom::new(room_id.clone(), max_players);
        let mut rooms = self.rooms.write().await;
        rooms.insert(room_id, Arc::new(RwLock::new(room)));
    }

    pub async fn get_room(&self, room_id: &str) -> Option<Arc<RwLock<GameRoom>>> {
        let rooms = self.rooms.read().await;
        rooms.get(room_id).cloned()
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    rooms: Arc<RwLock<HashMap<String, Arc<RwLock<GameRoom>>>>>,
    _economy: Arc<RwLock<EconomySystem>>,
    connections: Arc<RwLock<HashMap<PlayerId, mpsc::UnboundedSender<ServerMessage>>>>,
) -> anyhow::Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    let player_id = PlayerId::new();
    let (tx, mut rx) = mpsc::unbounded_channel();
    
    {
        let mut conns = connections.write().await;
        conns.insert(player_id, tx);
    }

    let welcome = ServerMessage::Welcome {
        player_id,
        message: "欢迎来到桂林字牌".to_string(),
    };
    ws_sender.send(tokio_tungstenite::tungstenite::Message::Text(
        serde_json::to_string(&welcome)?
    )).await?;

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(text) = serde_json::to_string(&msg) {
                if ws_sender.send(tokio_tungstenite::tungstenite::Message::Text(text)).await.is_err() {
                    break;
                }
            }
        }
    });

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        handle_client_message(
                            player_id,
                            client_msg,
                            &rooms,
                            &connections,
                        ).await;
                    }
                    Err(e) => {
                        warn!("解析消息失败: {}", e);
                    }
                }
            }
            Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                break;
            }
            Err(e) => {
                error!("WebSocket错误: {}", e);
                break;
            }
            _ => {}
        }
    }

    {
        let mut conns = connections.write().await;
        conns.remove(&player_id);
    }

    send_task.abort();
    info!("玩家 {:?} 断开连接", player_id);

    Ok(())
}

async fn handle_client_message(
    player_id: PlayerId,
    msg: ClientMessage,
    rooms: &Arc<RwLock<HashMap<String, Arc<RwLock<GameRoom>>>>>,
    connections: &Arc<RwLock<HashMap<PlayerId, mpsc::UnboundedSender<ServerMessage>>>>,
) {
    match msg {
        ClientMessage::JoinRoom { room_id } => {
            let rooms_guard = rooms.read().await;
            if let Some(room) = rooms_guard.get(&room_id) {
                let mut room_guard = room.write().await;
                room_guard.add_player(player_id);
                
                let conns = connections.read().await;
                if let Some(tx) = conns.get(&player_id) {
                    let _ = tx.send(ServerMessage::RoomJoined {
                        room_id: room_id.clone(),
                        player_id,
                    });
                }
            }
        }
        ClientMessage::LeaveRoom { room_id } => {
            let rooms_guard = rooms.read().await;
            if let Some(room) = rooms_guard.get(&room_id) {
                let mut room_guard = room.write().await;
                room_guard.remove_player(player_id);
            }
        }
        ClientMessage::Ready { room_id } => {
            let rooms_guard = rooms.read().await;
            if let Some(room) = rooms_guard.get(&room_id) {
                let mut room_guard = room.write().await;
                room_guard.set_player_ready(player_id, true);
            }
        }
        ClientMessage::PlayCard { room_id, card_idx } => {
            let rooms_guard = rooms.read().await;
            if let Some(room) = rooms_guard.get(&room_id) {
                let mut room_guard = room.write().await;
                if let Err(e) = room_guard.play_card(player_id, card_idx) {
                    let conns = connections.read().await;
                    if let Some(tx) = conns.get(&player_id) {
                        let _ = tx.send(ServerMessage::Error {
                            message: e.to_string(),
                        });
                    }
                }
            }
        }
        _ => {}
    }
}

impl Default for GameServer {
    fn default() -> Self {
        Self::new()
    }
}
