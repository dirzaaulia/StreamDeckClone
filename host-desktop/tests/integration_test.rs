use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use prost::Message;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;

// Include generated protocol code for tests
pub mod streamdeck {
    include!(concat!(env!("OUT_DIR"), "/streamdeck.rs"));
}

use streamdeck::{
    deck_message, DeckMessage, HandshakeRequest, Heartbeat, KeyEvent, KeyEventType,
};

#[tokio::test]
async fn test_handshake_and_key_event() {
    let test_port = 4456;
    let addr = format!("127.0.0.1:{}", test_port);

    // Spawn server in background
    let server_addr = addr.clone();
    tokio::spawn(async move {
        // We can run the server directly or let it listen
        let listener = tokio::net::TcpListener::bind(&server_addr).await.unwrap();
        if let Ok((stream, _)) = listener.accept().await {
            let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
            let (mut write, mut read) = ws_stream.split();

            while let Some(Ok(msg)) = read.next().await {
                if let WsMessage::Binary(bytes) = msg {
                    let deck_msg = DeckMessage::decode(bytes.as_ref()).unwrap();
                    match deck_msg.payload {
                        Some(deck_message::Payload::HandshakeReq(_)) => {
                            let res = DeckMessage {
                                timestamp: 12345,
                                payload: Some(deck_message::Payload::HandshakeRes(
                                    streamdeck::HandshakeResponse {
                                        success: true,
                                        server_version: "0.1.0".to_string(),
                                        message: "OK".to_string(),
                                        active_profile_id: "default".to_string(),
                                        active_page_id: "main".to_string(),
                                        session_token: "test-token".to_string(),
                                    },
                                )),
                            };
                            let mut buf = Vec::new();
                            res.encode(&mut buf).unwrap();
                            write.send(WsMessage::Binary(buf.into())).await.unwrap();
                        }
                        Some(deck_message::Payload::Heartbeat(hb)) => {
                            let pong = DeckMessage {
                                timestamp: 12345,
                                payload: Some(deck_message::Payload::Heartbeat(Heartbeat {
                                    ping: hb.ping,
                                })),
                            };
                            let mut buf = Vec::new();
                            pong.encode(&mut buf).unwrap();
                            write.send(WsMessage::Binary(buf.into())).await.unwrap();
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    sleep(Duration::from_millis(100)).await;

    // Connect test client
    let url = format!("ws://{}", addr);
    let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect to test server");
    let (mut write, mut read) = ws_stream.split();

    // 1. Send Handshake
    let handshake_req = DeckMessage {
        timestamp: 100,
        payload: Some(deck_message::Payload::HandshakeReq(HandshakeRequest {
            device_id: "android-pixel-8".to_string(),
            device_name: "Pixel 8 Pro".to_string(),
            client_version: "1.0.0".to_string(),
            screen_width_dp: 412,
            screen_height_dp: 915,
            auth_token: "".to_string(),
        })),
    };
    let mut buf = Vec::new();
    handshake_req.encode(&mut buf).unwrap();
    write.send(WsMessage::Binary(buf.into())).await.unwrap();

    // 2. Read Handshake Response
    let resp = read.next().await.unwrap().unwrap();
    if let WsMessage::Binary(bytes) = resp {
        let decoded = DeckMessage::decode(bytes.as_ref()).unwrap();
        match decoded.payload {
            Some(deck_message::Payload::HandshakeRes(res)) => {
                assert!(res.success);
                assert_eq!(res.server_version, "0.1.0");
                assert_eq!(res.active_profile_id, "default");
            }
            other => panic!("Expected HandshakeRes, got {:?}", other),
        }
    } else {
        panic!("Expected binary WebSocket message");
    }

    // 3. Send KeyEvent (Slot 3, Play/Pause)
    let key_event = DeckMessage {
        timestamp: 200,
        payload: Some(deck_message::Payload::KeyEvent(KeyEvent {
            profile_id: "default".to_string(),
            page_id: "main".to_string(),
            key_index: 3,
            event_type: KeyEventType::Down as i32,
            delta_value: 0,
        })),
    };
    let mut key_buf = Vec::new();
    key_event.encode(&mut key_buf).unwrap();
    write.send(WsMessage::Binary(key_buf.into())).await.unwrap();

    // 4. Send Heartbeat
    let ping_msg = DeckMessage {
        timestamp: 300,
        payload: Some(deck_message::Payload::Heartbeat(Heartbeat { ping: 999 })),
    };
    let mut ping_buf = Vec::new();
    ping_msg.encode(&mut ping_buf).unwrap();
    write.send(WsMessage::Binary(ping_buf.into())).await.unwrap();

    let pong_resp = read.next().await.unwrap().unwrap();
    if let WsMessage::Binary(bytes) = pong_resp {
        let decoded = DeckMessage::decode(bytes.as_ref()).unwrap();
        match decoded.payload {
            Some(deck_message::Payload::Heartbeat(hb)) => {
                assert_eq!(hb.ping, 999);
            }
            other => panic!("Expected Heartbeat pong, got {:?}", other),
        }
    }
}


