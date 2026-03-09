const WebSocket = require('ws');

const ws = new WebSocket('ws://127.0.0.1:8080');

let roomId = null;

ws.on('open', () => {
    console.log('Connected');
});

ws.on('message', (data) => {
    let msgStr = data.toString();
    console.log("RAW: " + msgStr.substring(0, 500));
    try {
        let msg = JSON.parse(msgStr);
        console.log(`Parsed type: ${msg.type}`);

        if (msg.type === "Welcome") {
            ws.send(JSON.stringify({ type: "CreateRoom", max_players: 3 }));
        } else if (msg.type === "RoomCreated") {
            roomId = msg.room_id;
        } else if (msg.type === "RoomJoined") {
            console.log("Calling StartGame for room " + roomId);
            ws.send(JSON.stringify({ type: "StartGame", room_id: roomId }));
        } else if (msg.type === "GameStateUpdate") {
            console.log("Parsing state JSON...");
            let state = JSON.parse(msg.state);
            console.log("SUCCESS");
            process.exit(0);
        }
    } catch (e) {
        console.error("FAIL: " + e.message);
        process.exit(1);
    }
});
