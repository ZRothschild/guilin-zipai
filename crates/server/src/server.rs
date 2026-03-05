use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures::{StreamExt, SinkExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use guilin_paizi_core::PlayerId;
use guilin_paizi_economy::EconomySystem;
use crate::room::GameRoom;
use crate::message::{ClientMessage, ServerMessage};
use crate::handler::MessageHandler;
use tracing::{info, error, warn};

pub struct GameServer {
    rooms: Arc<RwLock<HashMap<String, Arc<RwLock<GameRoom>>>>>,
    economy: Arc<RwLock<EconomySystem>>,
    connections: Arc<RwLock<HashMap<PlayerId, mpsc::UnboundedSender<ServerMessage>>>>,
    player_rooms: Arc<RwLock<HashMap<PlayerId, String>>>,
    handler: Arc<MessageHandler>,
}

impl GameServer {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            economy: Arc::new(RwLock::new(EconomySystem::default())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            player_rooms: Arc::new(RwLock::new(HashMap::new())),
            handler: Arc::new(MessageHandler::new()),
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
            let player_rooms = self.player_rooms.clone();
            let handler = self.handler.clone();
            
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, rooms, economy, connections, player_rooms, handler).await {
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
    player_rooms: Arc<RwLock<HashMap<PlayerId, String>>>,
    handler: Arc<MessageHandler>,
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
                            &player_rooms,
                            &handler,
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

    {
        let mut pr = player_rooms.write().await;
        if let Some(room_id) = pr.remove(&player_id) {
            let rooms_guard = rooms.read().await;
            if let Some(room) = rooms_guard.get(&room_id) {
                let mut room_guard = room.write().await;
                room_guard.remove_player(player_id);
                room_guard.set_player_online(player_id, false);
            }
        }
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
    player_rooms: &Arc<RwLock<HashMap<PlayerId, String>>>,
    handler: &Arc<MessageHandler>,
) {
    match msg {
        ClientMessage::JoinRoom { room_id } => {
            let rooms_guard = rooms.read().await;
            if let Some(room) = rooms_guard.get(&room_id) {
                let mut room_guard = room.write().await;
                if room_guard.add_player(player_id) {
                    room_guard.set_player_online(player_id, true);
                    
                    {
                        let mut pr = player_rooms.write().await;
                        pr.insert(player_id, room_id.clone());
                    }

                    let conns = connections.read().await;
                    if let Some(tx) = conns.get(&player_id) {
                        let _ = tx.send(ServerMessage::RoomJoined {
                            room_id: room_id.clone(),
                            player_id,
                        });
                    }

                    broadcast_to_room(rooms, &room_id, ServerMessage::PlayerJoined {
                        player_id,
                        name: format!("玩家{}", room_guard.players.len()),
                    }, connections).await;
                }
            }
        }
        ClientMessage::LeaveRoom { room_id } => {
            let rooms_guard = rooms.read().await;
            if let Some(room) = rooms_guard.get(&room_id) {
                let mut room_guard = room.write().await;
                room_guard.remove_player(player_id);
                
                {
                    let mut pr = player_rooms.write().await;
                    pr.remove(&player_id);
                }

                let conns = connections.read().await;
                if let Some(tx) = conns.get(&player_id) {
                    let _ = tx.send(ServerMessage::RoomLeft {
                        room_id: room_id.clone(),
                    });
                }
            }
        }
        ClientMessage::Ready { room_id } => {
            let rooms_guard = rooms.read().await;
            if let Some(room) = rooms_guard.get(&room_id) {
                let mut room_guard = room.write().await;
                room_guard.set_player_ready(player_id, true);
                
                broadcast_to_room(rooms, &room_id, ServerMessage::PlayerReady {
                    player_id,
                }, connections).await;
            }
        }
        ClientMessage::StartGame { room_id } => {
            let rooms_guard = rooms.read().await;
            if let Some(room) = rooms_guard.get(&room_id) {
                let mut room_guard = room.write().await;
                
                if room_guard.can_start() {
                    room_guard.start_game();
                    
                    let dealer = room_guard.game_state.players.get(room_guard.game_state.dealer_idx)
                        .map(|p| p.id);
                    
                    broadcast_to_room(rooms, &room_id, ServerMessage::GameStarted {
                        dealer: dealer.unwrap_or(player_id),
                    }, connections).await;

                    notify_game_state(rooms, &room_id, &connections).await;
                }
            }
        }
        ClientMessage::PlayCard { room_id, card_idx } => {
            let rooms_guard = rooms.read().await;
            if let Some(room) = rooms_guard.get(&room_id) {
                let mut room_guard = room.write().await;
                
                if !room_guard.can_player_action(player_id) {
                    let conns = connections.read().await;
                    if let Some(tx) = conns.get(&player_id) {
                        let _ = tx.send(ServerMessage::Error {
                            message: "还没轮到你出牌".to_string(),
                        });
                    }
                    return;
                }

                match room_guard.play_card(player_id, card_idx) {
                    Ok(()) => {
                        if let Some(card) = room_guard.get_player_hand(player_id)
                            .and_then(|h| h.get(card_idx)) {
                        broadcast_to_room(rooms, &room_id, ServerMessage::CardPlayed {
                            player_id,
                            card: *card,
                        }, connections).await;
                        }
                        notify_game_state(rooms, &room_id, &connections).await;
                        check_game_over(rooms, &room_id, connections).await;
                    }
                    Err(e) => {
                        let conns = connections.read().await;
                        if let Some(tx) = conns.get(&player_id) {
                            let _ = tx.send(ServerMessage::Error {
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
        }
        ClientMessage::Chi { room_id, card_indices } => {
            let rooms_guard = rooms.read().await;
            if let Some(room) = rooms_guard.get(&room_id) {
                let mut room_guard = room.write().await;
                
                if !room_guard.can_player_action(player_id) {
                    let conns = connections.read().await;
                    if let Some(tx) = conns.get(&player_id) {
                        let _ = tx.send(ServerMessage::Error {
                            message: "还没轮到你操作".to_string(),
                        });
                    }
                    return;
                }

                match room_guard.chi(player_id, card_indices.clone()) {
                    Ok(()) => {
                        notify_game_state(rooms, &room_id, &connections).await;
                    }
                    Err(e) => {
                        let conns = connections.read().await;
                        if let Some(tx) = conns.get(&player_id) {
                            let _ = tx.send(ServerMessage::Error {
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
        }
        ClientMessage::Peng { room_id, card_idx } => {
            let rooms_guard = rooms.read().await;
            if let Some(room) = rooms_guard.get(&room_id) {
                let mut room_guard = room.write().await;
                
                if !room_guard.can_player_action(player_id) {
                    let conns = connections.read().await;
                    if let Some(tx) = conns.get(&player_id) {
                        let _ = tx.send(ServerMessage::Error {
                            message: "还没轮到你操作".to_string(),
                        });
                    }
                    return;
                }

                match room_guard.peng(player_id, card_idx) {
                    Ok(()) => {
                        notify_game_state(rooms, &room_id, &connections).await;
                    }
                    Err(e) => {
                        let conns = connections.read().await;
                        if let Some(tx) = conns.get(&player_id) {
                            let _ = tx.send(ServerMessage::Error {
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
        }
        ClientMessage::Sao { room_id, card_idx } => {
            let rooms_guard = rooms.read().await;
            if let Some(room) = rooms_guard.get(&room_id) {
                let mut room_guard = room.write().await;
                
                if !room_guard.can_player_action(player_id) {
                    let conns = connections.read().await;
                    if let Some(tx) = conns.get(&player_id) {
                        let _ = tx.send(ServerMessage::Error {
                            message: "还没轮到你操作".to_string(),
                        });
                    }
                    return;
                }

                match room_guard.sao(player_id, card_idx) {
                    Ok(()) => {
                        notify_game_state(rooms, &room_id, &connections).await;
                    }
                    Err(e) => {
                        let conns = connections.read().await;
                        if let Some(tx) = conns.get(&player_id) {
                            let _ = tx.send(ServerMessage::Error {
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
        }
        ClientMessage::Hu { room_id } => {
            let rooms_guard = rooms.read().await;
            if let Some(room) = rooms_guard.get(&room_id) {
                let mut room_guard = room.write().await;
                
                match room_guard.hu(player_id) {
                    Ok(win_result) => {
                        broadcast_to_room(rooms, &room_id, ServerMessage::PlayerHu {
                            player_id,
                            is_zimo: win_result.is_zimo,
                        }, connections).await;
                        check_game_over(rooms, &room_id, connections).await;
                    }
                    Err(e) => {
                        let conns = connections.read().await;
                        if let Some(tx) = conns.get(&player_id) {
                            let _ = tx.send(ServerMessage::Error {
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
        }
        ClientMessage::Pass { room_id } => {
            let rooms_guard = rooms.read().await;
            if let Some(room) = rooms_guard.get(&room_id) {
                let mut room_guard = room.write().await;
                
                if !room_guard.can_player_action(player_id) {
                    return;
                }

                match room_guard.pass(player_id) {
                    Ok(()) => {
                        notify_game_state(rooms, &room_id, &connections).await;
                    }
                    Err(e) => {
                        let conns = connections.read().await;
                        if let Some(tx) = conns.get(&player_id) {
                            let _ = tx.send(ServerMessage::Error {
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
        }
        ClientMessage::UseSkill { room_id, skill_id, target } => {
            let rooms_guard = rooms.read().await;
            if let Some(room) = rooms_guard.get(&room_id) {
                let mut room_guard = room.write().await;
                
                if let Some(response) = handler.handle_message(
                    ClientMessage::UseSkill { room_id: room_id.clone(), skill_id, target },
                    &mut room_guard,
                    player_id,
                ).await {
                    let conns = connections.read().await;
                    if let Some(tx) = conns.get(&player_id) {
                        let _ = tx.send(response);
                    }
                    
                    broadcast_to_room(rooms, &room_id, ServerMessage::SkillUsed {
                        player_id,
                        skill_name: "技能".to_string(),
                        effect: serde_json::json!({}),
                    }, connections).await;
                }
            }
        }
        ClientMessage::Chat { room_id, message } => {
            broadcast_to_room(rooms, &room_id, ServerMessage::ChatMessage {
                player_id,
                message,
            }, connections).await;
        }
        _ => {}
    }
}

impl Default for GameServer {
    fn default() -> Self {
        Self::new()
    }
}

async fn broadcast_to_room(
    rooms: &Arc<RwLock<HashMap<String, Arc<RwLock<GameRoom>>>>>,
    room_id: &str,
    msg: ServerMessage,
    connections: &Arc<RwLock<HashMap<PlayerId, mpsc::UnboundedSender<ServerMessage>>>>,
) {
    let rooms_guard = rooms.read().await;
    if let Some(room) = rooms_guard.get(room_id) {
        let room_guard = room.read().await;
        let conns = connections.read().await;
        
        for player in &room_guard.players {
            if let Some(tx) = conns.get(&player.id) {
                let _ = tx.send(msg.clone());
            }
        }
    }
}

async fn notify_game_state(
    rooms: &Arc<RwLock<HashMap<String, Arc<RwLock<GameRoom>>>>>,
    room_id: &str,
    connections: &Arc<RwLock<HashMap<PlayerId, mpsc::UnboundedSender<ServerMessage>>>>,
) {
    let rooms_guard = rooms.read().await;
    if let Some(room) = rooms_guard.get(room_id) {
        let room_guard = room.read().await;
        let state_view = room_guard.get_game_state_view();
        
        let state_json = serde_json::to_string(&state_view).unwrap_or_default();
        
        let conns = connections.read().await;
        for player in &room_guard.players {
            if let Some(tx) = conns.get(&player.id) {
                let _ = tx.send(ServerMessage::GameStateUpdate {
                    state: state_json.clone(),
                });
                
                if room_guard.can_player_action(player.id) {
                    let _ = tx.send(ServerMessage::YourTurn);
                }
            }
        }
    }
}

async fn check_game_over(
    rooms: &Arc<RwLock<HashMap<String, Arc<RwLock<GameRoom>>>>>,
    room_id: &str,
    connections: &Arc<RwLock<HashMap<PlayerId, mpsc::UnboundedSender<ServerMessage>>>>,
) {
    let rooms_guard = rooms.read().await;
    if let Some(room) = rooms_guard.get(room_id) {
        let room_guard = room.read().await;
        
        if room_guard.is_game_over() {
            let winner = room_guard.get_winner();
            
            let conns = connections.read().await;
            for player in &room_guard.players {
                if let Some(tx) = conns.get(&player.id) {
                    let _ = tx.send(ServerMessage::GameEnded { winner });
                }
            }
        }
    }
}
