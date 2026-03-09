use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures::{StreamExt, SinkExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use futures::future::BoxFuture;
use guilin_paizi_core::PlayerId;
use guilin_paizi_economy::EconomySystem;
use crate::room::GameRoom;
use crate::message::{ClientMessage, ServerMessage};
use crate::handler::MessageHandler;
use crate::bot::BotLogic;
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
    economy: Arc<RwLock<EconomySystem>>,
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

    // 为新连接注册经济系统账户和等级系统
    {
        let mut econ = economy.write().await;
        econ.currency.register_player(player_id, 10000);
        econ.ranking.register_player(player_id);
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
                            rooms.clone(),
                            connections.clone(),
                            player_rooms.clone(),
                            handler.clone(),
                            economy.clone(),
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

fn handle_client_message(
    player_id: PlayerId,
    msg: ClientMessage,
    rooms: Arc<RwLock<HashMap<String, Arc<RwLock<GameRoom>>>>>,
    connections: Arc<RwLock<HashMap<PlayerId, mpsc::UnboundedSender<ServerMessage>>>>,
    player_rooms: Arc<RwLock<HashMap<PlayerId, String>>>,
    handler: Arc<MessageHandler>,
    economy: Arc<RwLock<EconomySystem>>,
) -> BoxFuture<'static, ()> {
    Box::pin(async move {
    match msg {
        ClientMessage::CreateRoom { max_players } => {
            let room_id = format!("{:04}", rand::random::<u16>() % 10000);
            let room = Arc::new(RwLock::new(GameRoom::new(room_id.clone(), max_players)));
            
            {
                let mut rooms_guard = rooms.write().await;
                rooms_guard.insert(room_id.clone(), room.clone());
            }

            {
                let conns = connections.read().await;
                if let Some(tx) = conns.get(&player_id) {
                    let _ = tx.send(ServerMessage::RoomCreated { room_id: room_id.clone() });
                }
            }
            
            // 自动加入
            perform_join_room(player_id, room_id, rooms.clone(), connections.clone(), player_rooms.clone()).await;
        }
        ClientMessage::JoinRoom { room_id } => {
            perform_join_room(player_id, room_id, rooms.clone(), connections.clone(), player_rooms.clone()).await;
        }
        ClientMessage::AddBot { room_id } => {
            let room = {
                let rooms_guard = rooms.read().await;
                rooms_guard.get(&room_id).cloned()
            };
            if let Some(room) = room {
                let bot_msg = {
                    let mut room_guard = room.write().await;
                    if let Some(bot_id) = room_guard.add_bot() {
                        Some(ServerMessage::PlayerJoined {
                            player_id: bot_id,
                            name: format!("机器人{}", room_guard.players.len()),
                            is_bot: true,
                        })
                    } else {
                        None
                    }
                };

                if let Some(msg) = bot_msg {
                    broadcast_to_room(rooms.clone(), room_id.clone(), msg, connections.clone()).await;
                    notify_room_info(rooms.clone(), room_id.clone(), connections.clone()).await;
                }
            }
        }
        ClientMessage::LeaveRoom { room_id } => {
            let room = {
                let rooms_guard = rooms.read().await;
                rooms_guard.get(&room_id).cloned()
            };
            if let Some(room) = room {
                {
                    let mut room_guard = room.write().await;
                    room_guard.remove_player(player_id);
                }
                
                {
                    let mut pr = player_rooms.write().await;
                    pr.remove(&player_id);
                }

                broadcast_to_room(rooms.clone(), room_id.clone(), ServerMessage::PlayerLeft {
                    player_id,
                }, connections.clone()).await;

                notify_room_info(rooms.clone(), room_id.clone(), connections.clone()).await;

                let conns = connections.read().await;
                if let Some(tx) = conns.get(&player_id) {
                    let _ = tx.send(ServerMessage::RoomLeft {
                        room_id: room_id.clone(),
                    });
                }
            }
        }
        ClientMessage::Ready { room_id } => {
            let room = {
                let rooms_guard = rooms.read().await;
                rooms_guard.get(&room_id).cloned()
            };
            if let Some(room) = room {
                let start_info = {
                    let mut room_guard = room.write().await;
                    room_guard.set_player_ready(player_id, true);
                    if room_guard.state == crate::room::RoomState::Playing {
                        Some(room_guard.game_state.players[room_guard.game_state.dealer_idx].id)
                    } else {
                        None
                    }
                };
                
                let ready_msg = ServerMessage::PlayerReady { player_id };
                broadcast_to_room(rooms.clone(), room_id.clone(), ready_msg, connections.clone()).await;
                notify_room_info(rooms.clone(), room_id.clone(), connections.clone()).await;

                if let Some(dealer_id) = start_info {
                    broadcast_to_room(rooms.clone(), room_id.clone(), ServerMessage::GameStarted {
                        dealer: dealer_id,
                    }, connections.clone()).await;

                    notify_game_state(rooms.clone(), room_id.clone(), connections.clone()).await;
                    
                    trigger_bot_actions(room_id.clone(), rooms.clone(), connections.clone(), player_rooms.clone(), handler.clone(), economy.clone()).await;
                }
            }
        }
        ClientMessage::StartGame { room_id } => {
            let room = {
                let rooms_guard = rooms.read().await;
                rooms_guard.get(&room_id).cloned()
            };
            if let Some(room) = room {
                let start_info = {
                    let mut room_guard = room.write().await;
                    
                    // 如果房人不满，自动补充机器人
                    while room_guard.players.len() < room_guard.max_players {
                        if let Some(bot_id) = room_guard.add_bot() {
                            let bot_name = format!("陪玩机器人{}", room_guard.players.len());
                            let msg = ServerMessage::PlayerJoined {
                                player_id: bot_id,
                                name: bot_name,
                                is_bot: true,
                            };
                            // 这里我们暂时在外面广播，因为我们需要 connections 的锁
                            // 但为了简化，我们可以先把消息存起来
                        } else {
                            break;
                        }
                    }
                    
                    // 强制所有人准备
                    let player_ids: Vec<PlayerId> = room_guard.players.iter().map(|p| p.id).collect();
                    for pid in player_ids {
                        room_guard.set_player_ready(pid, true);
                    }

                    if room_guard.state == crate::room::RoomState::Playing {
                        let dealer_id = room_guard.game_state.players.get(room_guard.game_state.dealer_idx)
                            .map(|p| p.id)
                            .unwrap_or(player_id);
                        Some(dealer_id)
                    } else if room_guard.can_start() {
                        room_guard.start_game();
                        let dealer_id = room_guard.game_state.players.get(room_guard.game_state.dealer_idx)
                            .map(|p| p.id)
                            .unwrap_or(player_id);
                        Some(dealer_id)
                    } else {
                        None
                    }
                };

                if let Some(dealer_id) = start_info {
                    // 重新广播房间信息，因为加了机器人
                    notify_room_info(rooms.clone(), room_id.clone(), connections.clone()).await;
                    
                    broadcast_to_room(rooms.clone(), room_id.clone(), ServerMessage::GameStarted {
                        dealer: dealer_id,
                    }, connections.clone()).await;

                    notify_game_state(rooms.clone(), room_id.clone(), connections.clone()).await;
                    
                    // 触发首轮机器人动作
                    trigger_bot_actions(room_id.clone(), rooms.clone(), connections.clone(), player_rooms.clone(), handler.clone(), economy.clone()).await;
                }
            }
        }
        ClientMessage::PlayCard { room_id, card_idx } => {
            let room = {
                let rooms_guard = rooms.read().await;
                rooms_guard.get(&room_id).cloned()
            };
            if let Some(room) = room {
                let result = {
                    let mut room_guard = room.write().await;
                    if !room_guard.can_player_action(player_id) {
                        (Err(anyhow::anyhow!("还没轮到你出牌")), None)
                    } else {
                        match room_guard.play_card(player_id, card_idx) {
                            Ok(()) => {
                                let card = room_guard.get_player_hand(player_id)
                                    .and_then(|h| h.get(card_idx))
                                    .cloned();
                                (Ok(()), card)
                            }
                            Err(e) => (Err(anyhow::anyhow!(e.to_string())), None),
                        }
                    }
                };

                match result {
                    (Ok(()), Some(card)) => {
                        broadcast_to_room(rooms.clone(), room_id.clone(), ServerMessage::CardPlayed {
                            player_id,
                            card,
                        }, connections.clone()).await;
                        notify_game_state(rooms.clone(), room_id.clone(), connections.clone()).await;
                        check_game_over(rooms.clone(), room_id.clone(), connections.clone(), economy.clone()).await;
                        trigger_bot_actions(room_id.clone(), rooms.clone(), connections.clone(), player_rooms.clone(), handler.clone(), economy.clone()).await;
                    }
                    (Err(e), _) => {
                        let conns = connections.read().await;
                        if let Some(tx) = conns.get(&player_id) {
                            let _ = tx.send(ServerMessage::Error {
                                message: e.to_string(),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
        ClientMessage::Chi { room_id, card_indices } => {
            let room = {
                let rooms_guard = rooms.read().await;
                rooms_guard.get(&room_id).cloned()
            };
            if let Some(room) = room {
                let result = {
                    let mut room_guard = room.write().await;
                    if !room_guard.can_player_action(player_id) {
                        Err(anyhow::anyhow!("还没轮到你操作"))
                    } else {
                        room_guard.chi(player_id, card_indices.clone()).map_err(|e| anyhow::anyhow!(e.to_string()))
                    }
                };

                match result {
                    Ok(()) => {
                        notify_game_state(rooms.clone(), room_id.clone(), connections.clone()).await;
                        trigger_bot_actions(room_id.clone(), rooms.clone(), connections.clone(), player_rooms.clone(), handler.clone(), economy.clone()).await;
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
            let room = {
                let rooms_guard = rooms.read().await;
                rooms_guard.get(&room_id).cloned()
            };
            if let Some(room) = room {
                let result = {
                    let mut room_guard = room.write().await;
                    if !room_guard.can_player_action(player_id) {
                        Err(anyhow::anyhow!("还没轮到你操作"))
                    } else {
                        room_guard.peng(player_id, card_idx).map_err(|e| anyhow::anyhow!(e.to_string()))
                    }
                };

                match result {
                    Ok(()) => {
                        notify_game_state(rooms.clone(), room_id.clone(), connections.clone()).await;
                        trigger_bot_actions(room_id.clone(), rooms.clone(), connections.clone(), player_rooms.clone(), handler.clone(), economy.clone()).await;
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
            let room = {
                let rooms_guard = rooms.read().await;
                rooms_guard.get(&room_id).cloned()
            };
            if let Some(room) = room {
                let result = {
                    let mut room_guard = room.write().await;
                    if !room_guard.can_player_action(player_id) {
                        Err(anyhow::anyhow!("还没轮到你操作"))
                    } else {
                        room_guard.sao(player_id, card_idx).map_err(|e| anyhow::anyhow!(e.to_string()))
                    }
                };

                match result {
                    Ok(()) => {
                        notify_game_state(rooms.clone(), room_id.clone(), connections.clone()).await;
                        trigger_bot_actions(room_id.clone(), rooms.clone(), connections.clone(), player_rooms.clone(), handler.clone(), economy.clone()).await;
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
            let room = {
                let rooms_guard = rooms.read().await;
                rooms_guard.get(&room_id).cloned()
            };
            if let Some(room) = room {
                let result = {
                    let mut room_guard = room.write().await;
                    room_guard.hu(player_id).map_err(|e| anyhow::anyhow!(e.to_string()))
                };

                match result {
                    Ok(win_result) => {
                        broadcast_to_room(rooms.clone(), room_id.clone(), ServerMessage::PlayerHu {
                            player_id,
                            is_zimo: win_result.is_zimo,
                        }, connections.clone()).await;
                        check_game_over(rooms.clone(), room_id.clone(), connections.clone(), economy.clone()).await;
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
            let room = {
                let rooms_guard = rooms.read().await;
                rooms_guard.get(&room_id).cloned()
            };
            if let Some(room) = room {
                let result = {
                    let mut room_guard = room.write().await;
                    if !room_guard.can_player_action(player_id) {
                        Err(anyhow::anyhow!("还没轮到你操作"))
                    } else {
                        room_guard.pass(player_id).map_err(|e| anyhow::anyhow!(e.to_string()))
                    }
                };

                match result {
                    Ok(()) => {
                        notify_game_state(rooms.clone(), room_id.clone(), connections.clone()).await;
                        trigger_bot_actions(room_id.clone(), rooms.clone(), connections.clone(), player_rooms.clone(), handler.clone(), economy.clone()).await;
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
            let room = {
                let rooms_guard = rooms.read().await;
                rooms_guard.get(&room_id).cloned()
            };
            if let Some(room) = room {
                let response = {
                    let mut room_guard = room.write().await;
                    handler.handle_message(
                        ClientMessage::UseSkill { room_id: room_id.clone(), skill_id, target },
                        &mut room_guard,
                        player_id,
                    ).await
                };

                if let Some(msg) = response {
                    let conns = connections.read().await;
                    if let Some(tx) = conns.get(&player_id) {
                        let _ = tx.send(msg.clone());
                    }
                    
                    if let ServerMessage::SkillUsed { .. } = msg {
                        broadcast_to_room(rooms.clone(), room_id.clone(), msg, connections.clone()).await;
                    }
                }
            }
        }
        ClientMessage::Chat { room_id, message } => {
            broadcast_to_room(rooms.clone(), room_id.clone(), ServerMessage::ChatMessage {
                player_id,
                message,
            }, connections.clone()).await;
        }
        _ => {}
    }
    })
}

fn trigger_bot_actions(
    room_id: String,
    rooms: Arc<RwLock<HashMap<String, Arc<RwLock<GameRoom>>>>>,
    connections: Arc<RwLock<HashMap<PlayerId, mpsc::UnboundedSender<ServerMessage>>>>,
    player_rooms: Arc<RwLock<HashMap<PlayerId, String>>>,
    handler: Arc<MessageHandler>,
    economy: Arc<RwLock<EconomySystem>>,
) -> BoxFuture<'static, ()> {
    Box::pin(async move {
    let room = {
        let rooms_guard = rooms.read().await;
        rooms_guard.get(&room_id).cloned()
    };
    if let Some(room) = room {
        loop {
            let room_guard = room.read().await;
            let mut bot_to_act = None;
            
            for bot_id in room_guard.get_bot_ids() {
                if let Some(action) = BotLogic::take_action(bot_id, &room_guard) {
                    bot_to_act = Some((bot_id, action));
                    break;
                }
            }
            drop(room_guard);

            if let Some((bot_id, action)) = bot_to_act {
                // 使用 tokio::spawn 处理机器人动作，打破 async fn 递归调用链
                let rooms = rooms.clone();
                let connections = connections.clone();
                let player_rooms = player_rooms.clone();
                let handler = handler.clone();
                let economy = economy.clone();
                
                tokio::spawn(async move {
                    // 模拟思考时间
                    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                    
                    handle_client_message(
                        bot_id,
                        action,
                        rooms,
                        connections,
                        player_rooms,
                        handler,
                        economy
                    ).await;
                });
            } else {
                break;
            }
        }
    }
    })
}

impl Default for GameServer {
    fn default() -> Self {
        Self::new()
    }
}

async fn broadcast_to_room(
    rooms: Arc<RwLock<HashMap<String, Arc<RwLock<GameRoom>>>>>,
    room_id: String,
    msg: ServerMessage,
    connections: Arc<RwLock<HashMap<PlayerId, mpsc::UnboundedSender<ServerMessage>>>>,
) {
    let room = {
        let rooms_guard = rooms.read().await;
        rooms_guard.get(&room_id).cloned()
    };

    if let Some(room) = room {
        let player_ids: Vec<PlayerId> = {
            let room_guard = room.read().await;
            room_guard.players.iter().map(|p| p.id).collect()
        };

        let conns = connections.read().await;
        for id in player_ids {
            if let Some(tx) = conns.get(&id) {
                let _ = tx.send(msg.clone());
            }
        }
    }
}

async fn perform_join_room(
    player_id: PlayerId,
    room_id: String,
    rooms: Arc<RwLock<HashMap<String, Arc<RwLock<GameRoom>>>>>,
    connections: Arc<RwLock<HashMap<PlayerId, mpsc::UnboundedSender<ServerMessage>>>>,
    player_rooms: Arc<RwLock<HashMap<PlayerId, String>>>,
) {
    let room = {
        let rooms_guard = rooms.read().await;
        rooms_guard.get(&room_id).cloned()
    };

    if let Some(room) = room {
        let (joined, room_count) = {
            let mut room_guard = room.write().await;
            if room_guard.add_player(player_id) {
                room_guard.set_player_online(player_id, true);
                (true, room_guard.players.len())
            } else {
                (false, 0)
            }
        };

        if joined {
            {
                let mut pr = player_rooms.write().await;
                pr.insert(player_id, room_id.clone());
            }

            {
                let conns = connections.read().await;
                if let Some(tx) = conns.get(&player_id) {
                    let _ = tx.send(ServerMessage::RoomJoined {
                        room_id: room_id.clone(),
                        player_id,
                    });
                }
            }

            let join_msg = ServerMessage::PlayerJoined {
                player_id,
                name: format!("玩家{}", room_count),
                is_bot: false,
            };
            
            broadcast_to_room(rooms.clone(), room_id.clone(), join_msg, connections.clone()).await;
            notify_room_info(rooms.clone(), room_id.clone(), connections.clone()).await;
        }
    }
}

async fn notify_room_info(
    rooms: Arc<RwLock<HashMap<String, Arc<RwLock<GameRoom>>>>>,
    room_id: String,
    connections: Arc<RwLock<HashMap<PlayerId, mpsc::UnboundedSender<ServerMessage>>>>,
) {
    let room = {
        let rooms_guard = rooms.read().await;
        rooms_guard.get(&room_id).cloned()
    };

    if let Some(room) = room {
        let (room_info, player_ids) = {
            let room_guard = room.read().await;
            (
                room_guard.get_room_info(),
                room_guard.players.iter().map(|p| p.id).collect::<Vec<_>>()
            )
        };
        
        let info_json = serde_json::to_string(&room_info).unwrap_or_default();
        let msg = ServerMessage::RoomUpdate { room_info: info_json };
        
        let conns = connections.read().await;
        for id in player_ids {
            if let Some(tx) = conns.get(&id) {
                let _ = tx.send(msg.clone());
            }
        }
    }
}

async fn notify_game_state(
    rooms: Arc<RwLock<HashMap<String, Arc<RwLock<GameRoom>>>>>,
    room_id: String,
    connections: Arc<RwLock<HashMap<PlayerId, mpsc::UnboundedSender<ServerMessage>>>>,
) {
    let room = {
        let rooms_guard = rooms.read().await;
        rooms_guard.get(&room_id).cloned()
    };

    if let Some(room) = room {
        let (state_data, player_info) = {
            let room_guard = room.read().await;
            (
                room_guard.game_state.clone(),
                room_guard.players.iter().map(|p| (p.id, room_guard.can_player_action(p.id))).collect::<Vec<_>>()
            )
        };
        
        let state_json = serde_json::to_string(&state_data).unwrap_or_default();
        
        let conns = connections.read().await;
        for (id, can_act) in player_info {
            if let Some(tx) = conns.get(&id) {
                let _ = tx.send(ServerMessage::GameStateUpdate {
                    state: state_json.clone(),
                });
                
                if can_act {
                    let _ = tx.send(ServerMessage::YourTurn);
                }
            }
        }
    }
}

async fn check_game_over(
    rooms: Arc<RwLock<HashMap<String, Arc<RwLock<GameRoom>>>>>,
    room_id: String,
    connections: Arc<RwLock<HashMap<PlayerId, mpsc::UnboundedSender<ServerMessage>>>>,
    economy: Arc<RwLock<EconomySystem>>,
) {
    let room = {
        let rooms_guard = rooms.read().await;
        rooms_guard.get(&room_id).cloned()
    };

    if let Some(room) = room {
        let (is_over, winner_id, outcomes, player_ids) = {
            let room_guard = room.read().await;
            if !room_guard.is_game_over() {
                return;
            }

            let winner = room_guard.get_winner();
            let mut outcomes = Vec::new();
            let mut ids = Vec::new();

            if let Some(winner_id_val) = winner {
                use guilin_paizi_economy::GameOutcome;
                for p in &room_guard.players {
                    let is_winner = p.id == winner_id_val;
                    let (huxi, duo, fan, zimo, tianhu, dihu) = if is_winner {
                        room_guard.game_state.win_result.as_ref().map(|res| {
                            (res.huxi, res.duo, res.fan, res.is_zimo, res.is_tianhu, res.is_dihu)
                        }).unwrap_or((0, 0, 0, false, false, false))
                    } else {
                        (0, 0, 0, false, false, false)
                    };

                    outcomes.push(GameOutcome {
                        player_id: p.id,
                        is_winner,
                        huxi,
                        duo,
                        fan,
                        is_zimo: zimo,
                        is_tianhu: tianhu,
                        is_dihu: dihu,
                        skill_modifiers: Vec::new(),
                    });
                }
            }
            
            for p in &room_guard.players {
                ids.push(p.id);
            }

            (true, winner, outcomes, ids)
        };

        if is_over {
            if let Some(winner) = winner_id {
                let settlement_results = {
                    let mut econ = economy.write().await;
                    // Note: Here we'd ideally pass game_state but we released the lock.
                    // For settlement purpose, we might need to clone game_state or pass only what's needed.
                    // Since check_game_over is complex, let's just use what we have.
                    // In a real project, we might need a snapshot of game_state.
                    
                    // Re-acquire lock briefly to get game_state
                    let room_guard = room.read().await;
                    econ.process_and_apply_game_result(&room_guard.game_state, outcomes)
                };
                
                let conns = connections.read().await;
                let econ = economy.write().await; // Briefly acquire to get balances
                for result in settlement_results {
                    if let Some(tx) = conns.get(&result.player_id) {
                        let _ = tx.send(ServerMessage::GameEnded { 
                            winner: Some(winner) 
                        });
                        let beans = econ.currency.get_balance(result.player_id).unwrap_or(0);
                        let _ = tx.send(ServerMessage::ChatMessage {
                             player_id: result.player_id,
                             message: format!("结算完成！获得/损失：{}，当前余额：{}", result.final_beans, beans)
                        });
                    }
                }
            } else {
                // 流局
                let conns = connections.read().await;
                for pid in player_ids {
                    if let Some(tx) = conns.get(&pid) {
                        let _ = tx.send(ServerMessage::GameEnded { winner: None });
                    }
                }
            }

            // 移除房间
            {
                let mut rooms_guard = rooms.write().await;
                rooms_guard.remove(&room_id);
            }
        }
    }
}
