use super::GameServer;

#[tokio::test]
async fn test_server_creation() {
    let server = GameServer::new();
    assert!(true, "Server created successfully");
}

#[tokio::test]
async fn test_room_creation() {
    let server = GameServer::new();
    
    // 测试创建房间
    server.create_room("test_room".to_string(), 4).await;
    
    // 获取房间
    let room = server.get_room("test_room").await;
    assert!(room.is_some(), "Room should be created");
    
    if let Some(room) = room {
        let room_guard = room.read().await;
        assert_eq!(room_guard.room_id, "test_room");
        assert_eq!(room_guard.max_players, 4);
    }
}

#[tokio::test]
async fn test_room_operations() {
    use guilin_paizi_core::PlayerId;
    
    let server = GameServer::new();
    server.create_room("test_room2".to_string(), 2).await;
    
    let room = server.get_room("test_room2").await.unwrap();
    let mut room_guard = room.write().await;
    
    let player_id = PlayerId::new();
    let added = room_guard.add_player(player_id);
    assert!(added, "Player should be added");
    
    assert_eq!(room_guard.get_player_count(), 1);
    assert!(!room_guard.is_full());
    
    room_guard.set_player_ready(player_id, true);
    assert!(room_guard.ready_players.contains(&player_id));
}

#[tokio::test]
async fn test_game_state_view() {
    use guilin_paizi_core::PlayerId;
    
    let server = GameServer::new();
    server.create_room("test_game".to_string(), 4).await;
    
    let room = server.get_room("test_game").await.unwrap();
    let room_guard = room.read().await;
    
    let view = room_guard.get_game_state_view();
    assert_eq!(view.players.len(), 0);
    assert_eq!(view.current_player_idx, 0);
    assert_eq!(view.round, 1);
}